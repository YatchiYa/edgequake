//! Second-pass chart/figure specialization after image classify (SPEC-047 Phase B / 015).
//!
//! SRP: [`super::prompts`] owns strings + routing predicates; this module owns
//! parse → searchable markdown merge (numbers land in index text).
//! MV-27: on specialize failure, prefer Pass A numeric dump from context over weak classify.

use std::sync::Arc;

use edgequake_llm::traits::LLMProvider;
use edgequake_storage::traits::KVStorage;
use serde::Deserialize;
use tracing::{info, warn};

use super::super::vision_content::ImageAnalysisResult;
use super::cache::chat_json_with_analysis_cache;
use super::json_recovery::parse_json_object;
use super::prompt_context::PromptContext;
use super::prompts::{
    chart_analysis_messages, figure_analysis_messages, is_figure_like_type,
    json_repair_user_message, should_specialize_as_chart,
};

#[derive(Debug, Deserialize)]
struct ChartKeyValue {
    #[serde(default)]
    label: String,
    #[serde(default)]
    value_raw: String,
}

#[derive(Debug, Deserialize)]
struct ChartSeriesPoint {
    #[serde(default)]
    x: Option<String>,
    #[serde(default)]
    y_raw: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ChartSeries {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    values: Vec<ChartSeriesPoint>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ChartAnalysisResult {
    #[serde(default)]
    name: String,
    #[serde(default)]
    chart_kind: String,
    #[serde(default)]
    title: String,
    #[serde(default)]
    x_axis: String,
    #[serde(default)]
    y_axis: String,
    #[serde(default)]
    key_values: Vec<ChartKeyValue>,
    #[serde(default)]
    series: Vec<ChartSeries>,
    /// Optional GFM table of the same points (SPEC-047 / 015 denser dump).
    #[serde(default)]
    data_table_md: String,
    #[serde(default)]
    description: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct FigureAnalysisResult {
    #[serde(default)]
    name: String,
    #[serde(default, rename = "type")]
    image_type: String,
    #[serde(default)]
    components: Vec<String>,
    #[serde(default)]
    relationships: Vec<String>,
    /// Verbatim labels/numbers readable on the figure.
    #[serde(default)]
    visible_text: Vec<String>,
    #[serde(default)]
    description: String,
}

/// Convert chart JSON into dense searchable markdown (numbers land in index text).
pub(crate) fn chart_analysis_to_description(c: &ChartAnalysisResult) -> String {
    let mut parts = Vec::new();
    if !c.title.trim().is_empty() {
        parts.push(format!("**Title:** {}", c.title.trim()));
    }
    if !c.chart_kind.trim().is_empty() {
        parts.push(format!("**Chart kind:** {}", c.chart_kind.trim()));
    }
    let axes: Vec<String> = [("X", c.x_axis.trim()), ("Y", c.y_axis.trim())]
        .into_iter()
        .filter(|(_, v)| !v.is_empty())
        .map(|(k, v)| format!("{k}: {v}"))
        .collect();
    if !axes.is_empty() {
        parts.push(format!("**Axes:** {}", axes.join("; ")));
    }
    if !c.key_values.is_empty() {
        let rows: Vec<String> = c
            .key_values
            .iter()
            .filter(|kv| !kv.label.trim().is_empty() || !kv.value_raw.trim().is_empty())
            .map(|kv| format!("- {}: {}", kv.label.trim(), kv.value_raw.trim()))
            .collect();
        if !rows.is_empty() {
            parts.push(format!("**Key values:**\n{}", rows.join("\n")));
        }
    }
    for series in &c.series {
        let sname = series.name.as_deref().unwrap_or("series").trim();
        let pts: Vec<String> = series
            .values
            .iter()
            .filter_map(|p| {
                let y = p.y_raw.as_deref()?.trim();
                if y.is_empty() {
                    return None;
                }
                let x = p.x.as_deref().unwrap_or("").trim();
                if x.is_empty() {
                    Some(y.to_string())
                } else {
                    Some(format!("{x}={y}"))
                }
            })
            .collect();
        if !pts.is_empty() {
            parts.push(format!("**Series {sname}:** {}", pts.join(", ")));
        }
    }
    let table = c.data_table_md.trim();
    if !table.is_empty() {
        parts.push(format!("**Data table:**\n{table}"));
    }
    if !c.description.trim().is_empty() {
        parts.push(c.description.trim().to_string());
    }
    parts.join("\n\n")
}

/// Convert figure JSON into searchable markdown (labels + relationships + visible text).
pub(crate) fn figure_analysis_to_description(f: &FigureAnalysisResult) -> String {
    let mut parts = Vec::new();
    if !f.components.is_empty() {
        let comps: Vec<String> = f
            .components
            .iter()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .map(|s| format!("- {s}"))
            .collect();
        if !comps.is_empty() {
            parts.push(format!("**Components:**\n{}", comps.join("\n")));
        }
    }
    if !f.relationships.is_empty() {
        let rels: Vec<String> = f
            .relationships
            .iter()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .map(|s| format!("- {s}"))
            .collect();
        if !rels.is_empty() {
            parts.push(format!("**Relationships:**\n{}", rels.join("\n")));
        }
    }
    if !f.visible_text.is_empty() {
        let texts: Vec<String> = f
            .visible_text
            .iter()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .map(|s| format!("- {s}"))
            .collect();
        if !texts.is_empty() {
            parts.push(format!("**Visible text:**\n{}", texts.join("\n")));
        }
    }
    if !f.description.trim().is_empty() {
        parts.push(f.description.trim().to_string());
    }
    parts.join("\n\n")
}

/// True when description lacks any digit (sparse / prose-only chart extract).
fn description_lacks_numeric_dump(description: &str) -> bool {
    !description.chars().any(|c| c.is_ascii_digit())
}

/// Merge Pass A numeric dumps when chart specialize omitted readable points (MV-27).
fn merge_pass_a_dump_when_sparse(description: &str, ctx: &PromptContext) -> String {
    if !description_lacks_numeric_dump(description) {
        return description.to_string();
    }
    match pass_a_numeric_dump_from_context(ctx) {
        Some(dump) if !dump.trim().is_empty() => {
            if description.trim().is_empty() {
                dump
            } else {
                format!("{dump}\n\n{description}")
            }
        }
        _ => description.to_string(),
    }
}

fn parse_chart_analysis(text: &str) -> Result<ChartAnalysisResult, String> {
    let mut parsed: ChartAnalysisResult = parse_json_object(text)?;
    if parsed.name.trim().is_empty() {
        parsed.name = "chart_content".into();
    }
    if chart_analysis_to_description(&parsed).trim().is_empty() {
        return Err("chart extract produced empty description".into());
    }
    Ok(parsed)
}

fn parse_figure_analysis(text: &str) -> Result<FigureAnalysisResult, String> {
    let mut parsed: FigureAnalysisResult = parse_json_object(text)?;
    if parsed.name.trim().is_empty() {
        parsed.name = "figure_content".into();
    }
    if figure_analysis_to_description(&parsed).trim().is_empty() {
        return Err("figure extract produced empty description".into());
    }
    Ok(parsed)
}

/// Extract Pass A numeric dump fragments from surrounding context (MV-27 soft-fail).
///
/// Prefers GFM tables and Key values / data-table sections already written by Pass A.
pub(crate) fn pass_a_numeric_dump_from_context(ctx: &PromptContext) -> Option<String> {
    let blob = format!("{}\n{}\n{}", ctx.captions, ctx.leading, ctx.trailing);
    let mut parts = Vec::new();
    let lower = blob.to_ascii_lowercase();
    if let Some(idx) = lower.find("**key values:**") {
        let slice = &blob[idx..];
        let end = slice.find("\n\n**").unwrap_or(slice.len().min(800));
        parts.push(slice[..end].trim().to_string());
    }
    if let Some(idx) = lower.find("**data table:**") {
        let slice = &blob[idx..];
        let end = slice.find("\n\n**").unwrap_or(slice.len().min(1200));
        parts.push(slice[..end].trim().to_string());
    }
    if parts.is_empty() {
        let lines: Vec<&str> = blob.lines().collect();
        let mut i = 0;
        while i + 1 < lines.len() {
            if lines[i].contains('|') && lines[i + 1].contains('|') && lines[i + 1].contains('-') {
                let mut table = vec![lines[i].trim()];
                i += 1;
                while i < lines.len() && lines[i].contains('|') {
                    table.push(lines[i].trim());
                    i += 1;
                }
                if table.len() >= 2 {
                    parts.push(format!("**Data table:**\n{}", table.join("\n")));
                }
                continue;
            }
            i += 1;
        }
    }
    let out = parts
        .into_iter()
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n");
    if out.trim().is_empty() {
        None
    } else {
        Some(out)
    }
}

fn soft_fail_chart_result(
    item_id: &str,
    classified: ImageAnalysisResult,
    ctx: &PromptContext,
    err: &str,
) -> ImageAnalysisResult {
    if let Some(dump) = pass_a_numeric_dump_from_context(ctx) {
        info!(
            %item_id,
            error = %err,
            dump_chars = dump.len(),
            "MV-27 chart specialize soft-fail: keeping Pass A numeric dump"
        );
        let mut out = classified;
        out.image_type = "Chart".into();
        if out.name.trim().is_empty() {
            out.name = "chart_content".into();
        }
        let classify_tail = out.description.trim();
        out.description = if classify_tail.is_empty() {
            dump
        } else {
            format!("{dump}\n\n{classify_tail}")
        };
        out
    } else {
        warn!(%item_id, error = %err, "chart specialize failed; keeping classify result");
        classified
    }
}

/// After classify, optionally run a typed second VLM pass and merge into `ImageAnalysisResult`.
///
/// Routing (DRY via prompts predicates):
/// 1. Chart specialize if type is Chart/Infographic **or** caption/context suggests chart
///    (and type is not Photo/Icon/…).
/// 2. Else figure specialize for Illustration/Flowchart/Wireframe.
pub async fn specialize_image_analysis(
    item_id: &str,
    bytes: &[u8],
    mime_type: &str,
    llm: &dyn LLMProvider,
    ctx: &PromptContext,
    kv: Option<Arc<dyn KVStorage>>,
    classified: ImageAnalysisResult,
) -> ImageAnalysisResult {
    if should_specialize_as_chart(&classified.image_type, ctx) {
        // MV-24: ink-crop full-page drawings so chart marks occupy more of the VLM frame.
        let specialize_bytes = edgequake_pdf::maybe_chart_specialize_bytes(bytes);
        let messages = chart_analysis_messages(&specialize_bytes, mime_type, ctx);
        match chat_json_with_analysis_cache(
            llm,
            kv,
            item_id,
            "drawing_chart",
            messages,
            parse_chart_analysis,
            json_repair_user_message,
        )
        .await
        {
            Ok((chart, _)) => {
                let mut out = classified;
                if !chart.name.trim().is_empty() {
                    out.name = chart.name.clone();
                }
                // Ensure chunk renderer uses [Chart Name] even when classify said Illustration.
                out.image_type = "Chart".into();
                out.description =
                    merge_pass_a_dump_when_sparse(&chart_analysis_to_description(&chart), ctx);
                return out;
            }
            Err(e) => {
                return soft_fail_chart_result(item_id, classified, ctx, &e);
            }
        }
    }

    if is_figure_like_type(&classified.image_type) {
        let messages = figure_analysis_messages(bytes, mime_type, ctx);
        match chat_json_with_analysis_cache(
            llm,
            kv,
            item_id,
            "drawing_figure",
            messages,
            parse_figure_analysis,
            json_repair_user_message,
        )
        .await
        {
            Ok((figure, _)) => {
                let mut out = classified;
                if !figure.name.trim().is_empty() {
                    out.name = figure.name.clone();
                }
                if !figure.image_type.trim().is_empty() {
                    let _ = figure.image_type;
                }
                out.description =
                    merge_pass_a_dump_when_sparse(&figure_analysis_to_description(&figure), ctx);
                return out;
            }
            Err(e) => {
                warn!(%item_id, error = %e, "figure specialize failed; keeping classify result");
                return classified;
            }
        }
    }

    classified
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_ctx_with_leading(leading: &str) -> PromptContext {
        PromptContext {
            language: "English".into(),
            captions: "n/a".into(),
            footnotes: "n/a".into(),
            leading: leading.into(),
            trailing: "n/a".into(),
        }
    }

    #[test]
    fn merge_pass_a_when_chart_description_has_no_digits() {
        let ctx = empty_ctx_with_leading(
            "| Panel | Series | Value |\n|---|---|---|\n| Average | full data | 52 |\n| Average | w/o code | 41 |",
        );
        let merged = merge_pass_a_dump_when_sparse("Trend summary only.", &ctx);
        assert!(merged.contains("52"));
        assert!(merged.contains("full data"));
        assert!(merged.contains("Trend summary"));
    }

    #[test]
    fn merge_pass_a_skipped_when_chart_already_has_digits() {
        let ctx = empty_ctx_with_leading("| Q | 99 |");
        let merged = merge_pass_a_dump_when_sparse("Score 42 on axis.", &ctx);
        assert_eq!(merged, "Score 42 on axis.");
    }

    #[test]
    fn chart_description_includes_key_values() {
        let c = ChartAnalysisResult {
            name: "revenue".into(),
            chart_kind: "bar".into(),
            title: "Q4 Revenue".into(),
            x_axis: "Quarter".into(),
            y_axis: "USD M".into(),
            key_values: vec![ChartKeyValue {
                label: "Q4".into(),
                value_raw: "42".into(),
            }],
            series: vec![],
            data_table_md: String::new(),
            description: "Revenue rose.".into(),
        };
        let d = chart_analysis_to_description(&c);
        assert!(d.contains("42"));
        assert!(d.contains("Q4"));
        assert!(d.contains("bar"));
        assert!(d.contains("USD M"));
    }

    #[test]
    fn chart_data_table_md_lands_in_description() {
        let c = ChartAnalysisResult {
            name: "rev".into(),
            chart_kind: "bar".into(),
            title: String::new(),
            x_axis: String::new(),
            y_axis: String::new(),
            key_values: vec![],
            series: vec![],
            data_table_md: "| Q | V |\n|---|---|\n| Q4 | 42 |".into(),
            description: String::new(),
        };
        let d = chart_analysis_to_description(&c);
        assert!(d.contains("| Q4 | 42 |"));
        assert!(d.contains("**Data table:**"));
    }

    #[test]
    fn figure_description_includes_components_and_visible_text() {
        let f = FigureAnalysisResult {
            name: "arch".into(),
            image_type: "Flowchart".into(),
            components: vec!["API".into(), "DB".into()],
            relationships: vec!["API → DB".into()],
            visible_text: vec!["latency ≤ 50ms".into()],
            description: "System flow.".into(),
        };
        let d = figure_analysis_to_description(&f);
        assert!(d.contains("API"));
        assert!(d.contains("API → DB"));
        assert!(d.contains("50ms"));
    }

    #[test]
    fn chart_series_points_land_in_text() {
        let c = ChartAnalysisResult {
            name: "s".into(),
            chart_kind: "line".into(),
            title: String::new(),
            x_axis: String::new(),
            y_axis: String::new(),
            key_values: vec![],
            series: vec![ChartSeries {
                name: Some("A".into()),
                values: vec![
                    ChartSeriesPoint {
                        x: Some("2020".into()),
                        y_raw: Some("10%".into()),
                    },
                    ChartSeriesPoint {
                        x: Some("2021".into()),
                        y_raw: Some("15%".into()),
                    },
                ],
            }],
            data_table_md: String::new(),
            description: String::new(),
        };
        let d = chart_analysis_to_description(&c);
        assert!(d.contains("2020=10%"));
        assert!(d.contains("2021=15%"));
    }

    #[test]
    fn pass_a_dump_extracts_gfm_table() {
        let ctx = empty_ctx_with_leading(
            "Revenue chart\n\n| Category / X | Value |\n|---|---|\n| Q4 | 42 |\n\nMore text",
        );
        let dump = pass_a_numeric_dump_from_context(&ctx).expect("dump");
        assert!(dump.contains("| Q4 | 42 |"));
    }

    #[test]
    fn soft_fail_prefers_pass_a_over_empty_classify() {
        let ctx = empty_ctx_with_leading("**Key values:**\n- Q4: 42\n");
        let classified = ImageAnalysisResult {
            name: "x".into(),
            image_type: "Screenshot".into(),
            description: "A screenshot.".into(),
        };
        let out = soft_fail_chart_result("im-1", classified, &ctx, "parse failed");
        assert_eq!(out.image_type, "Chart");
        assert!(out.description.contains("42"));
    }
}
