//! Pass-A vision system prompts for RAG ingest (SPEC-047 / 015).
//!
//! SSOT for EdgeQuake's pdf2md `ConversionConfig::system_prompt` override.
//! Extends generic page→markdown with **chart/figure number dump** rules so
//! readable values land in indexable text (first principles: information only
//! flows forward).
//!
//! Also the SSOT for SPEC-049 two-pass figure filter prompts.
//! [`figure_filter.rs`] imports these so all prompt text lives in one place.

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

// ── SPEC-049 Two-pass figure filter prompts ────────────────────────────────────

/// System prompt for Pass-1 (filter): is this a real figure?
///
/// Axiom (SPEC-049/001 modality-split): a visual crop carries value only when
/// text/Markdown CANNOT express the same meaning.  The LLM is the semantic oracle.
pub const FIGURE_FILTER_PASS1_SYSTEM: &str = r#"You are a visual-content filter for a scientific PDF figure extraction pipeline.

Decide: is this image a REAL FIGURE with independent visual signal, or a FALSE POSITIVE?

First-principles axiom: a crop has value only when plain text / Markdown CANNOT carry
the same meaning (e.g. a bar chart, architecture diagram, photograph).

Classify into exactly ONE kind:

REAL FIGURES (is_figure: true):
  bar_chart, line_chart, scatter_plot, heatmap, histogram, pie_chart, radar_chart,
  architecture_diagram, flowchart, diagram, illustration, photograph, system_demo,
  table_visual

FALSE POSITIVES (is_figure: false):
  logo, icon_logo, text_block, decorative_rule, empty

Respond in JSON ONLY — no fences, no explanation:
{"kind":"<one of above>","is_figure":<true|false>,"confidence":<0.0-1.0>}"#;

/// User message for Pass-1 (constant — the image is attached separately).
pub fn figure_filter_pass1_prompt() -> &'static str {
    "Classify this PDF crop: real figure or false positive?"
}

/// System prompt for Pass-2 (specialize).
pub const FIGURE_FILTER_PASS2_SYSTEM: &str =
    "You are an expert at extracting structured information from scientific figures. \
     Respond in Markdown only.";

/// Kind-aware user prompt for Pass-2.  Returns a `&'static str` for all known kinds.
pub fn figure_filter_pass2_prompt(kind: &crate::figure_filter::FigureKind) -> &'static str {
    use crate::figure_filter::FigureKind::*;
    match kind {
        BarChart | Histogram =>
            "Extract from this bar/histogram chart:\n\
             1. **Title** (if visible)\n\
             2. **X-axis** label and tick values\n\
             3. **Y-axis** label and range\n\
             4. **Series/groups** (legend)\n\
             5. **Data table** (Markdown) with approximate bar values\n\
             6. **Key observations** (2–3 bullets)",
        LineChart =>
            "Extract from this line chart:\n\
             1. **Title** (if visible)\n\
             2. **X-axis** label and range\n\
             3. **Y-axis** label and range\n\
             4. **Series** names and trend direction\n\
             5. **Key observations** (2–3 bullets)\n\
             6. **Approximate data** (Markdown table if readable)",
        ScatterPlot =>
            "Extract from this scatter plot:\n\
             1. **Title** and axis labels\n\
             2. **Groups/clusters** (colour / shape coding)\n\
             3. **Key patterns or outliers** (2–3 bullets)",
        Heatmap =>
            "Extract from this heatmap:\n\
             1. **Title**, row labels, column labels\n\
             2. **Colour scale** meaning (high / low)\n\
             3. **Hotspot regions** that stand out\n\
             4. Reconstruct as **Markdown table** if axes are readable",
        PieChart =>
            "Extract from this pie chart:\n\
             1. **Title** (if visible)\n\
             2. **Slices**: label and approximate % for each\n\
             3. **Key observation** (largest / smallest slice)",
        RadarChart =>
            "Extract from this radar/spider chart:\n\
             1. **Axes** (dimension names)\n\
             2. **Series** compared (legend)\n\
             3. **Notable strengths / weaknesses** per dimension",
        ArchitectureDiagram =>
            "Describe this architecture diagram:\n\
             1. **Top-level components** (list each named box/module)\n\
             2. **Data flow** — what flows between components and in what direction?\n\
             3. **External interfaces** (APIs, databases, users)\n\
             4. **Key design decision** visible in the diagram",
        Flowchart =>
            "Describe this flowchart:\n\
             1. **Start and end** conditions\n\
             2. **Main steps** in order (numbered list)\n\
             3. **Decision branches** (condition → outcome)\n\
             4. **Loops or back-edges** (if any)",
        Diagram =>
            "Describe this technical diagram:\n\
             1. **Main elements** and their roles\n\
             2. **Relationships and connections** between elements\n\
             3. **Directional flow** (if present)\n\
             4. **Key takeaway** in one sentence",
        SystemDemo =>
            "Describe this system demonstration screenshot:\n\
             1. **Pipeline stages** shown (list each labelled section)\n\
             2. **Input** to the system (if visible)\n\
             3. **Output / response** produced\n\
             4. **Key observations** about the system behaviour",
        TableVisual =>
            "Reconstruct this visual table:\n\
             1. **Headers** (column names)\n\
             2. **Rows** (as a Markdown table)\n\
             3. **Notable values** (maxima, minima, highlighted cells)",
        Illustration | Photograph =>
            "Write a descriptive caption:\n\
             1. **Subject** (what is depicted)\n\
             2. **Key visual elements** labelled (if any)\n\
             3. **One-sentence caption** suitable for a figure in a paper",
        // Noise kinds never reach Pass 2 — fallback for robustness
        _ => "Describe the key content of this image in 2–4 sentences.",
    }
}

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

    #[test]
    fn filter_prompts_have_required_sections() {
        assert!(FIGURE_FILTER_PASS1_SYSTEM.contains("is_figure"));
        assert!(FIGURE_FILTER_PASS1_SYSTEM.contains("text_block"));
        assert!(FIGURE_FILTER_PASS2_SYSTEM.contains("Markdown"));
        // Every figure kind must have a non-empty pass-2 prompt
        use crate::figure_filter::FigureKind;
        for kind in &[
            FigureKind::BarChart, FigureKind::Flowchart,
            FigureKind::ArchitectureDiagram, FigureKind::SystemDemo,
        ] {
            let p = figure_filter_pass2_prompt(kind);
            assert!(!p.is_empty(), "empty Pass-2 prompt for {kind:?}");
        }
    }
}
