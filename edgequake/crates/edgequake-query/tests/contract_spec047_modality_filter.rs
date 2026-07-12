//! SPEC-047 MV-32 — chart modality filter contract tests.

use edgequake_query::{query_prefers_chart_modality, with_chart_modality_filter, MODALITY_CHART};
use edgequake_storage::MetadataFilter;

#[test]
fn chart_query_heuristic_and_filter_contract() {
    assert!(query_prefers_chart_modality("What is Q4 revenue?"));
    let mf = with_chart_modality_filter(MetadataFilter::from_tenant_workspace_type(
        None, None, "chunk",
    ));
    assert_eq!(mf.modalities, Some(vec![MODALITY_CHART.to_string()]));
    assert!(mf.matches(&serde_json::json!({
        "type": "chunk",
        "modality": "chart"
    })));
    assert!(!mf.matches(&serde_json::json!({
        "type": "chunk",
        "modality": "figure"
    })));
}
