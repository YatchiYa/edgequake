//! SSOT for multimodal entity human labels (066 Drawing Entity Naming).
//!
//! Law: **identity ≠ presentation**. Graph node id stays `im-…` / `IM-…`.
//! `display_name` is the UI/RAG surface form.

use regex::Regex;
use std::sync::LazyLock;

static MM_DISPLAY_NAME: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^\[(?:Image|Chart|Figure|Table|Equation) Name\](.+)$")
        .expect("mm display name regex")
});

static PAGE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)-page-(\d{1,6})(?:-fig-(\d{1,4})|-chart)?(?:$|[^-a-z0-9])")
        .expect("drawing page locus regex")
});

static IMAGE_TYPE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^\[Image Type\](.+)$").expect("image type regex"));

static UUID_LIKE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$")
        .expect("uuid-like regex")
});

/// Parse friendly name from mm chunk content (LightRAG `_parse_mm_display_name`).
pub fn parse_mm_display_name(content: &str, fallback: &str) -> String {
    if let Some(cap) = MM_DISPLAY_NAME.captures(content) {
        let candidate = cap.get(1).map(|m| m.as_str().trim()).unwrap_or("");
        if !candidate.is_empty() {
            return candidate.to_string();
        }
    }
    fallback.to_string()
}

/// Kind of multimodal crop inferred from item id.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrawingItemKind {
    Figure,
    Chart,
    Page,
    Unknown,
}

/// Structural locus parsed from a drawing/table/equation item id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DrawingItemLocus {
    pub page: Option<u32>,
    pub fig: Option<u32>,
    pub kind: DrawingItemKind,
    /// Suggested `document_mm_assets.asset_id` stem (no `im-` / doc slug).
    pub asset_id_hint: Option<String>,
}

/// Inputs for resolving a human display label.
#[derive(Debug, Clone, Default)]
pub struct MmDisplayInput<'a> {
    pub item_id: &'a str,
    pub content: &'a str,
    pub heading: Option<&'a str>,
    pub caption: Option<&'a str>,
    pub doc_title: Option<&'a str>,
    pub sidecar_type: &'a str,
}

/// Resolved human label + structural props for a multimodal entity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MmDisplayLabel {
    pub display_name: String,
    pub page: Option<u32>,
    pub fig: Option<u32>,
    pub asset_id_hint: Option<String>,
    pub mm_subtype: Option<String>,
    pub kind: DrawingItemKind,
}

/// Strip workspace scope prefix (`{uuid}::NAME` → `NAME`).
pub fn bare_entity_id(node_or_item_id: &str) -> &str {
    node_or_item_id
        .rsplit_once("::")
        .map(|(_, rest)| rest)
        .unwrap_or(node_or_item_id)
}

/// True when a VLM/caption name is a known placeholder or useless.
pub fn is_placeholder_mm_name(name: &str, item_id: &str) -> bool {
    let t = name.trim();
    if t.is_empty() {
        return true;
    }
    let lower = t.to_ascii_lowercase();
    if matches!(
        lower.as_str(),
        "figure_content"
            | "chart_content"
            | "image_content"
            | "table_content"
            | "equation_content"
            | "drawing"
            | "figure"
            | "chart"
            | "image"
            | "untitled"
    ) {
        return true;
    }
    let bare = bare_entity_id(item_id);
    if t.eq_ignore_ascii_case(bare) || t.eq_ignore_ascii_case(item_id) {
        return true;
    }
    // Opaque im- identities are never good display names.
    let bare_l = bare.to_ascii_lowercase();
    bare_l.starts_with("im-") && t.eq_ignore_ascii_case(bare)
}

/// Parse page/fig/kind from `im-…-page-NNNN-fig-MM` / `-chart` (case-insensitive).
pub fn parse_drawing_item_locus(item_id: &str) -> DrawingItemLocus {
    let bare = bare_entity_id(item_id);
    let lower = bare.to_ascii_lowercase();

    let kind = if lower.contains("-fig-") {
        DrawingItemKind::Figure
    } else if lower.ends_with("-chart") || lower.contains("-chart.") {
        DrawingItemKind::Chart
    } else if lower.contains("-page-") {
        DrawingItemKind::Page
    } else {
        DrawingItemKind::Unknown
    };

    let (page, fig) = PAGE_RE
        .captures(&lower)
        .map(|c| {
            let page = c.get(1).and_then(|m| m.as_str().parse::<u32>().ok());
            let fig = c.get(2).and_then(|m| m.as_str().parse::<u32>().ok());
            (page, fig)
        })
        .unwrap_or((None, None));

    let asset_id_hint = asset_id_hint_from_item_id(&lower, kind, page, fig);

    DrawingItemLocus {
        page,
        fig,
        kind,
        asset_id_hint,
    }
}

