//! Cross-crate UTF-8 text-safety helpers (preview / cap / span clamp).
//!
//! ## SOLID
//! - **S**: Pure string boundary math only — no I/O, tracing, or domain policy.
//! - **O/D**: Callers own ellipsis style, sentence preference, and byte budgets;
//!   they depend on this module instead of re-implementing `floor_char_boundary`.
//!
//! Hosted in `edgequake-observability` for dependency fan-in (api / pipeline /
//! tasks / query already link it). Prefer these over ad-hoc `&s[..n]` slices.
//!
//! Byte-index slices panic when `n` lands inside a multi-byte code point
//! (e.g. en-dash `–` = U+2013).

/// Prefix of `s` with at most `max_bytes` bytes, never splitting a UTF-8 char.
#[inline]
pub fn utf8_prefix(s: &str, max_bytes: usize) -> &str {
    &s[..s.floor_char_boundary(max_bytes)]
}

/// Like [`utf8_prefix`], owned; appends ASCII `...` when truncated.
#[inline]
pub fn utf8_prefix_ellipsis(s: &str, max_bytes: usize) -> String {
    if s.len() <= max_bytes {
        return s.to_string();
    }
    let budget = max_bytes.saturating_sub(3);
    format!("{}...", utf8_prefix(s, budget))
}

/// Clamp a byte span to valid UTF-8 boundaries within `s`.
///
/// `start` floors; `end` ceils; both are clamped to `s.len()` and ordered.
#[inline]
pub fn utf8_clamp_span(s: &str, start: usize, end: usize) -> (usize, usize) {
    let len = s.len();
    let start = s.floor_char_boundary(start.min(len));
    let end = s.ceil_char_boundary(end.min(len)).max(start);
    (start, end)
}

/// Truncate to `max_bytes`, preferring the last `.` / `!` / `?` within budget.
///
/// Always returns a UTF-8-safe prefix (falls back to [`utf8_prefix`] when no
/// sentence end is found). Optional `prefer_sep` cuts at the last occurrence of
/// that separator when it sits past `max_bytes / 4`.
pub fn utf8_prefix_at_sentence(s: &str, max_bytes: usize, prefer_sep: Option<&str>) -> String {
    if s.len() <= max_bytes {
        return s.to_string();
    }
    let mut end = s.floor_char_boundary(max_bytes);
    for (i, c) in s.char_indices() {
        if i >= max_bytes {
            break;
        }
        if matches!(c, '.' | '!' | '?') {
            end = i + c.len_utf8();
        }
    }
    if let Some(sep) = prefer_sep {
        if let Some(sep_at) = s[..end].rfind(sep) {
            if sep_at > max_bytes / 4 {
                return s[..sep_at].to_string();
            }
        }
    }
    s[..end].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utf8_prefix_floors_inside_en_dash() {
        let mut s = String::new();
        s.push_str(&"a".repeat(10));
        s.push('–');
        s.push_str("tail");
        let mid = 10 + 1;
        assert!(!s.is_char_boundary(mid));
        let p = utf8_prefix(&s, mid);
        assert_eq!(p.len(), 10);
        assert!(!p.contains('–'));
    }

    #[test]
    fn utf8_prefix_ellipsis_short_unchanged() {
        assert_eq!(utf8_prefix_ellipsis("hi", 100), "hi");
    }

    #[test]
    fn utf8_prefix_ellipsis_truncates_safely() {
        let mut s = String::new();
        s.push_str(&"x".repeat(20));
        s.push('–');
        let out = utf8_prefix_ellipsis(&s, 22);
        assert!(out.ends_with("..."));
        assert!(std::str::from_utf8(out.as_bytes()).is_ok());
    }

    #[test]
    fn utf8_clamp_span_floors_and_ceils() {
        let mut s = String::new();
        s.push_str(&"a".repeat(10));
        s.push('–'); // bytes 10..13
        s.push_str(&"b".repeat(10));
        let (start, end) = utf8_clamp_span(&s, 11, 12);
        assert_eq!(start, 10);
        assert_eq!(end, 13);
        assert!(s.is_char_boundary(start));
        assert!(s.is_char_boundary(end));
        let _ = &s[start..end];
    }

    #[test]
    fn utf8_prefix_at_sentence_prefers_period() {
        let s = "Hello world. More text here that exceeds.";
        let out = utf8_prefix_at_sentence(s, 20, None);
        assert!(out.ends_with('.'));
        assert!(out.len() <= 20);
    }

    #[test]
    fn utf8_prefix_at_sentence_floors_en_dash() {
        let mut s = String::new();
        s.push_str(&"a".repeat(20));
        s.push('–');
        s.push_str(&"b".repeat(20));
        let out = utf8_prefix_at_sentence(&s, 21, None);
        assert_eq!(out.len(), 20);
        assert!(!out.contains('–'));
    }
}
