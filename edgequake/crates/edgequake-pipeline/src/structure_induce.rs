//! Structure induction for prose corpora that lack markdown headings (031 B3a).
//!
//! GraphRAG-Bench Acc medical text is a single ~1M-char blob with almost no
//! newlines or `#` headings. FAQ cues (`What is …?`, `How …?`) are inline.
//! This module lifts those cues into `##` headings so
//! [`crate::chunker::MarkdownChunking`] can attach section breadcrumbs for
//! extract/glean (LightRAG-style `---Section Context---`).

use regex::Regex;
use std::sync::OnceLock;

/// Env: `EDGEQUAKE_STRUCTURE_INDUCE=faq` (aliases: `1`, `true`, `yes`).
pub const STRUCTURE_INDUCE_ENV: &str = "EDGEQUAKE_STRUCTURE_INDUCE";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructureInduceMode {
    Faq,
}

pub fn structure_induce_mode_from_env() -> Option<StructureInduceMode> {
    let raw = std::env::var(STRUCTURE_INDUCE_ENV).ok()?;
    let v = raw.trim().to_ascii_lowercase();
    match v.as_str() {
        "faq" | "1" | "true" | "yes" | "on" => Some(StructureInduceMode::Faq),
        "0" | "false" | "no" | "off" | "" => None,
        _ => None,
    }
}

/// Apply env-selected induction, or return `text` unchanged.
pub fn maybe_induce_structure(text: &str) -> String {
    match structure_induce_mode_from_env() {
        Some(StructureInduceMode::Faq) => induce_faq_markdown(text),
        None => text.to_string(),
    }
}

fn faq_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"(?P<q>(?:What|What's|Whats|How|Which|When|Why|Where|Who|Is|Are|Can|Do|Does)\b[^?\n]{2,140}\?)",
        )
        .expect("faq regex")
    })
}

fn is_md_heading_line(ln: &str) -> bool {
    let t = ln.trim_start();
    let hashes = t.chars().take_while(|c| *c == '#').count();
    (1..=6).contains(&hashes) && t.as_bytes().get(hashes) == Some(&b' ')
}

fn markdown_heading_count(text: &str) -> usize {
    text.lines().filter(|ln| is_md_heading_line(ln)).count()
}

/// Lift inline FAQ questions into `##` headings for markdown chunking.
///
/// Idempotent when the text already has ≥3 markdown headings (real MD docs).
/// Consumes the matched question from the body (heading carries the cue).
pub fn induce_faq_markdown(text: &str) -> String {
    if text.trim().is_empty() {
        return String::new();
    }
    if markdown_heading_count(text) >= 3 {
        return text.to_string();
    }

    let re = faq_regex();
    let mut out = String::with_capacity(text.len().saturating_add(text.len() / 8));
    let mut last = 0usize;
    let mut inserted = 0usize;
    let mut prev_end = 0usize;

    for m in re.find_iter(text) {
        let q = m.as_str().trim();
        if q.len() < 8 {
            continue;
        }
        // Skip only true overlaps / glued duplicates (< 3 chars since last FAQ).
        if inserted > 0 && m.start().saturating_sub(prev_end) < 3 {
            continue;
        }
        // Require a sentence-ish left boundary (start, whitespace after punct, or start of run).
        if m.start() > 0 {
            let before = &text[..m.start()];
            let boundary_ok = match before.chars().rev().find(|c| !c.is_whitespace()) {
                None => true,
                Some(c) => matches!(c, '.' | '!' | '?' | ';' | ':' | '\n'),
            };
            if !boundary_ok {
                continue;
            }
        }

        out.push_str(&text[last..m.start()]);
        if !out.is_empty() && !out.ends_with('\n') {
            out.push('\n');
        }
        if !out.ends_with("\n\n") {
            if out.ends_with('\n') {
                out.push('\n');
            } else {
                out.push_str("\n\n");
            }
        }
        out.push_str("## ");
        out.push_str(q);
        out.push_str("\n\n");
        last = m.end();
        prev_end = m.end();
        inserted += 1;
    }
    out.push_str(&text[last..]);

    if inserted == 0 {
        return text.to_string();
    }
    tracing::info!(
        faq_headings = inserted,
        "031 B3a: induced FAQ markdown headings for structure-aware chunking"
    );
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn induces_headings_for_inline_faqs() {
        let prose = "About BCC. What is basal cell skin cancer? It is common. \
How is basal cell skin cancer treated? Surgery usually.";
        let md = induce_faq_markdown(prose);
        assert!(md.contains("## What is basal cell skin cancer?"));
        assert!(md.contains("## How is basal cell skin cancer treated?"));
        assert!(md.contains("It is common."));
        assert!(md.contains("Surgery usually."));
        // Question text consumed into heading (not duplicated as body lead).
        assert!(!md.contains("## What is basal cell skin cancer?\n\nWhat is basal cell"));
    }

    #[test]
    fn skips_already_structured_markdown() {
        let md = "# Doc\n\n## A\n\nbody\n\n## B\n\nmore\n\n## C\n\nend";
        assert_eq!(induce_faq_markdown(md), md);
    }

    #[test]
    fn no_faq_returns_original() {
        let t = "Basal cell skin cancer is common and usually cured with surgery.";
        assert_eq!(induce_faq_markdown(t), t);
    }
}
