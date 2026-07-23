//! SPEC-083 X-13 — Page marker SSOT (`PageMarkerWriter`).
//!
//! Grammar: `<!-- edgequake-page:N -->` (1-indexed). All writers and parsers
//! must go through this module so vision / EdgeParse / page-aware chunking
//! cannot drift.

/// Standard page marker prefix embedded in PDF-derived markdown.
pub const PAGE_MARKER_PREFIX: &str = "<!-- edgequake-page:";
/// Standard page marker suffix.
pub const PAGE_MARKER_SUFFIX: &str = " -->";

/// Single writer/parser for `<!-- edgequake-page:N -->` markers (X-13).
#[derive(Debug, Default, Clone, Copy)]
pub struct PageMarkerWriter;

impl PageMarkerWriter {
    /// Build the page marker string for a given 1-indexed page number.
    pub fn write(page: u32) -> String {
        format!("{PAGE_MARKER_PREFIX}{page}{PAGE_MARKER_SUFFIX}")
    }

    /// Parse page number from a marker line; `None` if not a marker.
    pub fn parse(line: &str) -> Option<u32> {
        let trimmed = line.trim();
        let inner = trimmed
            .strip_prefix(PAGE_MARKER_PREFIX)?
            .strip_suffix(PAGE_MARKER_SUFFIX)?;
        inner.trim().parse::<u32>().ok()
    }

    /// Strip every page marker (and the following newline when present).
    pub fn strip_all(markdown: &str) -> String {
        let mut out = String::with_capacity(markdown.len());
        for line in markdown.split_inclusive('\n') {
            let body = line.strip_suffix('\n').unwrap_or(line);
            if Self::parse(body).is_some() {
                continue;
            }
            out.push_str(line);
        }
        out
    }

    /// Strip existing markers then prepend a fresh marker for `page`.
    pub fn strip_before_restamp(markdown: &str, page: u32) -> String {
        let stripped = Self::strip_all(markdown);
        let body = stripped.trim_start_matches('\n');
        if body.is_empty() {
            Self::write(page)
        } else {
            format!("{}\n{}", Self::write(page), body)
        }
    }
}

/// Build the page marker string for a given 1-indexed page number.
pub fn make_page_marker(page: u32) -> String {
    PageMarkerWriter::write(page)
}

/// Parse page number from a marker line; `None` if not a marker.
pub fn parse_page_marker(line: &str) -> Option<u32> {
    PageMarkerWriter::parse(line)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contract_x_13_writer_roundtrip() {
        let marker = PageMarkerWriter::write(7);
        assert_eq!(marker, "<!-- edgequake-page:7 -->");
        assert_eq!(PageMarkerWriter::parse(&marker), Some(7));
        assert_eq!(
            PageMarkerWriter::parse("  <!-- edgequake-page:3 -->  "),
            Some(3)
        );
        assert!(PageMarkerWriter::parse("not a marker").is_none());
    }

    #[test]
    fn contract_x_13_strip_before_restamp() {
        let md = "<!-- edgequake-page:1 -->\nHello\n<!-- edgequake-page:2 -->\nWorld";
        let restamped = PageMarkerWriter::strip_before_restamp(md, 9);
        assert!(restamped.starts_with("<!-- edgequake-page:9 -->\n"));
        assert!(!restamped.contains("edgequake-page:1"));
        assert!(!restamped.contains("edgequake-page:2"));
        assert!(restamped.contains("Hello"));
        assert!(restamped.contains("World"));
    }
}
