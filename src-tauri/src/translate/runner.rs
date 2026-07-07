use std::path::Path;
use std::process::Stdio;
use std::sync::Arc;

use encoding_rs::GBK;
use tauri::{AppHandle, Emitter, Manager};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, BufReader};
use tokio::process::Child;
use tokio::sync::mpsc;

use crate::db::DbState;
use crate::secrets;
use crate::translate::args;
use crate::translate::history;
use crate::translate::model::{BabeldocInfo, OutputMode, TranslateEvent, TranslateRequest};
use crate::translate::progress::ProgressParser;

use super::state::{RunningTask, TaskRegistry};

pub async fn run_translate(app: AppHandle, task_id: String, req: TranslateRequest) {
    let emit = |ev: TranslateEvent| {
        let _ = app.emit("translate://progress", &ev);
    };

    // 1. Resolve API key.
    let api_key = {
        let Some(db) = app.try_state::<DbState>() else {
            emit(TranslateEvent::Status {
                task_id: task_id.clone(),
                status: "error".into(),
                output_files: None,
                message: Some("数据库未初始化，无法读取 API Key。".into()),
            });
            return;
        };
        let conn = db.conn.lock().unwrap();
        match secrets::get_secret(&conn, &req.provider.api_key_id) {
            Ok(Some(k)) => k,
            Ok(None) => {
                emit(TranslateEvent::Status {
                    task_id: task_id.clone(),
                    status: "error".into(),
                    output_files: None,
                    message: Some("API Key 未找到，请先在 API 配置页填写并保存。".into()),
                });
                return;
            }
            Err(e) => {
                emit(TranslateEvent::Status {
                    task_id: task_id.clone(),
                    status: "error".into(),
                    output_files: None,
                    message: Some(format!("读取 API Key 失败: {e}")),
                });
                return;
            }
        }
    };

    // 2. Probe babeldoc.
    match probe_babeldoc(Some(&app)).await {
        info if !info.installed => {
            emit(TranslateEvent::Status {
                task_id: task_id.clone(),
                status: "error".into(),
                output_files: None,
                message: Some(info.hint),
            });
            return;
        }
        _ => {}
    }

    // 3. Build args + spawn.
    let argv = args::build_args(&req, &api_key);
    let mut cmd = match build_command(&app, &argv) {
        Ok(cmd) => cmd,
        Err(e) => {
            emit(TranslateEvent::Status {
                task_id: task_id.clone(),
                status: "error".into(),
                output_files: None,
                message: Some(e.to_string()),
            });
            return;
        }
    };

    cmd.stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        // Force UTF-8 so Chinese Windows console encoding doesn't garble stderr
        // (rich's Windows renderer otherwise hits UnicodeEncodeError on non-GBK chars).
        .env("PYTHONUTF8", "1")
        .env("PYTHONIOENCODING", "utf-8")
        // Disable rich's legacy Windows renderer so it writes UTF-8 to the pipe
        // instead of trying the GBK console codepage.
        .env("PYTHONLEGACYWINDOWSSTDIO", "")
        .env("FORCE_COLOR", "0")
        .env("NO_COLOR", "1");

    let mut child: Child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            emit(TranslateEvent::Status {
                task_id: task_id.clone(),
                status: "error".into(),
                output_files: None,
                message: Some(format!("启动 babeldoc 失败: {e}")),
            });
            return;
        }
    };

    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    let (cancel_tx, mut cancel_rx) = mpsc::unbounded_channel::<()>();

    emit(TranslateEvent::Status {
        task_id: task_id.clone(),
        status: "running".into(),
        output_files: None,
        message: None,
    });

    if let Some(reg) = app.try_state::<Arc<TaskRegistry>>() {
        reg.insert(
            task_id.clone(),
            RunningTask {
                cancel_tx,
                status: "running".into(),
            },
        )
        .await;
    }

    let task_id_for_err = task_id.clone();
    let task_id_for_wait = task_id.clone();
    let app_for_wait = app.clone();

    // Spawn the stderr reader (progress + logs). stdout is mostly empty for babeldoc
    // but we drain it too so the pipe doesn't block.
    if let Some(stderr) = stderr {
        let app2 = app.clone();
        let tid = task_id.clone();
        tokio::spawn(async move {
            read_stderr(app2, tid, stderr).await;
        });
    }
    if let Some(stdout) = stdout {
        let app2 = app.clone();
        let tid = task_id.clone();
        tokio::spawn(async move {
            let mut reader = BufReader::new(stdout);
            let mut buf = Vec::with_capacity(4096);
            loop {
                buf.clear();
                match reader.read_until(b'\n', &mut buf).await {
                    Ok(0) => break,
                    Ok(_) => {
                        let line = decode_process_output(&buf).trim().to_string();
                        if !line.is_empty() {
                            let _ = app2.emit(
                                "translate://progress",
                                &TranslateEvent::Log {
                                    task_id: tid.clone(),
                                    line,
                                    stream: "stdout".into(),
                                },
                            );
                        }
                    }
                    Err(_) => break,
                }
            }
        });
    }

    let status = tokio::select! {
        wait = child.wait() => wait,
        _ = cancel_rx.recv() => {
            let _ = child.start_kill();
            child.wait().await
        }
    };
    let exit_ok = matches!(&status, Ok(s) if s.success());

    let was_cancelled = if let Some(reg) = app.try_state::<Arc<TaskRegistry>>() {
        matches!(
            reg.status(&task_id_for_wait).await.as_deref(),
            Some("cancelled")
        )
    } else {
        false
    };

    // Clear the registry entry.
    if let Some(reg) = app.try_state::<Arc<TaskRegistry>>() {
        reg.set_status(
            &task_id_for_wait,
            if exit_ok {
                "success"
            } else if was_cancelled {
                "cancelled"
            } else {
                "cancelled_or_error"
            },
        )
        .await;
        reg.remove(&task_id_for_wait).await;
    }

    if was_cancelled {
        persist_status(
            &app_for_wait,
            &task_id_for_wait,
            "cancelled",
            None,
            None,
            Some("user cancelled".into()),
        );
        let _ = app_for_wait.emit(
            "translate://progress",
            &TranslateEvent::Status {
                task_id: task_id_for_wait.clone(),
                status: "cancelled".into(),
                output_files: None,
                message: Some("用户已取消".into()),
            },
        );
    } else if exit_ok {
        // Scan output dir for produced PDFs.
        let files = scan_outputs(&req, &task_id_for_wait);
        persist_status(
            &app_for_wait,
            &task_id_for_wait,
            "success",
            Some(files.clone()),
            Some(100),
            None,
        );
        let _ = app_for_wait.emit(
            "translate://progress",
            &TranslateEvent::Status {
                task_id: task_id_for_wait.clone(),
                status: "success".into(),
                output_files: Some(files),
                message: None,
            },
        );
    } else {
        // Distinguish "cancelled" from "error": if the child was killed via the registry,
        // treat as cancelled; otherwise error. We approximate by checking the registry's
        // last status — but we already removed the entry. Simpler: examine exit signal.
        let (status_str, msg) = match &status {
            Ok(s) => (
                "error",
                format!("babeldoc 退出码 {}", s.code().unwrap_or(-1)),
            ),
            Err(_) => ("error", "babeldoc 执行失败".into()),
        };
        persist_status(
            &app_for_wait,
            &task_id_for_err,
            status_str,
            None,
            None,
            Some(msg.clone()),
        );
        let _ = app_for_wait.emit(
            "translate://progress",
            &TranslateEvent::Status {
                task_id: task_id_for_err.clone(),
                status: status_str.into(),
                output_files: None,
                message: Some(msg),
            },
        );
    }
}

