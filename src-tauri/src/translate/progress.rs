use regex::Regex;

/// Stateful parser for babeldoc's rich/tqdm progress output on stderr.
///
/// rich refreshes the active progress line in place using `\r` (carriage return),
/// and prints newlines for log entries. We feed bytes via `push_bytes`, which
/// splits on `\r` and `\n`, and on each "logical line" either:
///   - extracts a percentage for a Progress update, or
///   - returns the cleaned (ANSI-stripped) line as a Log.
pub struct ProgressParser {
    pending: Vec<u8>,
    last_overall: u32,
    ansi_re: Regex,
    /// `stage (cur/total)` — rich/tqdm description line.
    stage_re: Regex,
    pct_re: Regex,
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
            stage_re: Regex::new(r"(.+?)\s*\((\d+)[/:](\d+)\)").unwrap(),
            pct_re: Regex::new(r"(\d{1,3})%").unwrap(),
        }
    }

    /// Append raw bytes and return any complete logical lines that resulted.
    pub fn push_bytes(&mut self, bytes: &[u8]) -> Vec<ParsedLine> {
        self.pending.extend_from_slice(bytes);
        let mut out = Vec::new();
        loop {
            // Find the next CR or LF.
            let pos = self
                .pending
                .iter()
                .position(|&b| b == b'\r' || b == b'\n');
            let Some(pos) = pos else { break };
            let line_bytes: Vec<u8> = self.pending.drain(..=pos).collect();
            // strip trailing CR/LF
            let mut text_bytes = &line_bytes[..];
            while !text_bytes.is_empty() && (text_bytes.last() == Some(&b'\r') || text_bytes.last() == Some(&b'\n')) {
                text_bytes = &text_bytes[..text_bytes.len() - 1];
            }
            if text_bytes.is_empty() {
                continue;
            }
            let raw = String::from_utf8_lossy(text_bytes).to_string();
            let clean = strip_ansi(&self.ansi_re, &raw);
            if clean.trim().is_empty() {
                continue;
            }
            let overall = self.pct_re.captures(&clean).and_then(|c| {
                c[1].parse::<u32>().ok().filter(|&v| v <= 100)
            });
            // For stage: rich prints `desc (cur/total)` then the bar.
            let stage = self.stage_re.captures(&clean).map(|c| c[1].trim().to_string());
            if let Some(v) = overall {
                self.last_overall = v;
            }
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
        let raw = String::from_utf8_lossy(&self.pending).to_string();
        self.pending.clear();
        let clean = strip_ansi(&self.ansi_re, &raw);
        if clean.trim().is_empty() {
            return vec![];
        }
        let overall = self.pct_re.captures(&clean).and_then(|c| {
            c[1].parse::<u32>().ok().filter(|&v| v <= 100)
        });
        let stage = self.stage_re.captures(&clean).map(|c| c[1].trim().to_string());
        vec![ParsedLine {
            text: clean,
            overall,
            stage,
        }]
    }
}

fn strip_ansi(re: &Regex, s: &str) -> String {
    re.replace_all(s, "").to_string()
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
}
