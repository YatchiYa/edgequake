//! Parse human page-range strings into pdf2md [`PageSelection`] (SPEC-094).
//!
//! Supported forms (1-indexed):
//! - `"all"` — every page
//! - `"5"` — single page
//! - `"1-10"` — inclusive range
//! - `"1,3,5"` — explicit set

use edgequake_pdf2md::PageSelection;

use crate::error::PdfConversionError;

/// Parse a pages option string into [`PageSelection`].
pub fn parse_page_selection(raw: &str) -> Result<PageSelection, PdfConversionError> {
    let s = raw.trim().to_ascii_lowercase();
    if s.is_empty() || s == "all" {
        return Ok(PageSelection::All);
    }

    if let Some((start, end)) = s.split_once('-') {
        let start: usize = start.trim().parse().map_err(|_| {
            PdfConversionError::Backend(format!("Invalid start page in range: '{raw}'"))
        })?;
        let end: usize = end.trim().parse().map_err(|_| {
            PdfConversionError::Backend(format!("Invalid end page in range: '{raw}'"))
        })?;
        if start < 1 {
            return Err(PdfConversionError::Backend(format!(
                "Pages are 1-indexed, minimum is 1 (got {start})"
            )));
        }
        if start > end {
            return Err(PdfConversionError::Backend(format!(
                "Invalid page range '{start}-{end}': start must be <= end"
            )));
        }
        return Ok(PageSelection::Range(start, end));
    }

    if s.contains(',') {
        let mut pages = Vec::new();
        for part in s.split(',') {
            let page: usize = part.trim().parse().map_err(|_| {
                PdfConversionError::Backend(format!("Invalid page number: '{}'", part.trim()))
            })?;
            if page < 1 {
                return Err(PdfConversionError::Backend(format!(
                    "Pages are 1-indexed, minimum is 1 (got {page})"
                )));
            }
            pages.push(page);
        }
        return Ok(PageSelection::Set(pages));
    }

    let page: usize = s
        .parse()
        .map_err(|_| PdfConversionError::Backend(format!("Invalid page number: '{raw}'")))?;
    if page < 1 {
        return Err(PdfConversionError::Backend(format!(
            "Pages are 1-indexed, minimum is 1 (got {page})"
        )));
    }
    Ok(PageSelection::Single(page))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_all_forms() {
        assert!(matches!(
            parse_page_selection("all").unwrap(),
            PageSelection::All
        ));
        assert!(matches!(
            parse_page_selection("5").unwrap(),
            PageSelection::Single(5)
        ));
        assert!(matches!(
            parse_page_selection("1-10").unwrap(),
            PageSelection::Range(1, 10)
        ));
        assert!(matches!(
            parse_page_selection("1,3,5").unwrap(),
            PageSelection::Set(ref v) if v == &[1, 3, 5]
        ));
    }
}