fn persist_status(
    app: &AppHandle,
    task_id: &str,
    status: &str,
    output_files: Option<Vec<String>>,
    progress: Option<u32>,
    message: Option<String>,
) {
    let _ = history::update_status(app, task_id, status, progress, None, output_files, message);
}

fn build_command(
    app: &AppHandle,
    argv: &[String],
) -> crate::error::AppResult<tokio::process::Command> {
    let Some(sidecar) = resolve_sidecar(Some(app)) else {
        return Err(crate::error::AppError::NotFound(
            "未检测到内置 BabelDOC sidecar，请重新安装 PageWeave。".into(),
        ));
    };
    let mut c = tokio::process::Command::new(sidecar);
    c.args(argv);
    hide_child_console(&mut c);
    Ok(c)
}

pub(crate) fn hide_child_console(cmd: &mut tokio::process::Command) {
    #[cfg(windows)]
    {
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
}

/// Look for a bundled `babeldoc-sidecar(.exe)` next to the running app executable.
/// PyInstaller builds this as a one-folder sidecar, so the exe is only usable
/// when its sibling `_internal/` runtime directory is present too.
pub(crate) fn resolve_sidecar(app: Option<&AppHandle>) -> Option<std::path::PathBuf> {
    let candidates = [
        "babeldoc-sidecar.exe",
        "babeldoc-sidecar-x86_64-pc-windows-msvc.exe",
        "babeldoc-sidecar",
    ];
    // 1. Tauri resource directory. Packaged builds place the one-folder sidecar here.
    if let Some(app) = app {
        if let Ok(resource_dir) = app.path().resource_dir() {
            if let Some(path) =
                find_sidecar_in_dir(&resource_dir.join("babeldoc-sidecar"), &candidates)
            {
                return Some(path);
            }
        }
    }
    // 2. Next to the current executable.
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            if let Some(path) = find_sidecar_in_dir(dir, &candidates) {
                return Some(path);
            }
            // 3. In a sibling `sidecar/` subdirectory (dev layout).
            let sub = dir.join("sidecar");
            if let Some(path) = find_sidecar_in_dir(&sub, &candidates) {
                return Some(path);
            }
        }
    }
    // 4. Dev convenience: project-side built sidecar (best-effort, ignored if absent).
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let p = std::path::Path::new(&manifest_dir)
            .join("..")
            .join("sidecar")
            .join("dist")
            .join("babeldoc-sidecar")
            .join("babeldoc-sidecar.exe");
        if is_usable_sidecar(&p) {
            return Some(p);
        }
    }
    None
}

