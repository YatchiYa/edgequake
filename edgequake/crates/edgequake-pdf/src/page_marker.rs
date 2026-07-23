//! SPEC-083 X-13 — Page marker SSOT for PDF backends (`PageMarkerWriter`).
//!
//! Grammar must stay identical to `edgequake_pipeline::PageMarkerWriter`
//! (`<!-- edgequake-page:N -->`, 1-indexed). Contract tests assert parity.

/// Standard page marker prefix embedded in PDF-derived markdown.
pub const PAGE_MARKER_PREFIX: &str = "<!-- edgequake-page:";
/// Standard page marker suffix.
pub const PAGE_MARKER_SUFFIX: &str = " -->";

/// Single writer/parser for page markers used by vision + EdgeParse (X-13).
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_page_marker_writer_ssot() {
        assert_eq!(PageMarkerWriter::write(1), "<!-- edgequake-page:1 -->");
        assert_eq!(
            PageMarkerWriter::parse("<!-- edgequake-page:12 -->"),
            Some(12)
        );
        let restamped =
            PageMarkerWriter::strip_before_restamp("<!-- edgequake-page:3 -->\nBody", 5);
        assert_eq!(restamped, "<!-- edgequake-page:5 -->\nBody");
    }
}