fn asset_id_hint_from_item_id(
    lower_id: &str,
    kind: DrawingItemKind,
    page: Option<u32>,
    fig: Option<u32>,
) -> Option<String> {
    let page = page?;
    match kind {
        DrawingItemKind::Figure => {
            let fig = fig.unwrap_or(1);
            Some(format!("page-{page:04}-fig-{fig:02}"))
        }
        DrawingItemKind::Chart => Some(format!("page-{page:04}-chart")),
        DrawingItemKind::Page => Some(format!("page-{page:04}")),
        DrawingItemKind::Unknown => {
            // Fallback: strip leading im-{slug}- if present.
            let stripped = lower_id.strip_prefix("im-").unwrap_or(lower_id);
            // Drop uuid slug before first -page-
            if let Some(idx) = stripped.find("-page-") {
                Some(stripped[idx + 1..].to_string()) // page-NNNN...
            } else {
                None
            }
        }
    }
}

/// Short document title for display prefix; skips UUID-like / empty stems.
pub fn doc_short_title(raw: Option<&str>) -> Option<String> {
    let raw = raw?.trim();
    if raw.is_empty() {
        return None;
    }
    // File path → stem
    let stem = std::path::Path::new(raw)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(raw)
        .trim();
    if stem.is_empty() || UUID_LIKE.is_match(stem) {
        return None;
    }
    // Also reject im-prefixed opaque stems
    if stem.to_ascii_lowercase().starts_with("im-") {
        return None;
    }
    let mut s = stem.replace('_', " ");
    if s.chars().count() > 40 {
        s = s.chars().take(40).collect::<String>().trim().to_string();
        s.push('…');
    }
    Some(s)
}

fn parse_mm_subtype(content: &str) -> Option<String> {
    IMAGE_TYPE_RE
        .captures(content)
        .and_then(|c| c.get(1).map(|m| m.as_str().trim().to_string()))
        .filter(|s| !s.is_empty())
}

fn structural_label(locus: &DrawingItemLocus, sidecar_type: &str) -> String {
    let type_word = match sidecar_type.to_ascii_lowercase().as_str() {
        "table" => "Table",
        "equation" => "Equation",
        _ => match locus.kind {
            DrawingItemKind::Chart => "Chart",
            DrawingItemKind::Figure => "Fig",
            DrawingItemKind::Page => "Page",
            DrawingItemKind::Unknown => "Drawing",
        },
    };
    match (locus.fig, locus.page) {
        (Some(fig), Some(page)) => format!("{type_word} {fig} · p.{page}"),
        (None, Some(page)) => format!("{type_word} · p.{page}"),
        (Some(fig), None) => format!("{type_word} {fig}"),
        (None, None) => type_word.to_string(),
    }
}

/// Resolve human `display_name` + structural props (066 fallback chain).
pub fn resolve_mm_entity_display(input: MmDisplayInput<'_>) -> MmDisplayLabel {
    let locus = parse_drawing_item_locus(input.item_id);
    let mm_subtype = parse_mm_subtype(input.content);

    let vlm = parse_mm_display_name(input.content, "");
    let semantic = if !vlm.is_empty() && !is_placeholder_mm_name(&vlm, input.item_id) {
        Some(vlm)
    } else if let Some(cap) = input
        .caption
        .map(str::trim)
        .filter(|s| !s.is_empty() && !is_placeholder_mm_name(s, input.item_id))
    {
        Some(cap.to_string())
    } else if let Some(h) = input
        .heading
        .map(str::trim)
        .filter(|s| !s.is_empty() && !is_placeholder_mm_name(s, input.item_id))
    {
        // Strip leading "## Figure N:" noise lightly
        let h = h.trim_start_matches('#').trim();
        Some(h.to_string())
    } else {
        None
    };

    let base = if let Some(name) = semantic {
        match (locus.page, locus.fig) {
            (Some(page), Some(fig)) => format!("{name} · p.{page} · Fig {fig}"),
            (Some(page), None) => format!("{name} · p.{page}"),
            (None, Some(fig)) => format!("{name} · Fig {fig}"),
            (None, None) => name,
        }
    } else {
        structural_label(&locus, input.sidecar_type)
    };

    let display_name = if let Some(doc) = doc_short_title(input.doc_title) {
        // Avoid "Doc · Doc" duplication when base already starts with doc
        if base
            .to_ascii_lowercase()
            .starts_with(&doc.to_ascii_lowercase())
        {
            base
        } else {
            format!("{doc} · {base}")
        }
    } else {
        base
    };

    MmDisplayLabel {
        display_name,
        page: locus.page,
        fig: locus.fig,
        asset_id_hint: locus.asset_id_hint,
        mm_subtype,
        kind: locus.kind,
    }
}

