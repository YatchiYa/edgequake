//! Multimodal VLM / Extract prompt builders (LightRAG `prompt_multimodal.py` parity).
//!
//! SPEC-047 / 015: Chart specialize demands verbatim readable numbers into
//! `key_values` / `series` / optional `data_table_md` for RAG indexing.

use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use edgequake_llm::traits::{ChatMessage, ImageData};

use super::prompt_context::{table_content_format_label, PromptContext};

const IMAGE_ANALYSIS_SYSTEM_PROMPT: &str = "\
You are an expert image analyzer for RAG indexing. Analyze the provided image and return a single JSON object.

Use Additional Context (Captions, Footnotes, Leading/Trailing Text) only to disambiguate — the image itself takes priority.
Prefer type=Chart when the image is primarily a data plot (bars, lines, pie, scatter, area) even if the caption says \"Figure\".
Multi-panel grids of line/bar charts are Chart, not Illustration.
Prefer type=Illustration/Flowchart for diagrams without quantitative axes.
Return ONLY valid JSON with keys: \"name\" (snake_case), \"type\" (Photo|Illustration|Screenshot|Icon|Chart|Table|Infographic|Flowchart|Chat Log|Wireframe|Texture|Other), \"description\" (markdown, ≤500 words; include any visible numbers verbatim).
Output values for name and description must be in the requested language.";

const CHART_ANALYSIS_SYSTEM_PROMPT: &str = "\
You are an expert chart/data-visualization analyzer for RAG indexing.

