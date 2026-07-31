//! Structured ingestion progress counts (SPEC-048 / SPEC-091 LAW-IS1).
//!
//! WHY: Active Runs / document list must project one quantitative authority.
//! Free-text `stage_message` alone forces brittle regex on every reader.
//! Durable `progress_counts: { unit, current, total }` is the SSOT; messages
//! remain human copy. Writers sync counts when they set stage_message;
//! readers prefer structured counts and fall back to message parse once.

use serde_json::{json, Map, Value};

use crate::handlers::ingestion_types::IngestionProgressCounts;

/// Metadata key for structured counts (list / progress / operations).
pub const PROGRESS_COUNTS_KEY: &str = "progress_counts";

/// Serialize counts for JSONB / KV metadata.
pub fn progress_counts_json(counts: &IngestionProgressCounts) -> Value {
    json!({
        "unit": counts.unit,
        "current": counts.current,
        "total": counts.total,
    })
}

/// Insert or overwrite `progress_counts` on a metadata object.
pub fn insert_progress_counts(meta: &mut Map<String, Value>, counts: &IngestionProgressCounts) {
    meta.insert(PROGRESS_COUNTS_KEY.to_string(), progress_counts_json(counts));
}

/// Remove structured counts (e.g. capacity park with no itemized work).
pub fn clear_progress_counts(meta: &mut Map<String, Value>) {
    meta.remove(PROGRESS_COUNTS_KEY);
}

/// Build page counts (PDF convert).
pub fn pages_counts(current: u64, total: u64) -> IngestionProgressCounts {
    IngestionProgressCounts {
        current,
        total,
        unit: "pages".to_string(),
    }
}

/// Build chunk counts (extract / embed).
pub fn chunks_counts(current: u64, total: u64) -> IngestionProgressCounts {
    IngestionProgressCounts {
        current,
        total,
        unit: "chunks".to_string(),
    }
}

/// Build figure/image counts (vision analyze during converting).
pub fn figures_counts(current: u64, total: u64) -> IngestionProgressCounts {
    IngestionProgressCounts {
        current,
        total,
        unit: "figures".to_string(),
    }
}

/// Read structured counts from document metadata (prefer over message regex).
pub fn progress_counts_from_metadata(
    obj: &Map<String, Value>,
) -> Option<IngestionProgressCounts> {
    let c = obj.get(PROGRESS_COUNTS_KEY)?;
    progress_counts_from_value(c)
}

/// Parse a `progress_counts` JSON value (object with unit/current/total).
pub fn progress_counts_from_value(c: &Value) -> Option<IngestionProgressCounts> {
    let current = c.get("current").and_then(|v| {
        v.as_u64()
            .or_else(|| v.as_i64().map(|i| i.max(0) as u64))
            .or_else(|| v.as_f64().map(|f| f.max(0.0) as u64))
    })?;
    let total = c.get("total").and_then(|v| {
        v.as_u64()
            .or_else(|| v.as_i64().map(|i| i.max(0) as u64))
            .or_else(|| v.as_f64().map(|f| f.max(0.0) as u64))
    })?;
    if total == 0 {
        return None;
    }
    let unit = c
        .get("unit")
        .and_then(|v| v.as_str())
        .unwrap_or("chunks")
        .to_string();
    Some(IngestionProgressCounts {
        current,
        total,
        unit,
    })
}

/// Parse `42/351` style counters from free-text stage_message (write-time / fallback).
///
/// Code is law: this lives in one module so facade + writers cannot drift.
pub fn parse_counts_from_message(message: &str) -> Option<IngestionProgressCounts> {
    let lower = message.to_lowercase();
    let unit = if lower.contains("chunk") {
        "chunks"
    } else if lower.contains("figure") || lower.contains("chart") {
        "figures"
    } else if lower.contains("page") {
        "pages"
    } else if lower.contains("relat") {
        "relationships"
    } else if lower.contains("entit") {
        "entities"
    } else {
        "chunks"
    };

    let re = regex::Regex::new(r"(?i)(\d+)\s*/\s*(\d+)").ok()?;
    let caps = re.captures(message)?;
    let current: u64 = caps.get(1)?.as_str().parse().ok()?;
    let total: u64 = caps.get(2)?.as_str().parse().ok()?;
    if total == 0 {
        return None;
    }
    Some(IngestionProgressCounts {
        current,
        total,
        unit: unit.to_string(),
    })
}