/// Lazy read-path: resolve display from stored description + node id (existing graphs).
pub fn resolve_mm_display_from_node_props(
    node_id: &str,
    description: Option<&str>,
    entity_type: Option<&str>,
    existing_display_name: Option<&str>,
) -> String {
    if let Some(d) = existing_display_name
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        if !is_placeholder_mm_name(d, node_id) && !bare_entity_id(d).eq_ignore_ascii_case("im-") {
            let bare = bare_entity_id(node_id);
            if !d.eq_ignore_ascii_case(bare) && !d.eq_ignore_ascii_case(node_id) {
                return d.to_string();
            }
        }
    }
    let et = entity_type.unwrap_or("drawing");
    if !matches!(
        et.to_ascii_lowercase().as_str(),
        "drawing" | "table" | "equation"
    ) {
        // Non-mm: prefer bare id without workspace scope for label
        return bare_entity_id(node_id).to_string();
    }
    let label = resolve_mm_entity_display(MmDisplayInput {
        item_id: node_id,
        content: description.unwrap_or(""),
        heading: None,
        caption: None,
        doc_title: None,
        sidecar_type: et,
    });
    label.display_name
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locus_parses_figure_with_doc_slug() {
        let id = "im-019f7028-d3e3-7684-8b3b-a9259368329a-page-0002-fig-01";
        let loc = parse_drawing_item_locus(id);
        assert_eq!(loc.page, Some(2));
        assert_eq!(loc.fig, Some(1));
        assert_eq!(loc.kind, DrawingItemKind::Figure);
        assert_eq!(loc.asset_id_hint.as_deref(), Some("page-0002-fig-01"));
    }

    #[test]
    fn locus_parses_chart_and_scoped_id() {
        let id = "00000000-0000-0000-0000-000000000003::IM-PAGE-0012-CHART";
        let loc = parse_drawing_item_locus(id);
        assert_eq!(loc.page, Some(12));
        assert_eq!(loc.kind, DrawingItemKind::Chart);
        assert_eq!(loc.asset_id_hint.as_deref(), Some("page-0012-chart"));
    }

    #[test]
    fn placeholder_names_rejected() {
        assert!(is_placeholder_mm_name(
            "figure_content",
            "im-page-0001-fig-01"
        ));
        assert!(is_placeholder_mm_name("", "x"));
        assert!(is_placeholder_mm_name(
            "IM-PAGE-0001-FIG-01",
            "im-page-0001-fig-01"
        ));
        assert!(!is_placeholder_mm_name(
            "Architecture overview",
            "im-page-0001-fig-01"
        ));
    }

    #[test]
    fn doc_short_skips_uuid() {
        assert!(doc_short_title(Some("019f7028-d3e3-7684-8b3b-a9259368329a")).is_none());
        assert_eq!(
            doc_short_title(Some("AI_Singapore_Conference.pdf")).as_deref(),
            Some("AI Singapore Conference")
        );
    }

    #[test]
    fn resolve_prefers_vlm_name() {
        let label = resolve_mm_entity_display(MmDisplayInput {
            item_id: "im-doc-page-0002-fig-01",
            content: "[Figure Name]Architecture overview\n[Image Type]Flowchart\n\nbody",
            heading: Some("## Figure 1"),
            caption: None,
            doc_title: Some("AI Singapore Conference.pdf"),
            sidecar_type: "drawing",
        });
        assert!(label.display_name.contains("Architecture overview"));
        assert!(label.display_name.contains("p.2"));
        assert!(label.display_name.starts_with("AI Singapore Conference"));
        assert_eq!(label.page, Some(2));
        assert_eq!(label.fig, Some(1));
        assert_eq!(label.mm_subtype.as_deref(), Some("Flowchart"));
    }

    #[test]
    fn resolve_skips_placeholder_falls_to_structural() {
        let label = resolve_mm_entity_display(MmDisplayInput {
            item_id: "im-019f7028-d3e3-7684-8b3b-a9259368329a-page-0003-fig-02",
            content: "[Figure Name]figure_content\n[Image Type]Illustration\n\nbody",
            heading: None,
            caption: None,
            doc_title: Some("019f7028-d3e3-7684-8b3b-a9259368329a"),
            sidecar_type: "drawing",
        });
        assert_eq!(label.display_name, "Fig 2 · p.3");
        assert!(doc_short_title(Some("019f7028-d3e3-7684-8b3b-a9259368329a")).is_none());
    }

    #[test]
    fn resolve_uses_heading_when_no_vlm() {
        let label = resolve_mm_entity_display(MmDisplayInput {
            item_id: "im-page-0005-fig-01",
            content: "no marker here\n\nbody",
            heading: Some("## Figure 1: Revenue by region"),
            caption: None,
            doc_title: None,
            sidecar_type: "drawing",
        });
        assert!(label.display_name.contains("Revenue by region"));
    }

    #[test]
    fn lazy_read_path_from_description() {
        let name = resolve_mm_display_from_node_props(
            "ws::IM-PAGE-0001-FIG-01",
            Some("[Figure Name]System map\n[Image Type]Diagram\n\nx"),
            Some("drawing"),
            None,
        );
        assert!(name.contains("System map"));
        assert!(name.contains("p.1"));
    }

    #[test]
    fn lazy_prefers_existing_display_name() {
        let name = resolve_mm_display_from_node_props(
            "IM-PAGE-0001-FIG-01",
            Some("[Figure Name]Other\n"),
            Some("drawing"),
            Some("Cached nice name · p.1 · Fig 1"),
        );
        assert_eq!(name, "Cached nice name · p.1 · Fig 1");
    }
}