fn find_sidecar_in_dir(dir: &std::path::Path, candidates: &[&str]) -> Option<std::path::PathBuf> {
    candidates
        .iter()
        .map(|name| dir.join(name))
        .find(|path| is_usable_sidecar(path))
}

fn is_usable_sidecar(path: &std::path::Path) -> bool {
    if !path.is_file() {
        return false;
    }
    let Some(dir) = path.parent() else {
        return false;
    };
    let internal = dir.join("_internal");
    if !internal.is_dir() {
        return false;
    }
    std::fs::read_dir(internal)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .any(|entry| {
            let name = entry.file_name().to_string_lossy().to_ascii_lowercase();
            name.starts_with("python") && name.ends_with(".dll")
        })
}

async fn read_stderr<R: tokio::io::AsyncRead + Unpin>(app: AppHandle, task_id: String, reader: R) {
    let mut reader = reader;
    let mut parser = ProgressParser::new();
    let mut buf = [0u8; 4096];
    loop {
        match reader.read(&mut buf).await {
            Ok(0) => break,
            Ok(n) => {
                let lines = parser.push_bytes(&buf[..n]);
                for line in lines {
                    if let Some(overall) = line.overall {
                        let _ = app.emit(
                            "translate://progress",
                            &TranslateEvent::Progress {
                                task_id: task_id.clone(),
                                overall,
                                stage: line.stage.clone().unwrap_or_default(),
                                part_index: None,
                                total_parts: None,
                            },
                        );
                    }
                    let _ = app.emit(
                        "translate://progress",
                        &TranslateEvent::Log {
                            task_id: task_id.clone(),
                            line: mask_secrets(&line.text),
                            stream: "stderr".into(),
                        },
                    );
                }
            }
            Err(_) => break,
        }
    }
    // Flush trailing partial line.
    for line in parser.finish() {
        if let Some(overall) = line.overall {
            let _ = app.emit(
                "translate://progress",
                &TranslateEvent::Progress {
                    task_id: task_id.clone(),
                    overall,
                    stage: line.stage.clone().unwrap_or_default(),
                    part_index: None,
                    total_parts: None,
                },
            );
        }
        let _ = app.emit(
            "translate://progress",
            &TranslateEvent::Log {
                task_id: task_id.clone(),
                line: mask_secrets(&line.text),
                stream: "stderr".into(),
            },
        );
    }
}

/// Replace anything that looks like an API key on a log line with a mask.
fn mask_secrets(line: &str) -> String {
    // Mask `--openai-api-key sk-xxx` style.
    let masked = regex::Regex::new(r"(--openai-api-key\s+)(\S+)")
        .unwrap()
        .replace_all(line, "$1****")
        .to_string();
    // Mask bare `sk-...` tokens.
    regex::Regex::new(r"sk-[A-Za-z0-9]{6,}")
        .unwrap()
        .replace_all(&masked, "sk-****")
        .to_string()
}