Extract ONLY what is visually readable. Never invent, estimate, interpolate, or round from guesswork — omit unreadables.
For multi-panel / grid charts: extract EVERY subplot separately — prefix labels with panel title (e.g. \"Average | full data | 10B=52\").
Fail closed on density: if any number is readable, key_values and/or series and/or data_table_md MUST capture it — prose-only descriptions are incomplete.
Return ONLY valid JSON with keys:
- \"name\" (snake_case)
- \"chart_kind\" (bar|line|pie|scatter|area|stacked|other)
- \"title\" (string, may be empty)
- \"x_axis\" (string label/units, may be empty)
- \"y_axis\" (string label/units, may be empty)
- \"series\" (array of {\"name\": string, \"values\": [{\"x\": string, \"y_raw\": string}]} for every readable point; keep units in y_raw)
- \"key_values\" (array of {\"label\": string, \"value_raw\": string} for EVERY readable number/percentage/callout — densest searchable form; prefer ≥2 entries when multiple numbers are visible)
- \"data_table_md\" (GFM markdown table of the same points when ≥2 readable values exist; may be empty only if zero numbers are readable)
- \"description\" (markdown ≤300 words summarizing trends WITHOUT adding numbers not present in series/key_values/data_table_md)
Output ALL string fields in English (translate labels when needed; keep numeric tokens verbatim).";

const CHART_ANALYSIS_DENSE_RETRY_HINT: &str = "\
RETRY — prior extract was too sparse for RAG. Re-read the image carefully.\n\
REQUIREMENTS (fail closed):\n\
- Emit key_values for EVERY readable number/percentage/callout (label + value_raw).\n\
- Emit series.values for every readable (x, y) point.\n\
- Emit data_table_md as a GFM table covering the same points.\n\
- Do NOT invent values. Do NOT return prose-only description.\n\
- English only for string fields.";

const FIGURE_ANALYSIS_SYSTEM_PROMPT: &str = "\
You are an expert technical-figure / diagram analyzer for RAG indexing.

Focus on components, labels, relationships, flow, and any visible numbers — not decorative style.
Never invent labels, connections, or numbers that are not visible.
Return ONLY valid JSON with keys:
- \"name\" (snake_case)
- \"type\" (Illustration|Flowchart|Infographic|Screenshot|Other)
- \"components\" (array of short strings)
- \"relationships\" (array of short strings describing connections)
- \"visible_text\" (array of verbatim labels/numbers/callouts readable on the figure)
- \"description\" (markdown ≤400 words; quote visible labels and numbers verbatim)
Output ALL string fields in English (translate labels when needed; keep numeric tokens verbatim).";

const TABLE_ANALYSIS_SYSTEM_PROMPT: &str = "\
You are an expert table analyzer. Analyze the table content and return a single JSON object.

Use Additional Context only for disambiguation — table content takes priority. Never invent rows or values.
Prefer a markdown table of ALL visible cells with units preserved.
Wide / multi-section tables: include every column that is visible; do not drop unit columns.
Return ONLY valid JSON with keys: \"name\" (snake_case), \"type\" (always \"Table\"), \"description\" (markdown, ≤500 words).
Output ALL string fields in English (translate labels when needed; keep numeric tokens verbatim).";

const EQUATION_ANALYSIS_SYSTEM_PROMPT: &str = "\
You are an expert equation analyzer. Analyze the equation and return a single JSON object.

Use Additional Context only for disambiguation — equation body takes priority.
Return ONLY valid JSON with keys: \"name\" (snake_case), \"equation\" (LaTeX math-mode body, no $ delimiters), \"description\" (≤300 words).
Output values for name and description must be in the requested language.";

/// Image types that warrant a second-pass Chart extract (SPEC-047 MV Phase B/D).
pub fn is_chart_like_type(image_type: &str) -> bool {
    matches!(image_type.trim(), "Chart" | "Infographic")
}

/// Image types that warrant a second-pass Figure/diagram extract.
pub fn is_figure_like_type(image_type: &str) -> bool {
    matches!(
        image_type.trim(),
        "Illustration" | "Flowchart" | "Wireframe"
    )
}

/// Caption/context hint that the image is quantitative (route to chart specialize).
///
/// DRY: delegates to [`edgequake_pdf::text_suggests_chart`] (MV-24 SSOT).
pub fn context_suggests_chart(ctx: &PromptContext) -> bool {
    let blob = format!(
        "{} {} {} {}",
        ctx.captions, ctx.footnotes, ctx.leading, ctx.trailing
    );
    edgequake_pdf::text_suggests_chart(&blob)
}

/// Whether to run chart specialize after classify (type or context).
pub fn should_specialize_as_chart(image_type: &str, ctx: &PromptContext) -> bool {
    is_chart_like_type(image_type)
        || (context_suggests_chart(ctx)
            && !matches!(
                image_type.trim(),
                "Photo" | "Icon" | "Texture" | "Chat Log" | "Table"
            ))
}

/// Build initial VLM messages for image analysis with LightRAG context block.
pub fn image_analysis_messages(
    image_bytes: &[u8],
    mime_type: &str,
    ctx: &PromptContext,
) -> Vec<ChatMessage> {
    let base64_data = B64.encode(image_bytes);
    let image_data = ImageData::new(&base64_data, mime_type);

    let user_text = format!(
        "Analyze this image and return the JSON object.\n\
         Language: {}\n\n{}\n\nOutput:",
        ctx.language,
        ctx.additional_context_block()
    );

    vec![
        ChatMessage::system(IMAGE_ANALYSIS_SYSTEM_PROMPT),
        ChatMessage::user_with_images(user_text, vec![image_data]),
    ]
}

/// Second-pass chart extract (axes / key_values) after classify.
pub fn chart_analysis_messages(
    image_bytes: &[u8],
    mime_type: &str,
    ctx: &PromptContext,
) -> Vec<ChatMessage> {
    chart_analysis_messages_with_density(image_bytes, mime_type, ctx, false)
}

/// Dense retry after a sparse chart extract (SPEC-047 / 026 W1-dense-B).
///
/// Distinct user text ⇒ distinct analysis-cache fingerprint (OCP: no cache collision).
pub fn chart_analysis_messages_dense(
    image_bytes: &[u8],
    mime_type: &str,
    ctx: &PromptContext,
) -> Vec<ChatMessage> {
    chart_analysis_messages_with_density(image_bytes, mime_type, ctx, true)
}

fn chart_analysis_messages_with_density(
    image_bytes: &[u8],
    mime_type: &str,
    ctx: &PromptContext,
    dense_retry: bool,
) -> Vec<ChatMessage> {
    let base64_data = B64.encode(image_bytes);
    let image_data = ImageData::new(&base64_data, mime_type);
    let density_block = if dense_retry {
        format!("\n\n{CHART_ANALYSIS_DENSE_RETRY_HINT}\n")
    } else {
        String::new()
    };
    let user_text = format!(
        "Extract structured chart data as JSON.\n\
         Prefer key_values + series covering EVERY readable number; omit unreadables.\n\
         For grid/multi-panel charts: one entry per subplot × series × x-point.\n\
         Always emit data_table_md (GFM) when ≥2 numbers are readable.\n\
         Language: English (Acc pin — ignore other page languages for string fields).\n\
         {}\n{}\n\nOutput:",
        density_block,
        ctx.additional_context_block()
    );
    vec![
        ChatMessage::system(CHART_ANALYSIS_SYSTEM_PROMPT),
        ChatMessage::user_with_images(user_text, vec![image_data]),
    ]
}

/// Second-pass figure/diagram extract after classify.
pub fn figure_analysis_messages(
    image_bytes: &[u8],
    mime_type: &str,
    ctx: &PromptContext,
) -> Vec<ChatMessage> {
    let base64_data = B64.encode(image_bytes);
    let image_data = ImageData::new(&base64_data, mime_type);
    let user_text = format!(
        "Extract structured figure/diagram content as JSON.\n\
         Include visible_text with every readable label and number.\n\
         Language: {}\n\n{}\n\nOutput:",
        ctx.language,
        ctx.additional_context_block()
    );
    vec![
        ChatMessage::system(FIGURE_ANALYSIS_SYSTEM_PROMPT),
        ChatMessage::user_with_images(user_text, vec![image_data]),
    ]
}

/// Extract-role messages for HTML/JSON table analysis.
pub fn table_analysis_messages(
    table_body: &str,
    format: &str,
    ctx: &PromptContext,
) -> Result<Vec<ChatMessage>, String> {
    let format_label = table_content_format_label(format)?;
    let user_text = format!(
        "Analyze this table and return the JSON object.\n\
         Language: {}\n\n\
         ================ TABLE CONTENT ================\n\
         The TABLE CONTENT below is in {format_label}.\n\
         ```\n{table_body}\n```\n\n\
         {}\n\nOutput:",
        ctx.language,
        ctx.additional_context_block()
    );
    Ok(vec![
        ChatMessage::system(TABLE_ANALYSIS_SYSTEM_PROMPT),
        ChatMessage::user(user_text),
    ])
}

/// Extract-role messages for equation analysis.
pub fn equation_analysis_messages(equation_body: &str, ctx: &PromptContext) -> Vec<ChatMessage> {
    let user_text = format!(
        "Analyze this equation and return the JSON object.\n\
         Language: {}\n\n\
         ================ EQUATION BODY ================\n\
         ```\n{equation_body}\n```\n\n\
         {}\n\nOutput:",
        ctx.language,
        ctx.additional_context_block()
    );
    vec![
        ChatMessage::system(EQUATION_ANALYSIS_SYSTEM_PROMPT),
        ChatMessage::user(user_text),
    ]
}

/// Fingerprint text for analysis cache hashing (LightRAG args_hash inputs).
pub fn prompt_cache_fingerprint(messages: &[ChatMessage]) -> String {
    use edgequake_llm::traits::ChatRole;
    messages
        .iter()
        .map(|m| {
            let role = match m.role {
                ChatRole::System => "system",
                ChatRole::User => "user",
                ChatRole::Assistant => "assistant",
                ChatRole::Tool => "tool",
                ChatRole::Function => "function",
            };
            format!("{role}:{}", m.content)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Repair-turn user message after invalid JSON.
pub fn json_repair_user_message(invalid_response: &str) -> String {
    format!("Previous invalid response:\n{invalid_response}\n\nReturn corrected JSON only.")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn image_prompt_includes_caption_from_context() {
        let ctx = PromptContext {
            language: "English".into(),
            captions: "Quarterly revenue chart".into(),
            footnotes: "n/a".into(),
            leading: "See figure below".into(),
            trailing: "n/a".into(),
        };
        let msgs = image_analysis_messages(&[0u8; 8], "image/png", &ctx);
        let user = msgs[1].content.as_str();
        assert!(user.contains("Quarterly revenue chart"));
        assert!(user.contains("See figure below"));
    }

    #[test]
    fn table_prompt_includes_format_label() {
        let ctx = PromptContext {
            language: "English".into(),
            captions: "n/a".into(),
            footnotes: "n/a".into(),
            leading: "n/a".into(),
            trailing: "n/a".into(),
        };
        let msgs = table_analysis_messages("<tr><td>A</td></tr>", "html", &ctx).unwrap();
        assert!(msgs[1].content.contains("HTML format"));
    }

    #[test]
    fn chart_and_figure_route_helpers() {
        assert!(is_chart_like_type("Chart"));
        assert!(is_chart_like_type("Infographic"));
        assert!(!is_chart_like_type("Photo"));
        assert!(is_figure_like_type("Flowchart"));
        assert!(is_figure_like_type("Illustration"));
        assert!(!is_figure_like_type("Chart"));
    }

    #[test]
    fn context_routes_math_figure_caption_to_chart_specialize() {
        let ctx = PromptContext {
            language: "English".into(),
            captions: "Figure 1. Model performance across capability dimensions. 10T-token corpus."
                .into(),
            footnotes: "n/a".into(),
            leading: "n/a".into(),
            trailing: "n/a".into(),
        };
        assert!(context_suggests_chart(&ctx));
        assert!(should_specialize_as_chart("Illustration", &ctx));
    }

    #[test]
    fn context_routes_figure_caption_to_chart_specialize() {
        let ctx = PromptContext {
            language: "English".into(),
            captions: "Figure 3. Revenue by quarter (%)".into(),
            footnotes: "n/a".into(),
            leading: "n/a".into(),
            trailing: "n/a".into(),
        };
        assert!(context_suggests_chart(&ctx));
        assert!(should_specialize_as_chart("Illustration", &ctx));
        assert!(!should_specialize_as_chart("Photo", &ctx));
    }

    #[test]
    fn chart_prompt_mentions_key_values_and_series() {
        let ctx = PromptContext {
            language: "English".into(),
            captions: "n/a".into(),
            footnotes: "n/a".into(),
            leading: "n/a".into(),
            trailing: "n/a".into(),
        };
        let msgs = chart_analysis_messages(&[0u8; 4], "image/png", &ctx);
        let system = msgs[0].content.as_str();
        assert!(system.contains("key_values"));
        assert!(system.contains("series"));
        assert!(system.contains("data_table_md"));
        assert!(system.contains("Never invent"));
        assert!(system.contains("Fail closed on density"));
        assert!(system.contains("Output ALL string fields in English"));
        assert!(msgs[1].content.contains("Language: English"));
    }

    #[test]
    fn chart_dense_retry_prompt_is_distinct() {
        let ctx = PromptContext {
            language: "English".into(),
            captions: "n/a".into(),
            footnotes: "n/a".into(),
            leading: "n/a".into(),
            trailing: "n/a".into(),
        };
        let base = chart_analysis_messages(&[0u8; 4], "image/png", &ctx);
        let dense = chart_analysis_messages_dense(&[0u8; 4], "image/png", &ctx);
        assert!(dense[1].content.contains("RETRY"));
        assert!(dense[1].content.contains("fail closed"));
        assert_ne!(base[1].content, dense[1].content);
    }

    #[test]
    fn figure_prompt_mentions_visible_text() {
        let ctx = PromptContext {
            language: "English".into(),
            captions: "n/a".into(),
            footnotes: "n/a".into(),
            leading: "n/a".into(),
            trailing: "n/a".into(),
        };
        let msgs = figure_analysis_messages(&[0u8; 4], "image/png", &ctx);
        assert!(msgs[0].content.contains("components"));
        assert!(msgs[0].content.contains("visible_text"));
    }
}
