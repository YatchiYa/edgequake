//! Lightweight document-language detection (SPEC-134 Slice D, WP-3).
//!
//! Transduction law: output language = source language at every text-generating
//! stage. One detection from the first non-empty Pass-A page is propagated
//! everywhere (Pass-B, verify, entity extraction). No LLM call — stopword and
//! script heuristics cover French / English / German / Spanish plus CJK/Hangul.

/// Detect the document's source language from conversion markdown.
///
/// Returns a canonical SPEC-096 name (`"French"`, `"English"`, …) or `None`
/// when the signal is too weak to commit (never force a language).
pub fn detect_document_language(markdown: &str) -> Option<&'static str> {
    let sample = first_nonempty_page_body(markdown)?;
    score_language(&sample)
}

fn first_nonempty_page_body(markdown: &str) -> Option<String> {
    const MARKER: &str = "<!-- edgequake-page:";
    let mut search = 0usize;
    let mut starts: Vec<usize> = Vec::new();
    while let Some(rel) = markdown[search..].find(MARKER) {
        starts.push(search + rel);
        search += rel + MARKER.len();
    }
    if starts.is_empty() {
        return scoreable_body(markdown);
    }
    for (i, &start) in starts.iter().enumerate() {
        let end = starts.get(i + 1).copied().unwrap_or(markdown.len());
        let section = &markdown[start..end];
        let body = section.find('\n').map(|n| &section[n + 1..]).unwrap_or("");
        if let Some(clean) = scoreable_body(body) {
            return Some(clean);
        }
    }
    None
}

fn scoreable_body(text: &str) -> Option<String> {
    let stripped = strip_markup(text);
    if stripped.chars().filter(|c| c.is_alphabetic()).count() < 20 {
        return None;
    }
    // Placeholder is lowercased by strip_markup — match the lowered form.
    if stripped.contains("no text extracted for this page") {
        return None;
    }
    Some(stripped)
}

fn strip_markup(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '<' {
            for n in chars.by_ref() {
                if n == '>' {
                    break;
                }
            }
            out.push(' ');
            continue;
        }
        if c == '!' && chars.peek() == Some(&'[') {
            // ![alt](url) — keep alt text, drop the url.
            chars.next();
            for n in chars.by_ref() {
                if n == ']' {
                    break;
                }
                if n.is_alphabetic() || n.is_whitespace() {
                    out.push(n);
                }
            }
            if chars.peek() == Some(&'(') {
                chars.next();
                for n in chars.by_ref() {
                    if n == ')' {
                        break;
                    }
                }
            }
            out.push(' ');
            continue;
        }
        if c == '$' {
            // Skip a LaTeX span (inline $...$).
            for n in chars.by_ref() {
                if n == '$' {
                    break;
                }
            }
            continue;
        }
        if c.is_alphabetic() || c.is_whitespace() || c == '\'' || c == '-' {
            out.push(c.to_ascii_lowercase());
        } else {
            out.push(' ');
        }
    }
    out
}

fn score_language(sample: &str) -> Option<&'static str> {
    if let Some(script) = detect_script(sample) {
        return Some(script);
    }
    let mut counts = [0u32; 4];
    const LANGS: [&str; 4] = ["French", "English", "German", "Spanish"];
    const WORDS: [&[&str]; 4] = [
        &[
            "les",
            "des",
            "une",
            "pour",
            "dans",
            "est",
            "pas",
            "avec",
            "aux",
            "cette",
            "ces",
            "mais",
            "nous",
            "vous",
            "sont",
            "etre",
            "très",
            "resultat",
            "essais",
            "homologation",
        ],
        &[
            "the", "and", "of", "to", "is", "that", "with", "this", "for", "are", "was", "from",
            "have", "been", "not",
        ],
        &[
            "der", "die", "das", "und", "den", "von", "ist", "auf", "ein", "eine", "nicht", "mit",
            "dem", "sich",
        ],
        &[
            "los", "las", "una", "por", "con", "del", "para", "como", "más", "está", "son", "una",
        ],
    ];
    for token in sample.split_whitespace() {
        let t = token.trim_matches(|c: char| !c.is_alphabetic());
        if t.len() < 2 {
            continue;
        }
        for (i, words) in WORDS.iter().enumerate() {
            if words.contains(&t) {
                counts[i] += 1;
            }
        }
    }
    let mut order: Vec<(usize, u32)> = counts.iter().copied().enumerate().collect();
    order.sort_by_key(|b| std::cmp::Reverse(b.1));
    let (best_i, best) = order[0];
    let second = order[1].1;
    if best < 3 {
        return None;
    }
    if second > 0 && best < second.saturating_mul(2) {
        return None;
    }
    Some(LANGS[best_i])
}

fn detect_script(sample: &str) -> Option<&'static str> {
    let mut cjk = 0u32;
    let mut hiragana_katakana = 0u32;
    let mut hangul = 0u32;
    for c in sample.chars() {
        match c {
            '\u{3040}'..='\u{30FF}' => {
                hiragana_katakana += 1;
                cjk += 1;
            }
            '\u{4E00}'..='\u{9FFF}' | '\u{3400}'..='\u{4DBF}' => cjk += 1,
            '\u{AC00}'..='\u{D7AF}' => hangul += 1,
            _ => {}
        }
    }
    if hangul >= 8 {
        return Some("Korean");
    }
    if hiragana_katakana >= 8 {
        return Some("Japanese");
    }
    if cjk >= 12 {
        return Some("Chinese");
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_french_from_first_page() {
        let md = "<!-- edgequake-page:1 -->\n\nRésultat essais mécanique.\nPas d'essai en réception.\n21 essais pour l'homologation.\nLes valeurs sont dans le tableau.\n";
        assert_eq!(detect_document_language(md), Some("French"));
    }

    #[test]
    fn detects_english_from_first_page() {
        let md = "<!-- edgequake-page:1 -->\n\nThe results of the mechanical tests are in this table and the values have been recorded.\n";
        assert_eq!(detect_document_language(md), Some("English"));
    }

    #[test]
    fn skips_empty_placeholder_page() {
        let md = "<!-- edgequake-page:1 -->\n\n*[No text extracted for this page; see page image below.]*\n\n<!-- edgequake-page:2 -->\n\nThe results of the mechanical tests are in this table and the values have been recorded.\n";
        assert_eq!(detect_document_language(md), Some("English"));
    }

    #[test]
    fn weak_signal_returns_none() {
        assert!(detect_document_language("<!-- edgequake-page:1 -->\n\n12 34 56\n").is_none());
        assert!(detect_document_language("").is_none());
    }

    #[test]
    fn detects_chinese_script() {
        let md =
            "<!-- edgequake-page:1 -->\n\n这是一份机械试验结果报告，包含所有测试数据和分析内容。\n";
        assert_eq!(detect_document_language(md), Some("Chinese"));
    }
}
