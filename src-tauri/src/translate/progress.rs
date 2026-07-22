use regex::Regex;

use crate::translate::runner::decode_process_output;

/// Stateful parser for babeldoc's rich/tqdm progress output on stderr.
///
/// rich refreshes the active progress line in place using `\r` (carriage return),
/// and prints newlines for log entries. We feed bytes via `push_bytes`, which
/// splits on `\r` and `\n`, and on each "logical line" either:
///   - extracts a percentage for a Progress update, or
///   - returns the cleaned (ANSI-stripped) line as a Log.
///
/// Progress is monotonic: percentage never decreases for a running task.
pub struct ProgressParser {
    pending: Vec<u8>,
    last_overall: u32,
    ansi_re: Regex,
    /// Cursor-control / erase sequences that are not CSI `m` color codes.
    ansi_misc_re: Regex,
    /// `stage (cur/total)` — rich/tqdm description line.
    stage_re: Regex,
    desc_count_re: Regex,
    translate_count_re: Regex,
    pct_re: Regex,
    parenthesized_ratio_re: Regex,
    count_re: Regex,
    /// rich `MofNCompleteColumn`: `42/100` after stage text / bar glyphs.
    mofn_re: Regex,
    whitespace_re: Regex,
    trailing_mofn_re: Regex,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParsedLine {
    /// ANSI-stripped visible text of the line.
    pub text: String,
    /// Percentage parsed from the line, if any.
    pub overall: Option<u32>,
    pub stage: Option<String>,
}

impl ProgressParser {
    pub fn new() -> Self {
        Self {
            pending: Vec::new(),
            last_overall: 0,
            ansi_re: Regex::new("\x1b\\[[0-9;]*[A-Za-z]").unwrap(),
            ansi_misc_re: Regex::new("\x1b\\].*?(?:\x07|\x1b\\\\)|\x1b[()][AB012]|\x1b[>=]").unwrap(),
            stage_re: Regex::new(r"(.+?)\s*\((\d+)[/:](\d+)\)").unwrap(),
            desc_count_re: Regex::new(r"^(.+?)\s+-+\s+(\d+)(?:[/?](\d+|--))?").unwrap(),
            translate_count_re: Regex::new(r"^translate\s+(\d{1,3})(?:\b|$)").unwrap(),
            pct_re: Regex::new(r"(\d{1,3})\s*%").unwrap(),
            parenthesized_ratio_re: Regex::new(r"\(\d+[/:]\d+\)").unwrap(),
            count_re: Regex::new(r"\b(\d+)/(\d+)\b").unwrap(),
            mofn_re: Regex::new(r"(?:^|\s)(\d{1,6})/(\d{1,6})(?:\s|$)").unwrap(),
            whitespace_re: Regex::new(r"[ \t\u{00a0}]+").unwrap(),
            trailing_mofn_re: Regex::new(r"\s+\d+/\d+\s*$").unwrap(),
        }
    }

    /// Append raw bytes and return any complete logical lines that resulted.
    pub fn push_bytes(&mut self, bytes: &[u8]) -> Vec<ParsedLine> {
        self.pending.extend_from_slice(bytes);
        let mut out = Vec::new();
        loop {
            // Find the next CR or LF.
            let pos = self.pending.iter().position(|&b| b == b'\r' || b == b'\n');
            let Some(pos) = pos else { break };
            let line_bytes: Vec<u8> = self.pending.drain(..=pos).collect();
            // strip trailing CR/LF
            let mut text_bytes = &line_bytes[..];
            while !text_bytes.is_empty()
                && (text_bytes.last() == Some(&b'\r') || text_bytes.last() == Some(&b'\n'))
            {
                text_bytes = &text_bytes[..text_bytes.len() - 1];
            }
            if text_bytes.is_empty() {
                continue;
            }
            let raw = decode_process_output(text_bytes);
            let clean = self.clean_line(&raw);
            if clean.trim().is_empty() {
                continue;
            }
            let (overall, stage) = self.parse_clean_line(&clean);
            out.push(ParsedLine {
                text: clean,
                overall,
                stage,
            });
        }
        out
    }

