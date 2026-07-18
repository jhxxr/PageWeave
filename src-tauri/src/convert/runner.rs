use std::path::Path;
use std::process::Stdio;
use std::sync::Arc;

use encoding_rs::GBK;
use tauri::{AppHandle, Emitter, Manager};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Child;
use tokio::sync::mpsc;

use crate::convert::args;
use crate::convert::model::{ConvertEvent, ConvertRequest, MarkitdownInfo};
use crate::convert::state::ConvertRegistry;

pub async fn run_convert(
    app: AppHandle,
    task_id: String,
    req: ConvertRequest,
    mut cancel_rx: mpsc::UnboundedReceiver<()>,
) {
    let emit = |ev: ConvertEvent| {
        let _ = app.emit("convert://progress", &ev);
    };

    // 1. Resolve output path + validate input.
    let output_path = match args::resolve_output_path(&req.input_path, &req.output_dir) {
        Ok(p) => p,
        Err(e) => {
            release_slot(&app, &task_id, "error").await;
            emit(ConvertEvent::Status {
                task_id: task_id.clone(),
                status: "error".into(),
                output_file: None,
                message: Some(e.to_string()),
            });
            return;
        }
    };
    let output_path_str = output_path.to_string_lossy().to_string();

    // 2. Probe sidecar.
    match probe_markitdown(Some(&app)).await {
        info if !info.installed => {
            release_slot(&app, &task_id, "error").await;
            emit(ConvertEvent::Status {
                task_id: task_id.clone(),
                status: "error".into(),
                output_file: None,
                message: Some(info.hint),
            });
            return;
        }
        _ => {}
    }

    // 3. Build command + spawn.
    let argv = args::build_args(&req.input_path, &output_path_str);
    let mut cmd = match build_command(&app, &argv) {
        Ok(cmd) => cmd,
        Err(e) => {
            release_slot(&app, &task_id, "error").await;
            emit(ConvertEvent::Status {
                task_id: task_id.clone(),
                status: "error".into(),
                output_file: None,
                message: Some(e.to_string()),
            });
            return;
        }
    };

    cmd.stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env("PYTHONUTF8", "1")
        .env("PYTHONIOENCODING", "utf-8")
        .env("FORCE_COLOR", "0")
        .env("NO_COLOR", "1");

    let mut child: Child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            release_slot(&app, &task_id, "error").await;
            emit(ConvertEvent::Status {
                task_id: task_id.clone(),
                status: "error".into(),
                output_file: None,
                message: Some(format!("启动 markitdown 失败: {e}")),
            });
            return;
        }
    };

    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    emit(ConvertEvent::Status {
        task_id: task_id.clone(),
        status: "running".into(),
        output_file: None,
        message: None,
    });

    let task_id_for_wait = task_id.clone();
    let app_for_wait = app.clone();

    if let Some(stderr) = stderr {
        let app2 = app.clone();
        let tid = task_id.clone();
        tokio::spawn(async move {
            drain_stream(app2, tid, stderr, "stderr").await;
        });
    }
    if let Some(stdout) = stdout {
        let app2 = app.clone();
        let tid = task_id.clone();
        tokio::spawn(async move {
            drain_stream(app2, tid, stdout, "stdout").await;
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

    let was_cancelled = if let Some(reg) = app.try_state::<Arc<ConvertRegistry>>() {
        matches!(
            reg.status(&task_id_for_wait).await.as_deref(),
            Some("cancelled")
        )
    } else {
        false
    };

    if let Some(reg) = app.try_state::<Arc<ConvertRegistry>>() {
        reg.set_status(
            &task_id_for_wait,
            if exit_ok {
                "success"
            } else if was_cancelled {
                "cancelled"
            } else {
                "error"
            },
        )
        .await;
        reg.remove(&task_id_for_wait).await;
    }

    if was_cancelled {
        let _ = app_for_wait.emit(
            "convert://progress",
            &ConvertEvent::Status {
                task_id: task_id_for_wait.clone(),
                status: "cancelled".into(),
                output_file: None,
                message: Some("用户已取消".into()),
            },
        );
        return;
    }

    if exit_ok {
        // Prefer the resolved path if it was written; otherwise scan for any new md.
        let output_file = if output_path.is_file() {
            Some(output_path_str)
        } else {
            find_output_md(&req.input_path, &req.output_dir)
        };
        if output_file.is_none() {
            let _ = app_for_wait.emit(
                "convert://progress",
                &ConvertEvent::Status {
                    task_id: task_id_for_wait.clone(),
                    status: "error".into(),
                    output_file: None,
                    message: Some("转换结束但未找到输出 Markdown 文件".into()),
                },
            );
            return;
        }
        // Reject empty outputs as soft failures so the user knows conversion was useless.
        if let Some(ref p) = output_file {
            if let Ok(meta) = std::fs::metadata(p) {
                if meta.len() == 0 {
                    let _ = app_for_wait.emit(
                        "convert://progress",
                        &ConvertEvent::Status {
                            task_id: task_id_for_wait.clone(),
                            status: "error".into(),
                            output_file: output_file.clone(),
                            message: Some(
                                "输出 Markdown 为空，请检查源文件是否包含可提取文本".into(),
                            ),
                        },
                    );
                    return;
                }
            }
        }
        let _ = app_for_wait.emit(
            "convert://progress",
            &ConvertEvent::Status {
                task_id: task_id_for_wait.clone(),
                status: "success".into(),
                output_file,
                message: None,
            },
        );
    } else {
        let msg = match &status {
            Ok(s) => format!("markitdown 退出码 {}", s.code().unwrap_or(-1)),
            Err(_) => "markitdown 执行失败".into(),
        };
        let _ = app_for_wait.emit(
            "convert://progress",
            &ConvertEvent::Status {
                task_id: task_id_for_wait.clone(),
                status: "error".into(),
                output_file: None,
                message: Some(msg),
            },
        );
    }
}

async fn release_slot(app: &AppHandle, task_id: &str, status: &str) {
    if let Some(reg) = app.try_state::<Arc<ConvertRegistry>>() {
        reg.set_status(task_id, status).await;
        reg.remove(task_id).await;
    }
}

fn find_output_md(input_path: &str, output_dir: &str) -> Option<String> {
    let stem = Path::new(input_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");
    let dir = Path::new(output_dir);
    let mut matches: Vec<String> = dir
        .read_dir()
        .ok()
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().to_string();
            let lower = name.to_ascii_lowercase();
            if lower == format!("{}.md", stem.to_ascii_lowercase())
                || (lower.starts_with(&format!("{}-", stem.to_ascii_lowercase()))
                    && lower.ends_with(".md"))
            {
                Some(entry.path().to_string_lossy().to_string())
            } else {
                None
            }
        })
        .collect();
    matches.sort();
    matches.pop()
}

fn build_command(
    app: &AppHandle,
    argv: &[String],
) -> crate::error::AppResult<tokio::process::Command> {
    let Some(sidecar) = resolve_markitdown_sidecar(Some(app)) else {
        return Err(crate::error::AppError::NotFound(
            "未检测到内置 markitdown sidecar，请重新安装 PageWeave。".into(),
        ));
    };
    let mut c = tokio::process::Command::new(sidecar);
    c.args(argv);
    hide_child_console(&mut c);
    Ok(c)
}

fn hide_child_console(cmd: &mut tokio::process::Command) {
    #[cfg(windows)]
    {
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
}

/// Look for a bundled `markitdown-sidecar(.exe)` with a usable `_internal/` sibling.
pub(crate) fn resolve_markitdown_sidecar(app: Option<&AppHandle>) -> Option<std::path::PathBuf> {
    let candidates = [
        "markitdown-sidecar.exe",
        "markitdown-sidecar-x86_64-pc-windows-msvc.exe",
        "markitdown-sidecar",
    ];
    if let Some(app) = app {
        if let Ok(resource_dir) = app.path().resource_dir() {
            if let Some(path) =
                find_sidecar_in_dir(&resource_dir.join("markitdown-sidecar"), &candidates)
            {
                return Some(path);
            }
        }
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            if let Some(path) = find_sidecar_in_dir(dir, &candidates) {
                return Some(path);
            }
            let sub = dir.join("sidecar");
            if let Some(path) = find_sidecar_in_dir(&sub, &candidates) {
                return Some(path);
            }
        }
    }
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let p = std::path::Path::new(&manifest_dir)
            .join("..")
            .join("sidecar")
            .join("dist")
            .join("markitdown-sidecar")
            .join("markitdown-sidecar.exe");
        if is_usable_sidecar(&p) {
            return Some(p);
        }
        let p2 = std::path::Path::new(&manifest_dir)
            .join("..")
            .join("sidecar")
            .join("dist")
            .join("markitdown-sidecar")
            .join("markitdown-sidecar-x86_64-pc-windows-msvc.exe");
        if is_usable_sidecar(&p2) {
            return Some(p2);
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

async fn drain_stream<R: tokio::io::AsyncRead + Unpin>(
    app: AppHandle,
    task_id: String,
    reader: R,
    stream: &str,
) {
    let mut reader = BufReader::new(reader);
    let mut buf = Vec::with_capacity(4096);
    loop {
        buf.clear();
        match reader.read_until(b'\n', &mut buf).await {
            Ok(0) => break,
            Ok(_) => {
                let line = decode_process_output(&buf).trim().to_string();
                if !line.is_empty() {
                    let _ = app.emit(
                        "convert://progress",
                        &ConvertEvent::Log {
                            task_id: task_id.clone(),
                            line,
                            stream: stream.into(),
                        },
                    );
                }
            }
            Err(_) => break,
        }
    }
}

fn decode_process_output(bytes: &[u8]) -> String {
    match std::str::from_utf8(bytes) {
        Ok(s) => s.to_string(),
        Err(_) => {
            let (decoded, _, _) = GBK.decode(bytes);
            decoded.into_owned()
        }
    }
}

/// Probe whether the bundled markitdown sidecar is available.
pub async fn probe_markitdown(app: Option<&AppHandle>) -> MarkitdownInfo {
    if let Some(sidecar) = resolve_markitdown_sidecar(app) {
        let mut cmd = tokio::process::Command::new(&sidecar);
        hide_child_console(&mut cmd);
        let version = cmd
            .arg("--version")
            .output()
            .await
            .ok()
            .map(|o| {
                let out = decode_process_output(&o.stdout);
                let err = decode_process_output(&o.stderr);
                let text = if !out.trim().is_empty() { out } else { err };
                text.trim().to_string()
            })
            .filter(|s| !s.is_empty());
        return MarkitdownInfo {
            installed: true,
            version,
            path: Some(sidecar.to_string_lossy().to_string()),
            hint: String::new(),
        };
    }
    MarkitdownInfo {
        installed: false,
        version: None,
        path: None,
        hint: "未检测到内置 markitdown sidecar，请重新安装 PageWeave 或先构建 sidecar。".into(),
    }
}