/// Scan output_dir for the produced mono/dual PDFs of the first input file.
fn scan_outputs(req: &TranslateRequest, _task_id: &str) -> Vec<String> {
    let Some(pdf) = req.pdf_paths.first() else {
        return vec![];
    };
    let stem = Path::new(pdf)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");
    let dir = Path::new(&req.output_dir);
    let mut out = Vec::new();
    let want_mono = matches!(req.output_mode, OutputMode::Mono | OutputMode::Both);
    let want_dual = matches!(req.output_mode, OutputMode::Dual | OutputMode::Both);
    for entry in dir.read_dir().ok().into_iter().flatten().flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if want_mono && is_babeldoc_output(&name, stem, "mono") {
            out.push(entry.path().to_string_lossy().to_string());
        }
        if want_dual && is_babeldoc_output(&name, stem, "dual") {
            out.push(entry.path().to_string_lossy().to_string());
        }
    }
    out.sort();
    out
}

fn is_babeldoc_output(name: &str, stem: &str, kind: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    let stem_lower = stem.to_ascii_lowercase();
    lower == format!("{stem_lower}-{kind}.pdf")
        || lower.starts_with(&format!("{stem_lower}."))
            && (lower.ends_with(&format!(".{kind}.pdf"))
                || lower.ends_with(&format!(".{kind}.no_watermark.pdf")))
}

/// Probe whether the bundled BabelDOC sidecar is available.
pub async fn probe_babeldoc(app: Option<&AppHandle>) -> BabeldocInfo {
    if let Some(sidecar) = resolve_sidecar(app) {
        let mut cmd = tokio::process::Command::new(&sidecar);
        hide_child_console(&mut cmd);
        let version = cmd
            .arg("--version")
            .output()
            .await
            .ok()
            .map(|o| decode_process_output(&o.stdout).trim().to_string())
            .filter(|s| !s.is_empty());
        return BabeldocInfo {
            installed: true,
            version,
            path: Some(sidecar.to_string_lossy().to_string()),
            hint: String::new(),
        };
    }
    BabeldocInfo {
        installed: false,
        version: None,
        path: None,
        hint: "未检测到内置 BabelDOC sidecar，请重新安装 PageWeave。".into(),
    }
}

pub(crate) fn decode_process_output(bytes: &[u8]) -> String {
    match std::str::from_utf8(bytes) {
        Ok(s) => s.to_string(),
        Err(_) => {
            let (decoded, _, _) = GBK.decode(bytes);
            decoded.into_owned()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::translate::model::TranslateProvider;

    fn req(output_dir: String, mode: OutputMode) -> TranslateRequest {
        TranslateRequest {
            task_id: None,
            pdf_paths: vec!["C:/tmp/pageweave-smoke-valid.pdf".into()],
            output_dir,
            lang_in: "en".into(),
            lang_out: "zh".into(),
            output_mode: mode,
            provider: TranslateProvider {
                base_url: "https://example.test/v1".into(),
                api_key_id: "key_test".into(),
                model: "m".into(),
            },
            qps: 1,
            advanced: None,
        }
    }

    #[test]
    fn scans_current_babeldoc_output_names() {
        let dir = std::env::temp_dir().join(format!(
            "pageweave_scan_outputs_{}",
            uuid::Uuid::new_v4().simple()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("pageweave-smoke-valid.zh.mono.pdf"), b"mono").unwrap();
        std::fs::write(dir.join("pageweave-smoke-valid.zh.dual.pdf"), b"dual").unwrap();

        let outputs = scan_outputs(
            &req(dir.to_string_lossy().to_string(), OutputMode::Both),
            "t",
        );

        assert_eq!(outputs.len(), 2);
        assert!(outputs.iter().any(|p| p.ends_with(".zh.mono.pdf")));
        assert!(outputs.iter().any(|p| p.ends_with(".zh.dual.pdf")));

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn scans_legacy_babeldoc_output_names() {
        let dir = std::env::temp_dir().join(format!(
            "pageweave_scan_outputs_{}",
            uuid::Uuid::new_v4().simple()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("pageweave-smoke-valid-mono.pdf"), b"mono").unwrap();
        std::fs::write(dir.join("pageweave-smoke-valid-dual.pdf"), b"dual").unwrap();

        let outputs = scan_outputs(
            &req(dir.to_string_lossy().to_string(), OutputMode::Mono),
            "t",
        );

        assert_eq!(outputs.len(), 1);
        assert!(outputs[0].ends_with("-mono.pdf"));

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn decodes_utf8_before_falling_back_to_gbk() {
        assert_eq!(decode_process_output("中文".as_bytes()), "中文");
        assert_eq!(decode_process_output(&[0xd6, 0xd0, 0xce, 0xc4]), "中文");
    }
}
