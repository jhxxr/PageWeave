use std::path::{Path, PathBuf};
use std::process::Stdio;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use tokio::process::Command;

use crate::error::{AppError, AppResult};

const REPO_API_LATEST_RELEASE: &str =
    "https://api.github.com/repos/jhxxr/PageWeave/releases/latest";
const MIN_READY_CACHE_BYTES: u64 = 50 * 1024 * 1024;

#[derive(Debug, Clone, Serialize)]
pub struct OfflineAssetsInfo {
    pub installed: bool,
    pub cache_dir: String,
    pub size_bytes: u64,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct OfflineAssetsInstallResult {
    pub ok: bool,
    pub cache_dir: String,
    pub asset_name: Option<String>,
    pub message: String,
}

#[derive(Debug, Deserialize)]
struct GithubRelease {
    assets: Vec<GithubAsset>,
}

#[derive(Debug, Deserialize)]
struct GithubAsset {
    name: String,
    browser_download_url: String,
}

#[tauri::command]
pub async fn get_offline_assets_info() -> AppResult<OfflineAssetsInfo> {
    Ok(offline_assets_info())
}

#[tauri::command]
pub async fn install_offline_assets_from_file(
    path: String,
) -> AppResult<OfflineAssetsInstallResult> {
    let existing = offline_assets_info();
    if existing.installed {
        return Ok(OfflineAssetsInstallResult {
            ok: true,
            cache_dir: existing.cache_dir,
            asset_name: None,
            message: "离线资源已安装，无需重复恢复".into(),
        });
    }

    let package = PathBuf::from(path);
    if !package.exists() {
        return Err(AppError::NotFound("离线资源包不存在".into()));
    }
    if !package.is_file() {
        return Err(AppError::InvalidInput(
            "请选择 offline_assets_*.zip 文件".into(),
        ));
    }
    if !package
        .file_name()
        .and_then(|n| n.to_str())
        .map(is_offline_asset_name)
        .unwrap_or(false)
    {
        return Err(AppError::InvalidInput(
            "离线资源包文件名应为 offline_assets_*.zip".into(),
        ));
    }

    let restore_dir = package
        .parent()
        .ok_or_else(|| AppError::InvalidInput("无法读取资源包所在目录".into()))?;
    restore_offline_assets(restore_dir).await?;
    let info = offline_assets_info();
    Ok(OfflineAssetsInstallResult {
        ok: info.installed,
        cache_dir: info.cache_dir,
        asset_name: package
            .file_name()
            .and_then(|n| n.to_str())
            .map(str::to_string),
        message: if info.installed {
            "离线资源安装完成".into()
        } else {
            "已执行安装，但尚未检测到完整缓存".into()
        },
    })
}

#[tauri::command]
pub async fn install_offline_assets_from_release(
    app: AppHandle,
) -> AppResult<OfflineAssetsInstallResult> {
    let existing = offline_assets_info();
    if existing.installed {
        return Ok(OfflineAssetsInstallResult {
            ok: true,
            cache_dir: existing.cache_dir,
            asset_name: None,
            message: "离线资源已安装，无需重复下载".into(),
        });
    }

    let client = reqwest::Client::builder().user_agent("PageWeave").build()?;
    let release = client
        .get(REPO_API_LATEST_RELEASE)
        .send()
        .await?
        .error_for_status()?
        .json::<GithubRelease>()
        .await?;
    let asset = release
        .assets
        .into_iter()
        .find(|a| is_offline_asset_name(&a.name))
        .ok_or_else(|| AppError::NotFound("最新 Release 中未找到 offline_assets_*.zip".into()))?;

    let bytes = client
        .get(&asset.browser_download_url)
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?;

    let download_dir = app
        .path()
        .app_cache_dir()
        .map_err(|e| AppError::Io(e.to_string()))?
        .join("offline-assets");
    tokio::fs::create_dir_all(&download_dir).await?;
    let package_path = download_dir.join(&asset.name);
    tokio::fs::write(&package_path, &bytes).await?;

    restore_offline_assets(&download_dir).await?;
    let info = offline_assets_info();
    Ok(OfflineAssetsInstallResult {
        ok: info.installed,
        cache_dir: info.cache_dir,
        asset_name: Some(asset.name),
        message: if info.installed {
            "离线资源安装完成".into()
        } else {
            "已下载并执行安装，但尚未检测到完整缓存".into()
        },
    })
}

fn offline_assets_info() -> OfflineAssetsInfo {
    let cache_dir = babeldoc_cache_dir();
    let size_bytes = dir_size(&cache_dir).unwrap_or(0);
    let installed = size_bytes >= MIN_READY_CACHE_BYTES;
    OfflineAssetsInfo {
        installed,
        cache_dir: cache_dir.to_string_lossy().to_string(),
        size_bytes,
        message: if installed {
            "已检测到 BabelDOC 离线资源缓存".into()
        } else {
            "未检测到完整离线资源缓存".into()
        },
    }
}

async fn restore_offline_assets(asset_dir: &Path) -> AppResult<()> {
    let mut cmd = restore_command(asset_dir);
    cmd.stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env("PYTHONUTF8", "1")
        .env("PYTHONIOENCODING", "utf-8");

    let out = cmd.output().await?;
    if out.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
    let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
    let detail = if stderr.is_empty() { stdout } else { stderr };
    Err(AppError::Translate(format!(
        "恢复离线资源失败: {}",
        if detail.is_empty() {
            "未知错误"
        } else {
            &detail
        }
    )))
}

fn restore_command(asset_dir: &Path) -> Command {
    let mut cmd = if let Some(sidecar) = super::runner::resolve_sidecar() {
        Command::new(sidecar)
    } else if which::which("babeldoc").is_ok() {
        Command::new("babeldoc")
    } else {
        let mut c = Command::new("python");
        c.arg("-m").arg("babeldoc");
        c
    };
    cmd.arg("--restore-offline-assets").arg(asset_dir);
    cmd
}

fn is_offline_asset_name(name: &str) -> bool {
    name.starts_with("offline_assets_") && name.ends_with(".zip")
}

fn babeldoc_cache_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("BABELDOC_CACHE_DIR") {
        return PathBuf::from(dir);
    }
    if let Ok(dir) = std::env::var("XDG_CACHE_HOME") {
        return PathBuf::from(dir).join("babeldoc");
    }
    if let Ok(dir) = std::env::var("USERPROFILE").or_else(|_| std::env::var("HOME")) {
        return PathBuf::from(dir).join(".cache").join("babeldoc");
    }
    PathBuf::from(".cache").join("babeldoc")
}

fn dir_size(path: &Path) -> std::io::Result<u64> {
    let mut total = 0;
    if !path.exists() {
        return Ok(0);
    }
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let meta = entry.metadata()?;
        if meta.is_dir() {
            total += dir_size(&entry.path())?;
        } else {
            total += meta.len();
        }
    }
    Ok(total)
}