/// Resolve counts: structured metadata wins; message parse is fallback only.
pub fn resolve_progress_counts(
    obj: &Map<String, Value>,
    message: &str,
) -> Option<IngestionProgressCounts> {
    progress_counts_from_metadata(obj).or_else(|| parse_counts_from_message(message))
}

/// Sync structured counts from a stage message onto metadata (write path).
///
/// Clears the key when the message has no N/M so stale counts cannot linger
/// across stages (e.g. pages → extracting with no chunk totals yet).
pub fn sync_progress_counts_from_message(meta: &mut Map<String, Value>, message: &str) {
    match parse_counts_from_message(message) {
        Some(counts) => insert_progress_counts(meta, &counts),
        None => clear_progress_counts(meta),
    }
}

/// Apply stage fields + optional structured counts onto a metadata map.
pub fn apply_stage_progress_fields(
    meta: &mut Map<String, Value>,
    current_stage: &str,
    stage_message: &str,
    stage_progress: f64,
    counts: Option<&IngestionProgressCounts>,
) {
    meta.insert("current_stage".to_string(), json!(current_stage));
    meta.insert("stage_message".to_string(), json!(stage_message));
    meta.insert(
        "stage_progress".to_string(),
        json!(stage_progress.clamp(0.0, 1.0)),
    );
    meta.insert(
        "updated_at".to_string(),
        json!(chrono::Utc::now().to_rfc3339()),
    );
    if let Some(c) = counts {
        insert_progress_counts(meta, c);
    } else {
        sync_progress_counts_from_message(meta, stage_message);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_counts_in_metadata() {
        let mut meta = Map::new();
        insert_progress_counts(&mut meta, &chunks_counts(3, 12));
        let got = progress_counts_from_metadata(&meta).unwrap();
        assert_eq!(got.current, 3);
        assert_eq!(got.total, 12);
        assert_eq!(got.unit, "chunks");
    }

    #[test]
    fn zero_total_is_rejected() {
        let mut meta = Map::new();
        insert_progress_counts(&mut meta, &pages_counts(0, 0));
        assert!(progress_counts_from_metadata(&meta).is_none());
    }

    #[test]
    fn resolve_prefers_structured_over_message() {
        let mut meta = Map::new();
        insert_progress_counts(&mut meta, &pages_counts(4, 9));
        let got = resolve_progress_counts(&meta, "chunk 1/99").unwrap();
        assert_eq!(got.unit, "pages");
        assert_eq!(got.current, 4);
        assert_eq!(got.total, 9);
    }

    #[test]
    fn sync_from_message_writes_and_clears() {
        let mut meta = Map::new();
        sync_progress_counts_from_message(
            &mut meta,
            "Converting PDF to Markdown: page 4/9 (44%)",
        );
        let got = progress_counts_from_metadata(&meta).unwrap();
        assert_eq!(got.unit, "pages");
        assert_eq!((got.current, got.total), (4, 9));

        sync_progress_counts_from_message(&mut meta, "Extracting entities…");
        assert!(progress_counts_from_metadata(&meta).is_none());
    }

    #[test]
    fn parse_units_for_page_chunk_figure() {
        assert_eq!(
            parse_counts_from_message("page 2/10").unwrap().unit,
            "pages"
        );
        assert_eq!(
            parse_counts_from_message("chunk 3/12").unwrap().unit,
            "chunks"
        );
        assert_eq!(
            parse_counts_from_message("figure 1/4").unwrap().unit,
            "figures"
        );
    }
}