    /// Current best-known overall progress.
    pub fn current(&self) -> u32 {
        self.last_overall
    }

    /// Flush any remaining pending bytes as a final line (used when the stream ends).
    pub fn finish(&mut self) -> Vec<ParsedLine> {
        if self.pending.is_empty() {
            return vec![];
        }
        let raw = decode_process_output(&self.pending);
        self.pending.clear();
        let clean = self.clean_line(&raw);
        if clean.trim().is_empty() {
            return vec![];
        }
        let (overall, stage) = self.parse_clean_line(&clean);
        vec![ParsedLine {
            text: clean,
            overall,
            stage,
        }]
    }

    fn clean_line(&self, raw: &str) -> String {
        let no_osc = self.ansi_misc_re.replace_all(raw, "");
        let no_csi = self.ansi_re.replace_all(&no_osc, "");
        // Collapse runs of spaces left by stripped control codes / bar glyphs.
        let collapsed = self.whitespace_re.replace_all(&no_csi, " ");
        collapsed.trim().to_string()
    }

    fn parse_clean_line(&mut self, clean: &str) -> (Option<u32>, Option<String>) {
        let overall = self
            .pct_re
            .captures(clean)
            .and_then(|c| c[1].parse::<u32>().ok().filter(|&v| v <= 100))
            .or_else(|| self.parse_count_progress(clean));
        // For stage: rich prints `desc (cur/total)` then the bar.
        let stage = self
            .stage_re
            .captures(clean)
            .map(|c| c[1].trim().to_string())
            .or_else(|| {
                self.desc_count_re
                    .captures(clean)
                    .map(|c| c[1].trim().to_string())
            })
            .or_else(|| {
                self.translate_count_re
                    .captures(clean)
                    .map(|_| "translate".to_string())
            })
            .or_else(|| extract_stage_prefix(clean, &self.trailing_mofn_re));
        let overall = overall.and_then(|v| self.commit_progress(v));
        (overall, stage)
    }

    /// Only accept non-decreasing progress so bar jitter / stage restarts don't go backwards.
    fn commit_progress(&mut self, v: u32) -> Option<u32> {
        if v > self.last_overall {
            self.last_overall = v;
            Some(v)
        } else if v == self.last_overall {
            Some(v)
        } else {
            // Still report current so stage labels keep updating, but keep overall.
            None
        }
    }

