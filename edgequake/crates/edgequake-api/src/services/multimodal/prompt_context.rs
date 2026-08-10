//! LightRAG prompt variable bundle (`prompt_multimodal.py` uniform template vars).

use super::context::SurroundingContext;
use super::manifest::ManifestItem;

/// Target language for `name` / `description` outputs (LightRAG `language`).
pub fn prompt_language() -> String {
    std::env::var("EDGEQUAKE_MM_PROMPT_LANGUAGE")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "English".into())
}

/// Variables shared by image/table/equation analysis prompts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptContext {
    pub language: String,
    pub captions: String,
    pub footnotes: String,
    pub leading: String,
    pub trailing: String,
    /// SPEC-015V Pass B system prompt overrides (None → built-in SSOT).
    pub image_system_prompt: Option<String>,
    pub chart_system_prompt: Option<String>,
    pub figure_system_prompt: Option<String>,
    /// SPEC-015V extract gates for specialize routing.
    pub extract_images: bool,
    pub extract_charts: bool,
    pub extract_figures: bool,
}

impl Default for PromptContext {
    fn default() -> Self {
        Self {
            language: "English".into(),
            captions: "n/a".into(),
            footnotes: "n/a".into(),
            leading: "n/a".into(),
            trailing: "n/a".into(),
            image_system_prompt: None,
            chart_system_prompt: None,
            figure_system_prompt: None,
            extract_images: true,
            extract_charts: true,
            extract_figures: true,
        }
    }
}

impl PromptContext {
    pub fn na_or(value: Option<&str>) -> String {
        value
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .unwrap_or("n/a")
            .to_string()
    }

    pub fn from_item_and_surrounding(
        item: &ManifestItem,
        surrounding: &SurroundingContext,
    ) -> Self {
        Self::from_parts(
            item.caption.as_deref(),
            item.footnote.as_deref(),
            surrounding,
        )
    }

    pub fn from_parts(
        caption: Option<&str>,
        footnote: Option<&str>,
        surrounding: &SurroundingContext,
    ) -> Self {
        Self {
            language: prompt_language(),
            captions: Self::na_or(caption),
            footnotes: Self::na_or(footnote),
            leading: Self::na_or(Some(surrounding.leading.as_str())),
            trailing: Self::na_or(Some(surrounding.trailing.as_str())),
            image_system_prompt: None,
            chart_system_prompt: None,
            figure_system_prompt: None,
            extract_images: true,
            extract_charts: true,
            extract_figures: true,
        }
    }

    /// Apply SPEC-015V resolved extract policy onto this prompt context.
    pub fn apply_vision_extract(&mut self, extract: &edgequake_pdf::VisionExtractConfig) {
        self.image_system_prompt = extract.image_system_prompt.clone();
        self.chart_system_prompt = extract.chart_system_prompt.clone();
        self.figure_system_prompt = extract.figure_system_prompt.clone();
        self.extract_images = extract.extract_images;
        self.extract_charts = extract.extract_charts;
        self.extract_figures = extract.extract_figures;
    }

    pub fn additional_context_block(&self) -> String {
        format!(
            "================ ADDITIONAL CONTEXT ================\n\
             - Captions: {}\n\n\
             - Footnotes: {}\n\n\
             - Leading Text:\n```\n{}\n```\n\n\
             - Trailing Text:\n```\n{}\n```",
            self.captions, self.footnotes, self.leading, self.trailing
        )
    }
}

/// Human-readable table format clause (LightRAG `table_content_format_label`).
pub fn table_content_format_label(fmt: &str) -> Result<String, String> {
    match fmt.trim().to_ascii_lowercase().as_str() {
        "html" => Ok(
            "HTML format — a <table> fragment where merged cells use rowspan/colspan and the header (if any) is inside <thead>".into(),
        ),
        "json" => Ok(
            "JSON format — a 2-D array where each inner array is one table row; the first row(s) may be the header".into(),
        ),
        other => Err(format!("unknown table format {other:?}; expected 'html' or 'json'")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_format_label_html_and_json() {
        assert!(table_content_format_label("html").unwrap().contains("HTML"));
        assert!(table_content_format_label("json").unwrap().contains("JSON"));
        assert!(table_content_format_label("xml").is_err());
    }

    #[test]
    fn prompt_context_uses_na_for_missing_caption() {
        let item = ManifestItem {
            item_id: "tb-1".into(),
            modality: "table".into(),
            start: 0,
            end: 0,
            matched: String::new(),
            asset_path: None,
            mime_type: None,
            body: None,
            caption: None,
            footnote: None,
            footnotes: Vec::new(),
            block_id: None,
            heading: None,
            analyze_result: None,
        };
        let ctx = PromptContext::from_item_and_surrounding(
            &item,
            &SurroundingContext {
                leading: "before".into(),
                trailing: String::new(),
            },
        );
        assert_eq!(ctx.captions, "n/a");
        assert_eq!(ctx.leading, "before");
        assert_eq!(ctx.trailing, "n/a");
    }
}
