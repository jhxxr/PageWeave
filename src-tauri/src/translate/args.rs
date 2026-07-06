use crate::translate::model::{OcrMode, OutputMode, TranslateRequest};

/// Build the babeldoc CLI argument vector. The API key is intentionally passed via
/// `--openai-api-key` for MVP simplicity (known debt: process cmdline is visible to
/// other local processes; later we'll move to a `--config` toml with 0600 perms).
///
/// No-regression invariant: when `req.advanced` is `None` or has every field
/// `None`, the returned vector is byte-identical to the pre-advanced-params
/// arg list. `default_advanced_matches_baseline` locks this.
pub fn build_args(req: &TranslateRequest, api_key: &str) -> Vec<String> {
    let pdf = req
        .pdf_paths
        .first()
        .expect("translate requires at least one pdf path");
    let a = req.advanced.as_ref();
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
    ];

    // ---- Compatibility bundle resolution ----
    // `enhance_compatibility = None` ⇒ true (historical default).
    // When ON, emit `--enhance-compatibility` and IGNORE the individual
    // sub-flags (skip_clean / disable_rich_text_translate / dual_translate_first)
    // — the bundle already implies them, and emitting `--dual-translate-first`
    // twice (once via bundle, once standalone) is messy.
    let enhance = a.and_then(|x| x.enhance_compatibility).unwrap_or(true);
    if enhance {
        args.push("--enhance-compatibility".into());
    } else {
        if a.and_then(|x| x.skip_clean).unwrap_or(false) {
            args.push("--skip-clean".into());
        }
        if a.and_then(|x| x.disable_rich_text_translate).unwrap_or(false) {
            args.push("--disable-rich-text-translate".into());
        }
        // dual_translate_first only meaningful in dual/both output.
        if req.output_mode != OutputMode::Mono
            && a.and_then(|x| x.dual_translate_first).unwrap_or(false)
        {
            args.push("--dual-translate-first".into());
        }
    }

    // ---- OCR tri-state: only one of these is emitted at a time ----
    // `ocr_mode = None` ⇒ Auto (historical default).
    match a.and_then(|x| x.ocr_mode).unwrap_or(OcrMode::Auto) {
        OcrMode::Auto => args.push("--auto-enable-ocr-workaround".into()),
        OcrMode::Off => args.push("--skip-scanned-detection".into()),
        OcrMode::Force => args.push("--ocr-workaround".into()),
    }

    // ---- Hardcoded MVP flags (not exposed in the advanced UI) ----
    args.push("--watermark-output-mode".into());
    args.push("no_watermark".into());
    args.push("--report-interval".into());
    args.push("0.1".into());

    // ---- output_mode (unchanged) ----
    match req.output_mode {
        OutputMode::Mono => args.push("--no-dual".into()),
        OutputMode::Dual => args.push("--no-mono".into()),
        OutputMode::Both => {}
    }

    // ---- Dual-only layout option ----
    if req.output_mode != OutputMode::Mono
        && a.and_then(|x| x.use_alternating_pages_dual).unwrap_or(false)
    {
        args.push("--use-alternating-pages-dual".into());
    }

    // ---- Curated optional flags (None = no arg) ----
    if let Some(p) = a.and_then(|x| x.pages.as_deref()) {
        if !p.is_empty() {
            args.push("--pages".into());
            args.push(p.to_string());
        }
    }
    if let Some(n) = a.and_then(|x| x.min_text_length) {
        args.push("--min-text-length".into());
        args.push(n.to_string());
    }
    if let Some(n) = a.and_then(|x| x.max_pages_per_part) {
        args.push("--max-pages-per-part".into());
        args.push(n.to_string());
    }

    if let Some(files) = a.and_then(|x| x.glossary_files.as_deref()) {
        if !files.is_empty() {
            args.push("--glossary-files".into());
            args.push(files.join(","));
        }
    }
    if a.and_then(|x| x.no_auto_extract_glossary).unwrap_or(false) {
        args.push("--no-auto-extract-glossary".into());
    }
    if a.and_then(|x| x.save_auto_extracted_glossary).unwrap_or(false) {
        args.push("--save-auto-extracted-glossary".into());
    }

    if let Some(f) = a.and_then(|x| x.primary_font_family.as_deref()) {
        if !f.is_empty() && f != "auto" {
            args.push("--primary-font-family".into());
            args.push(f.to_string());
        }
    }

    if a.and_then(|x| x.translate_table_text).unwrap_or(false) {
        args.push("--translate-table-text".into());
    }
    if a.and_then(|x| x.disable_graphic_element_process).unwrap_or(false) {
        args.push("--disable-graphic-element-process".into());
    }
    if a.and_then(|x| x.no_merge_alternating_line_numbers).unwrap_or(false) {
        args.push("--no-merge-alternating-line-numbers".into());
    }
    if a.and_then(|x| x.disable_same_text_fallback).unwrap_or(false) {
        args.push("--disable-same-text-fallback".into());
    }

    if a.and_then(|x| x.ignore_cache).unwrap_or(false) {
        args.push("--ignore-cache".into());
    }
    if let Some(n) = a.and_then(|x| x.pool_max_workers) {
        args.push("--pool-max-workers".into());
        args.push(n.to_string());
    }
    if let Some(n) = a.and_then(|x| x.term_pool_max_workers) {
        args.push("--term-pool-max-workers".into());
        args.push(n.to_string());
    }

    if let Some(s) = a.and_then(|x| x.custom_system_prompt.as_deref()) {
        if !s.is_empty() {
            args.push("--custom-system-prompt".into());
            args.push(s.to_string());
        }
    }
    if a.and_then(|x| x.no_send_temperature).unwrap_or(false) {
        args.push("--no-send-temperature".into());
    }
    if a.and_then(|x| x.enable_json_mode_if_requested).unwrap_or(false) {
        args.push("--enable-json-mode-if-requested".into());
    }
    if a.and_then(|x| x.send_dashscope_header).unwrap_or(false) {
        args.push("--send-dashscope-header".into());
    }
    if let Some(s) = a.and_then(|x| x.openai_reasoning.as_deref()) {
        if !s.is_empty() {
            args.push("--openai-reasoning".into());
            args.push(s.to_string());
        }
    }

    args
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::translate::model::{OutputMode, TranslateProvider};

    fn req() -> TranslateRequest {
        TranslateRequest {
            task_id: None,
            pdf_paths: vec!["C:/tmp/input.pdf".into()],
            output_dir: "C:/tmp/out".into(),
            lang_in: "en".into(),
            lang_out: "zh".into(),
            output_mode: OutputMode::Mono,
            provider: TranslateProvider {
                base_url: "https://example.test/v1".into(),
                api_key_id: "key_test".into(),
                model: "m".into(),
            },
            qps: 1,
            advanced: None,
        }
    }

    /// The exact arg list produced by the pre-advanced-params build_args.
    /// `default_advanced_matches_baseline` locks the no-regression invariant
    /// against this literal.
    fn baseline_args() -> Vec<String> {
        vec![
            "--files".into(),
            "C:/tmp/input.pdf".into(),
            "--output".into(),
            "C:/tmp/out".into(),
            "--lang-in".into(),
            "en".into(),
            "--lang-out".into(),
            "zh".into(),
            "--openai".into(),
            "--openai-model".into(),
            "m".into(),
            "--openai-base-url".into(),
            "https://example.test/v1".into(),
            "--openai-api-key".into(),
            "sk-test".into(),
            "--qps".into(),
            "1".into(),
            "--enhance-compatibility".into(),
            "--auto-enable-ocr-workaround".into(),
            "--watermark-output-mode".into(),
            "no_watermark".into(),
            "--report-interval".into(),
            "0.1".into(),
            "--no-dual".into(),
        ]
    }

    fn args_contain(args: &[String], flag: &str) -> bool {
        args.iter().any(|a| a == flag)
    }

    fn args_contain_pair(args: &[String], flag: &str, val: &str) -> bool {
        args.windows(2).any(|pair| pair[0] == flag && pair[1] == val)
    }

    #[test]
    fn requests_no_watermark_output() {
        let args = build_args(&req(), "sk-test");
        let watermark_arg = args
            .windows(2)
            .find(|pair| pair[0] == "--watermark-output-mode")
            .map(|pair| pair[1].as_str());

        assert_eq!(watermark_arg, Some("no_watermark"));
    }

    #[test]
    fn default_advanced_matches_baseline() {
        let args = build_args(&req(), "sk-test");
        assert_eq!(args, baseline_args());
    }

    #[test]
    fn empty_advanced_matches_baseline() {
        let mut r = req();
        r.advanced = Some(Default::default());
        let args = build_args(&r, "sk-test");
        assert_eq!(args, baseline_args());
    }

    #[test]
    fn compat_off_emits_subflags_not_bundle() {
        let mut r = req();
        r.output_mode = OutputMode::Dual;
        r.advanced = Some(crate::translate::model::AdvancedParams {
            enhance_compatibility: Some(false),
            skip_clean: Some(true),
            disable_rich_text_translate: Some(true),
            dual_translate_first: Some(true),
            ..Default::default()
        });
        let args = build_args(&r, "sk-test");
        assert!(!args_contain(&args, "--enhance-compatibility"));
        assert!(args_contain(&args, "--skip-clean"));
        assert!(args_contain(&args, "--disable-rich-text-translate"));
        assert!(args_contain(&args, "--dual-translate-first"));
    }

    #[test]
    fn compat_on_ignores_individuals() {
        let mut r = req();
        r.advanced = Some(crate::translate::model::AdvancedParams {
            enhance_compatibility: Some(true),
            skip_clean: Some(true),
            disable_rich_text_translate: Some(true),
            dual_translate_first: Some(true),
            ..Default::default()
        });
        let args = build_args(&r, "sk-test");
        assert!(args_contain(&args, "--enhance-compatibility"));
        assert!(!args_contain(&args, "--skip-clean"));
        assert!(!args_contain(&args, "--disable-rich-text-translate"));
        assert!(!args_contain(&args, "--dual-translate-first"));
    }

    #[test]
    fn ocr_off_emits_skip_scanned() {
        let mut r = req();
        r.advanced = Some(crate::translate::model::AdvancedParams {
            ocr_mode: Some(OcrMode::Off),
            ..Default::default()
        });
        let args = build_args(&r, "sk-test");
        assert!(args_contain(&args, "--skip-scanned-detection"));
        assert!(!args_contain(&args, "--auto-enable-ocr-workaround"));
        assert!(!args_contain(&args, "--ocr-workaround"));
    }

    #[test]
    fn ocr_force_emits_ocr_workaround() {
        let mut r = req();
        r.advanced = Some(crate::translate::model::AdvancedParams {
            ocr_mode: Some(OcrMode::Force),
            ..Default::default()
        });
        let args = build_args(&r, "sk-test");
        assert!(args_contain(&args, "--ocr-workaround"));
        assert!(!args_contain(&args, "--auto-enable-ocr-workaround"));
        assert!(!args_contain(&args, "--skip-scanned-detection"));
    }

    #[test]
    fn ocr_auto_is_default() {
        let mut r = req();
        r.advanced = Some(crate::translate::model::AdvancedParams {
            ocr_mode: Some(OcrMode::Auto),
            ..Default::default()
        });
        let args = build_args(&r, "sk-test");
        assert!(args_contain(&args, "--auto-enable-ocr-workaround"));
    }

    #[test]
    fn glossary_files_joined_with_comma() {
        let mut r = req();
        r.advanced = Some(crate::translate::model::AdvancedParams {
            glossary_files: Some(vec!["a.csv".into(), "b.csv".into()]),
            ..Default::default()
        });
        let args = build_args(&r, "sk-test");
        assert!(args_contain_pair(&args, "--glossary-files", "a.csv,b.csv"));
    }

    #[test]
    fn pages_emitted_when_set() {
        let mut r = req();
        r.advanced = Some(crate::translate::model::AdvancedParams {
            pages: Some("1-3".into()),
            ..Default::default()
        });
        let args = build_args(&r, "sk-test");
        assert!(args_contain_pair(&args, "--pages", "1-3"));
    }

    #[test]
    fn min_text_length_emitted_when_set() {
        let mut r = req();
        r.advanced = Some(crate::translate::model::AdvancedParams {
            min_text_length: Some(10),
            ..Default::default()
        });
        let args = build_args(&r, "sk-test");
        assert!(args_contain_pair(&args, "--min-text-length", "10"));
    }

    #[test]
    fn dual_translate_first_skipped_in_mono() {
        let mut r = req();
        // mono output + bundle off + dual_translate_first on
        r.advanced = Some(crate::translate::model::AdvancedParams {
            enhance_compatibility: Some(false),
            dual_translate_first: Some(true),
            ..Default::default()
        });
        let args = build_args(&r, "sk-test");
        assert!(!args_contain(&args, "--dual-translate-first"));
    }

    #[test]
    fn use_alternating_pages_dual_skipped_in_mono() {
        let mut r = req();
        r.advanced = Some(crate::translate::model::AdvancedParams {
            use_alternating_pages_dual: Some(true),
            ..Default::default()
        });
        let args = build_args(&r, "sk-test");
        assert!(!args_contain(&args, "--use-alternating-pages-dual"));
    }
}