    fn parse_count_progress(&self, clean: &str) -> Option<u32> {
        // Prefer explicit % first (handled by caller).
        // Overall-only signals (never stage-local 12/40):
        //   1. `translate N` / `translate N/100` (babeldoc overall task)
        //   2. any MofN / cur/total with total==100 (rich overall bar)
        // Stage work counters stay stage labels only — mapping them to overall
        // spikes the bar early and then monotonic commit freezes later updates.
        if let Some(c) = self.translate_count_re.captures(clean) {
            if let Ok(v) = c[1].parse::<u32>() {
                if v <= 100 {
                    return Some(v);
                }
            }
        }

        let mut best_overall: Option<u32> = None;
        for c in self.mofn_re.captures_iter(clean) {
            let cur = match c[1].parse::<u32>() {
                Ok(v) => v,
                Err(_) => continue,
            };
            let total = match c[2].parse::<u32>() {
                Ok(v) => v,
                Err(_) => continue,
            };
            if total != 100 {
                continue;
            }
            let pct = cur.min(100);
            best_overall = Some(best_overall.map_or(pct, |b| b.max(pct)));
        }
        if best_overall.is_some() {
            return best_overall;
        }

        let without_stage_counts = self.parenthesized_ratio_re.replace_all(clean, "");
        self.count_re
            .captures_iter(&without_stage_counts)
            .filter_map(|c| {
                let cur = c[1].parse::<u32>().ok()?;
                let total = c[2].parse::<u32>().ok()?;
                if total == 100 {
                    return Some(cur.min(100));
                }
                None
            })
            .last()
            .or_else(|| {
                // Legacy tqdm: only on the overall "translate" description line.
                if !clean.to_ascii_lowercase().starts_with("translate") {
                    return None;
                }
                let c = self.desc_count_re.captures(clean)?;
                let cur = c[2].parse::<u32>().ok()?;
                let total = c.get(3)?.as_str().parse::<u32>().ok()?;
                if total == 0 {
                    return None;
                }
                Some(((cur.min(total) * 100) / total).min(100))
            })
    }
}

/// Heuristic stage label from free-form rich / log lines.
fn extract_stage_prefix(clean: &str, trailing_mofn_re: &Regex) -> Option<String> {
    let t = clean.trim();
    if t.is_empty() {
        return None;
    }
    // Skip pure bar / number noise.
    if t.chars()
        .all(|c| c.is_ascii_digit() || "%/:-. ".contains(c) || is_bar_glyph(c))
    {
        return None;
    }
    // Logger lines — leave as log-only (still shown in activity feed).
    if t.starts_with("INFO ")
        || t.starts_with("DEBUG ")
        || t.starts_with("WARNING ")
        || t.starts_with("ERROR ")
        || t.starts_with("CRITICAL ")
    {
        return None;
    }
    // Take leading word-ish tokens before bar glyphs or percent.
    let cut = t
        .find(is_bar_glyph)
        .or_else(|| t.find('%'))
        .unwrap_or(t.len());
    let head = t[..cut].trim();
    if head.is_empty() || head.len() > 80 {
        return None;
    }
    // Drop trailing mofn like "42/100" from head.
    let head = trailing_mofn_re.replace(head, "").trim().to_string();
    if head.is_empty() {
        return None;
    }
    // Only treat as stage if it looks like a task description (letters present).
    if head.chars().any(|c| c.is_alphabetic()) {
        Some(head)
    } else {
        None
    }
}

fn is_bar_glyph(c: char) -> bool {
    matches!(c, '|' | '▌' | '▎' | '▍' | '▊' | '▉' | '█' | '░' | '▒' | '▓')
        || ('\u{2580}'..='\u{259f}').contains(&c)
        || ('\u{2500}'..='\u{257f}').contains(&c)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_progress_line() {
        let mut p = ProgressParser::new();
        let lines = p.push_bytes(
            b"\x1b[36mtranslate\x1b[0m \x1b[32m\xe2\x96\x81\xe2\x96\x81\xe2\x96\x81 45%\r",
        );
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].overall, Some(45));
        assert!(lines[0].text.contains("45%"));
    }

    #[test]
    fn handles_split_across_feeds() {
        let mut p = ProgressParser::new();
        let a = p.push_bytes(b"some partial line without newline");
        assert!(a.is_empty());
        let b = p.push_bytes(b" and the rest\r");
        assert_eq!(b.len(), 1);
        assert!(b[0].text.contains("partial line"));
    }

    #[test]
    fn ignores_pct_over_100() {
        let mut p = ProgressParser::new();
        let lines = p.push_bytes(b"not a real 999% thing\n");
        assert!(lines[0].overall.is_none());
    }

    #[test]
    fn stage_local_counts_do_not_set_overall() {
        let mut p = ProgressParser::new();
        let lines = p.push_bytes(b"Parse Layout (1/1) ----- 15/20 0:00\n");
        assert_eq!(lines.len(), 1);
        // Stage work units must not become overall % (would freeze monotonic bar).
        assert_eq!(lines[0].overall, None);
        assert_eq!(lines[0].stage.as_deref(), Some("Parse Layout"));
        assert_eq!(p.current(), 0);
    }

    #[test]
    fn ignores_stage_count_when_work_count_is_unknown() {
        let mut p = ProgressParser::new();
        let lines = p.push_bytes(b"Translate Paragraphs (1/1) ----- 245/-- 0:00\n");
        assert_eq!(lines.len(), 1);
        // Stage still extracted; overall may be absent when total unknown.
        assert_eq!(lines[0].stage.as_deref(), Some("Translate Paragraphs"));
    }

    #[test]
    fn parses_rich_stage_count_without_total_as_stage_only() {
        let mut p = ProgressParser::new();
        let lines = p.push_bytes(b"translate -- 42 0:03 0:24\n");
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].stage.as_deref(), Some("translate"));
        assert_eq!(p.current(), 0);
    }

    #[test]
    fn parses_rich_stage_count_with_total() {
        let mut p = ProgressParser::new();
        let lines = p.push_bytes(b"translate -- 42/100 0:03 0:24\n");
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].overall, Some(42));
        assert_eq!(lines[0].stage.as_deref(), Some("translate"));
        assert_eq!(p.current(), 42);
    }

    #[test]
    fn ignores_elapsed_and_remaining_time_counts() {
        let mut p = ProgressParser::new();
        let lines = p.push_bytes(b"translate --- 69/100 0:08 0:04\n");
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].overall, Some(69));
        assert_eq!(lines[0].stage.as_deref(), Some("translate"));
        assert_eq!(p.current(), 69);
    }

    #[test]
    fn parses_translate_task_count_as_overall_progress() {
        let mut p = ProgressParser::new();
        let lines = p.push_bytes(b"translate 35\n");
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].overall, Some(35));
        assert_eq!(lines[0].stage.as_deref(), Some("translate"));
        assert_eq!(p.current(), 35);
    }

    #[test]
    fn decodes_gbk_log_lines() {
        let mut p = ProgressParser::new();
        let lines = p.push_bytes(&[0xd6, 0xd0, 0xce, 0xc4, b' ', b'4', b'5', b'%', b'\n']);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].text, "中文 45%");
        assert_eq!(lines[0].overall, Some(45));
    }

    #[test]
    fn parses_rich_mofn_without_percent() {
        let mut p = ProgressParser::new();
        // rich default: description + bar + MofNComplete (no % column)
        let lines = p.push_bytes("translate ━━━━━━━━━━━━━━━━━━━━━━━━━━━━ 42/100 0:03 0:24\r".as_bytes());
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].overall, Some(42));
        assert_eq!(lines[0].stage.as_deref(), Some("translate"));
    }

    #[test]
    fn progress_is_monotonic() {
        let mut p = ProgressParser::new();
        let a = p.push_bytes(b"translate 50%\n");
        assert_eq!(a[0].overall, Some(50));
        let b = p.push_bytes(b"Parse Layout (1/1) 10/100\n");
        // 10% would go backwards; reject as overall update, keep last at 50.
        assert!(b[0].overall.is_none() || b[0].overall == Some(50));
        assert_eq!(p.current(), 50);
        let c = p.push_bytes(b"translate 60%\n");
        assert_eq!(c[0].overall, Some(60));
        assert_eq!(p.current(), 60);
    }

    #[test]
    fn parses_stage_with_part_index_without_overall() {
        let mut p = ProgressParser::new();
        let lines =
            p.push_bytes(b"IL Translator (1/1) ------------------------------- 12/40 0:01 0:03\n");
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].stage.as_deref(), Some("IL Translator"));
        assert_eq!(lines[0].overall, None);
        // Overall only from the translate task bar.
        let t = p.push_bytes(b"translate ------------------------------- 42/100 0:01 0:03\n");
        assert_eq!(t[0].overall, Some(42));
        assert_eq!(p.current(), 42);
    }

    #[test]
    fn percent_on_stage_line_is_accepted_as_overall() {
        // Some rich columns print an explicit percentage on the overall bar only;
        // if a stage line carries %, treat it as overall (still monotonic).
        let mut p = ProgressParser::new();
        let lines = p.push_bytes(b"translate 18%\n");
        assert_eq!(lines[0].overall, Some(18));
    }
}
