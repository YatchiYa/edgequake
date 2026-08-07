//! Surrounding context for multimodal VLM prompts (LightRAG `multimodal_context.py` stub).
//!
//! Phase 4b: extract leading/trailing prose around a placeholder for future
//! prompt enrichment; images-only path uses this when drawing tags are present.

/// Extract leading and trailing text around a byte range in markdown.
pub fn surrounding_context(
    markdown: &str,
    start: usize,
    end: usize,
    max_chars: usize,
) -> (String, String) {
    let (start, end) = edgequake_observability::utf8_clamp_span(markdown, start, end);
    let leading = markdown[..start]
        .chars()
        .rev()
        .take(max_chars)
        .collect::<String>()
        .chars()
        .rev()
        .collect::<String>();
    let trailing = markdown[end..].chars().take(max_chars).collect::<String>();
    (leading.trim().to_string(), trailing.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_leading_and_trailing() {
        let md = "Before paragraph.\n<drawing id=\"x\" />\nAfter paragraph.";
        let tag = "<drawing id=\"x\" />";
        let start = md.find(tag).unwrap();
        let end = start + tag.len();
        let (lead, trail) = surrounding_context(md, start, end, 32);
        assert!(lead.contains("Before"));
        assert!(trail.contains("After"));
    }

    #[test]
    fn surrounding_context_tolerates_mid_char_offsets() {
        let md = format!("{}–{}", "a".repeat(10), "b".repeat(10));
        // En-dash occupies bytes 10..13; pass an interior offset.
        let (lead, trail) = surrounding_context(&md, 11, 12, 8);
        assert!(lead.is_char_boundary(lead.len()));
        assert!(trail.is_char_boundary(trail.len()));
        assert!(!lead.contains('\u{FFFD}'));
    }
}
