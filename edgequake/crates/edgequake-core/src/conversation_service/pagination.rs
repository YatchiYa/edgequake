//! Offset-cursor helpers for conversation list pagination (SPEC-141).
//!
//! The WebUI infinite query already sends `cursor`. Storage is offset/limit.
//! Encode the next offset as a decimal string so the existing client works
//! without a second pager.

use crate::types::PaginationMeta;

/// Parse `cursor` as a `usize` offset. Missing or garbage → 0.
pub(crate) fn offset_from_cursor(cursor: Option<&str>) -> usize {
    cursor
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .and_then(|c| c.parse::<usize>().ok())
        .unwrap_or(0)
}

/// Honest `has_more` / `next_cursor` from offset pagination.
///
/// `has_more` is `offset + taken < total`, not `total > taken` (the latter
/// is true on every last page when `total > limit`).
pub(crate) fn pagination_meta(
    offset: usize,
    limit: usize,
    taken: usize,
    total: usize,
) -> PaginationMeta {
    let next_offset = offset.saturating_add(taken);
    let has_more = next_offset < total;
    PaginationMeta {
        next_cursor: has_more.then(|| next_offset.to_string()),
        prev_cursor: (offset > 0).then(|| offset.saturating_sub(limit).to_string()),
        total: Some(total),
        has_more,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn garbage_cursor_is_offset_zero() {
        assert_eq!(offset_from_cursor(None), 0);
        assert_eq!(offset_from_cursor(Some("")), 0);
        assert_eq!(offset_from_cursor(Some("abc")), 0);
        assert_eq!(offset_from_cursor(Some("20")), 20);
    }

    #[test]
    fn last_page_has_no_next_cursor() {
        let meta = pagination_meta(20, 20, 5, 25);
        assert!(!meta.has_more);
        assert!(meta.next_cursor.is_none());
        assert_eq!(meta.prev_cursor.as_deref(), Some("0"));
    }

    #[test]
    fn full_page_with_remainder_sets_next_cursor() {
        let meta = pagination_meta(0, 20, 20, 25);
        assert!(meta.has_more);
        assert_eq!(meta.next_cursor.as_deref(), Some("20"));
    }
}
