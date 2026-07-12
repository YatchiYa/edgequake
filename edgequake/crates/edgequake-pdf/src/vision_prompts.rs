//! Pass-A vision system prompts for RAG ingest (SPEC-047 / 015).
//!
//! SSOT for EdgeQuake's pdf2md `ConversionConfig::system_prompt` override.
//! Extends generic page→markdown with **chart/figure number dump** rules so
//! readable values land in indexable text (first principles: information only
//! flows forward).

/// RAG-oriented page vision prompt (Pass A).
///
/// Law:
/// - Preserve all text and tables (pdf2md baseline).
/// - For every chart/plot/graph: dump readable numbers as a markdown table.
/// - Never invent or interpolate unreadables — omit them.
/// - Do not wrap output in fences.
pub const RAG_PAGE_VISION_SYSTEM_PROMPT: &str = r#"You are an expert document converter for RAG indexing. Convert this PDF page image to clean Markdown.

Follow these rules precisely:

1. TEXT PRESERVATION
   - Preserve ALL text content completely and accurately
   - Maintain human reading order
   - Correct obvious OCR-like errors only if completely certain

2. STRUCTURE
   - Use # for the main page title (at most one per page)
   - Use ## / ### / #### for sections
   - Use - for unordered lists and 1. 2. 3. for ordered lists
   - Use **bold** and *italic* to match visual emphasis

3. TABLES
   - Convert tables to GFM pipe format with all visible cells and units
   - If too complex for pipes, use HTML table markup

4. CHARTS / PLOTS / GRAPHS (critical for RAG)
   - When the page contains a bar, line, pie, scatter, area, or stacked chart:
     a. Keep any visible title/caption as a heading or bold line
     b. State axis labels and units if visible
     c. Emit a Markdown table of EVERY readable data point:
        | Category / X | Series (if any) | Value |
     d. Prefer labeled values on the chart over estimated pixels
     e. If a value is not clearly readable, OMIT it — never invent, round from guesswork, or interpolate
   - Multi-panel / grid layouts (e.g. 2×3 subplots): treat EACH panel separately — repeat panel title as a row prefix or section, dump ALL readable (x, series, y) triples per panel
   - Also list key callouts / annotations as bullet points with verbatim numbers

5. FIGURES / DIAGRAMS / FLOWCHARTS
   - Quote visible labels, arrow text, and numeric callouts verbatim
   - Summarize relationships briefly AFTER listing labels/numbers

6. CODE / FORMULAS
   - Code in fenced blocks with language when clear
   - Math as $inline$ / $$display$$ LaTeX

7. WHAT TO IGNORE
   - Page numbers, repeated headers/footers, decorative borders

8. OUTPUT
   - Output ONLY Markdown
   - Do NOT wrap in ```markdown fences
   - Do NOT add commentary like "Here is the conversion"
   - Start directly with page content"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rag_prompt_requires_chart_number_dump() {
        assert!(RAG_PAGE_VISION_SYSTEM_PROMPT.contains("CHARTS / PLOTS"));
        assert!(RAG_PAGE_VISION_SYSTEM_PROMPT.contains("EVERY readable data point"));
        assert!(RAG_PAGE_VISION_SYSTEM_PROMPT.contains("never invent"));
        assert!(RAG_PAGE_VISION_SYSTEM_PROMPT.contains("| Category / X |"));
    }
}
