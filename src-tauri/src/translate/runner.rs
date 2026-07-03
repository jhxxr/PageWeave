use std::path::Path;
use std::process::Stdio;
use std::sync::Arc;

use tauri::{AppHandle, Emitter, Manager};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, BufReader};
use tokio::process::Child;
use tokio::sync::Mutex;

use crate::secrets;
use crate::translate::args;
use crate::translate::model::{BabeldocInfo, OutputMode, TranslateEvent, TranslateRequest};
use crate::translate::progress::ProgressParser;

use super::state::{RunningTask, TaskRegistry};

pub async fn run_translate(app: AppHandle, task_id: String, req: TranslateRequest) {
    let emit = |ev: TranslateEvent| {
        let _ = app.emit("translate://progress", &ev);
    };

    // 1. Resolve API key.
    let api_key = match secrets::get_secret(&req.provider.api_key_id) {
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
    };

    // 2. Probe babeldoc.
    match probe_babeldoc().await {
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
    let mut cmd = build_command(&argv);

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

    // Share the child between the registry (for cancel) and the runner (for wait).
    let child_slot: Arc<Mutex<Option<Child>>> = Arc::new(Mutex::new(Some(child)));

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
                child: child_slot.clone(),
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
                        let line = String::from_utf8_lossy(&buf).trim().to_string();
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

    // Take the child out of the shared slot so we can `wait` on it exclusively.
    // cancel_translate may have already killed it; `wait` will still observe exit.
    let mut owned_child = {
        let mut g = child_slot.lock().await;
        g.take()
    };

    let status = match owned_child.as_mut() {
        Some(c) => c.wait().await,
        None => // already taken (shouldn't happen) — synthesize an error
            Err(std::io::Error::new(std::io::ErrorKind::Other, "child already taken")),
    };
    let exit_ok = matches!(&status, Ok(s) if s.success());

    // Clear the registry entry.
    if let Some(reg) = app.try_state::<Arc<TaskRegistry>>() {
        reg.set_status(
            &task_id_for_wait,
            if exit_ok { "success" } else { "cancelled_or_error" },
        )
        .await;
        reg.remove(&task_id_for_wait).await;
    }

    if exit_ok {
        // Scan output dir for produced PDFs.
        let files = scan_outputs(&req, &task_id_for_wait);
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

fn build_command(argv: &[String]) -> tokio::process::Command {
    // Resolution order for the babeldoc binary:
    //   1. The bundled sidecar shipped next to this app's exe (preferred — no Python needed).
    //   2. A `babeldoc` console script on PATH (user-installed Python+BabelDOC).
    //   3. `python -m babeldoc` as a last resort.
    if let Some(sidecar) = resolve_sidecar() {
        let mut c = tokio::process::Command::new(sidecar);
        c.args(argv);
        return c;
    }
    if which::which("babeldoc").is_ok() {
        let mut c = tokio::process::Command::new("babeldoc");
        c.args(argv);
        return c;
    }
    let mut c = tokio::process::Command::new("python");
    c.arg("-m").arg("babeldoc").args(argv);
    c
}

/// Look for a bundled `babeldoc-sidecar(.exe)` next to the running app executable.
/// In dev this is `src-tauri/target/debug/`; in a bundled install it's the app dir.
/// Tauri's `externalBin` would rename it to `babeldoc-sidecar-<triple>.exe`, but we
/// also accept the plain name so a hand-placed exe works during development.
fn resolve_sidecar() -> Option<std::path::PathBuf> {
    let candidates = ["babeldoc-sidecar.exe", "babeldoc-sidecar"];
    // 1. Next to the current executable.
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            for name in candidates {
                let p = dir.join(name);
                if p.exists() {
                    return Some(p);
                }
            }
            // 2. In a sibling `sidecar/` subdirectory (dev layout).
            let sub = dir.join("sidecar");
            for name in candidates {
                let p = sub.join(name);
                if p.exists() {
                    return Some(p);
                }
            }
        }
    }
    // 3. Dev convenience: project-side built sidecar (best-effort, ignored if absent).
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let p = std::path::Path::new(&manifest_dir)
            .join("..")
            .join("sidecar")
            .join("dist")
            .join("babeldoc-sidecar")
            .join("babeldoc-sidecar.exe");
        if p.exists() {
            return Some(p);
        }
    }
    None
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
        if want_mono && name == format!("{stem}-mono.pdf") {
            out.push(entry.path().to_string_lossy().to_string());
        }
        if want_dual && name == format!("{stem}-dual.pdf") {
            out.push(entry.path().to_string_lossy().to_string());
        }
    }
    out
}

/// Probe whether babeldoc is available — either as a bundled sidecar next to
/// this app's exe, or as a `babeldoc` console script on PATH.
pub async fn probe_babeldoc() -> BabeldocInfo {
    // 1. Bundled sidecar (preferred; no Python needed).
    if let Some(sidecar) = resolve_sidecar() {
        let version = tokio::process::Command::new(&sidecar)
            .arg("--version")
            .output()
            .await
            .ok()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .filter(|s| !s.is_empty());
        return BabeldocInfo {
            installed: true,
            version,
            path: Some(sidecar.to_string_lossy().to_string()),
            hint: String::new(),
        };
    }
    // 2. User-installed `babeldoc` on PATH.
    let bin = match which::which("babeldoc") {
        Ok(p) => p,
        Err(_) => {
            return BabeldocInfo {
                installed: false,
                version: None,
                path: None,
                hint: "未检测到 babeldoc。请先安装 Python 3.10–3.13 并运行: pip install BabelDOC".into(),
            };
        }
    };
    let out = tokio::process::Command::new("babeldoc")
        .arg("--version")
        .output()
        .await;
    let version = match out {
        Ok(o) => String::from_utf8_lossy(&o.stdout).trim().to_string(),
        Err(_) => String::new(),
    };
    BabeldocInfo {
        installed: true,
        version: if version.is_empty() { None } else { Some(version) },
        path: Some(bin.to_string_lossy().to_string()),
        hint: String::new(),
    }
}
