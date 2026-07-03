use crate::translate::model::{OutputMode, TranslateRequest};

/// Build the babeldoc CLI argument vector. The API key is intentionally passed via
/// `--openai-api-key` for MVP simplicity (known debt: process cmdline is visible to
/// other local processes; later we'll move to a `--config` toml with 0600 perms).
pub fn build_args(req: &TranslateRequest, api_key: &str) -> Vec<String> {
    let pdf = req
        .pdf_paths
        .first()
        .expect("translate requires at least one pdf path");
    let mut args = vec![
        "--files".into(),
        pdf.clone(),
        "--output".into(),
        req.output_dir.clone(),
        "--lang-in".into(),
        req.lang_in.clone(),
        "--lang-out".into(),
        req.lang_out.clone(),
        "--openai".into(),
        "--openai-model".into(),
        req.provider.model.clone(),
        "--openai-base-url".into(),
        req.provider.base_url.clone(),
        "--openai-api-key".into(),
        api_key.to_string(),
        "--qps".into(),
        req.qps.to_string(),
        // MVP defaults: better reader compat + auto OCR for scanned PDFs.
        "--enhance-compatibility".into(),
        "--auto-enable-ocr-workaround".into(),
        "--report-interval".into(),
        "0.1".into(),
    ];

    match req.output_mode {
        OutputMode::Mono => {
            args.push("--no-dual".into());
        }
        OutputMode::Dual => {
            args.push("--no-mono".into());
        }
        OutputMode::Both => {}
    }

    args
}
