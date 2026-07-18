use std::path::{Path, PathBuf};

use chrono::Local;

use crate::error::{AppError, AppResult};

/// MVP whitelist — keep in sync with frontend dialog filters.
pub const ALLOWED_EXTENSIONS: &[&str] = &["pdf", "docx", "pptx", "xlsx", "xls"];

/// True when `path` has a whitelist extension (case-insensitive).
pub fn is_allowed_extension(path: &str) -> bool {
    Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| {
            let lower = e.to_ascii_lowercase();
            ALLOWED_EXTENSIONS.iter().any(|a| *a == lower)
        })
        .unwrap_or(false)
}

/// Reject remote schemes; only local absolute-ish paths are accepted.
pub fn is_local_path(path: &str) -> bool {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return false;
    }
    let lower = trimmed.to_ascii_lowercase();
    if lower.starts_with("http://")
        || lower.starts_with("https://")
        || lower.starts_with("ftp://")
        || lower.starts_with("file://")
    {
        return false;
    }
    true
}

/// Resolve the output `.md` path under `output_dir`.
///
/// 1. `{stem}.md` if free
/// 2. `{stem}-YYYYMMDD-HHMMSS.md` (local time) if the candidate already exists
///
/// Never overwrites an existing file.
pub fn resolve_output_path(input_path: &str, output_dir: &str) -> AppResult<PathBuf> {
    if !is_local_path(input_path) {
        return Err(AppError::InvalidInput(
            "仅支持本地文件路径，不支持远程 URI".into(),
        ));
    }
    if !is_allowed_extension(input_path) {
        return Err(AppError::InvalidInput(format!(
            "不支持的文件类型，仅支持: {}",
            ALLOWED_EXTENSIONS
                .iter()
                .map(|e| format!(".{e}"))
                .collect::<Vec<_>>()
                .join(" / ")
        )));
    }
    if output_dir.trim().is_empty() {
        return Err(AppError::InvalidInput("请选择输出目录".into()));
    }
    if !is_local_path(output_dir) {
        return Err(AppError::InvalidInput("输出目录必须是本地路径".into()));
    }

    let input = Path::new(input_path);
    if !input.is_file() {
        return Err(AppError::NotFound(format!("输入文件不存在: {input_path}")));
    }

    let stem = input
        .file_stem()
        .and_then(|s| s.to_str())
        .filter(|s| !s.is_empty())
        .unwrap_or("output");

    let dir = PathBuf::from(output_dir);
    if !dir.exists() {
        std::fs::create_dir_all(&dir).map_err(|e| {
            AppError::Io(format!("无法创建输出目录 {}: {e}", dir.display()))
        })?;
    } else if !dir.is_dir() {
        return Err(AppError::InvalidInput(format!(
            "输出路径不是目录: {output_dir}"
        )));
    }

    let candidate = dir.join(format!("{stem}.md"));
    if !candidate.exists() {
        return Ok(candidate);
    }

    let stamp = Local::now().format("%Y%m%d-%H%M%S");
    let stamped = dir.join(format!("{stem}-{stamp}.md"));
    // Extremely unlikely collision within the same second; still avoid overwrite.
    if stamped.exists() {
        let nanos = Local::now().timestamp_subsec_millis();
        return Ok(dir.join(format!("{stem}-{stamp}-{nanos:03}.md")));
    }
    Ok(stamped)
}

/// Build markitdown CLI argv: `[input, "-o", output]`.
pub fn build_args(input_path: &str, output_path: &str) -> Vec<String> {
    vec![
        input_path.to_string(),
        "-o".into(),
        output_path.to_string(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn whitelist_is_case_insensitive() {
        assert!(is_allowed_extension(r"C:\docs\a.PDF"));
        assert!(is_allowed_extension(r"C:\docs\a.docx"));
        assert!(is_allowed_extension("report.XLSX"));
        assert!(!is_allowed_extension("notes.md"));
        assert!(!is_allowed_extension("image.png"));
    }

    #[test]
    fn rejects_remote_uri() {
        assert!(!is_local_path("https://example.com/a.pdf"));
        assert!(!is_local_path("http://x/y.docx"));
        assert!(is_local_path(r"C:\Users\me\file.pdf"));
    }

    #[test]
    fn resolve_output_uses_stem_md_when_free() {
        let dir = std::env::temp_dir().join(format!(
            "pageweave_convert_out_{}",
            uuid::Uuid::new_v4().simple()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let input = dir.join("sample.docx");
        std::fs::write(&input, b"x").unwrap();

        let out = resolve_output_path(
            &input.to_string_lossy(),
            &dir.to_string_lossy(),
        )
        .unwrap();
        assert_eq!(out.file_name().unwrap(), "sample.md");

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn resolve_output_stamps_when_exists() {
        let dir = std::env::temp_dir().join(format!(
            "pageweave_convert_out_{}",
            uuid::Uuid::new_v4().simple()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let input = dir.join("sample.pdf");
        std::fs::write(&input, b"x").unwrap();
        std::fs::write(dir.join("sample.md"), b"old").unwrap();

        let out = resolve_output_path(
            &input.to_string_lossy(),
            &dir.to_string_lossy(),
        )
        .unwrap();
        let name = out.file_name().unwrap().to_string_lossy();
        assert!(name.starts_with("sample-"));
        assert!(name.ends_with(".md"));
        assert_ne!(name, "sample.md");
        // Original must remain.
        assert_eq!(std::fs::read(dir.join("sample.md")).unwrap(), b"old");

        let _ = std::fs::remove_dir_all(dir);
    }
}
