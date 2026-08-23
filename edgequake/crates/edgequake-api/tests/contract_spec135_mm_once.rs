//! SPEC-135 U-135-MM-ONCE — inline asset is not re-appended as a sidecar.

use sha2::{Digest, Sha256};

use edgequake_api::services::{
    append_mm_chunks_to_text, filter_mm_chunks_already_inlined, mm_asset_already_inlined,
    MultimodalChunk,
};

const SHA_MM_ONCE: &str = "322742ae94d56a3c0d712c40b5a9b05146472fca31c3a1366190aece29f89a1c";
const ASSET: &str = "cost_capability_synthetic_a";
const CHART_NEEDLE: &str = "[Chart Name]cost_capability_synthetic_a";

fn fixtures_dir() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../specs/135-chunking/fixtures")
}

fn sidecar_chunk(id: &str) -> MultimodalChunk {
    serde_json::from_value(serde_json::json!({
        "item_id": id,
        "modality": "drawing",
        "text": format!("[Chart Name]{id}\n[Image Type]Chart\nDuplicate sidecar."),
        "sidecar": {
            "type": "drawing",
            "id": id,
            "refs": [{ "type": "drawing", "id": id }]
        },
        "chunk_order_index": 0,
        "page_start": 1
    }))
    .expect("multimodal chunk")
}

#[test]
fn u135_mm_once_skips_inlined_sidecar() {
    let path = fixtures_dir().join("mm_once.md");
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let got = Sha256::digest(&bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    assert_eq!(got, SHA_MM_ONCE, "fixture mm_once.md SHA-256 mismatch");
    let body = String::from_utf8(bytes).expect("utf-8");

    assert!(mm_asset_already_inlined(&body, ASSET));
    let sidecar = sidecar_chunk(ASSET);
    let kept = filter_mm_chunks_already_inlined(&body, std::slice::from_ref(&sidecar));
    assert!(kept.is_empty(), "inlined asset must not remain as leftover sidecar");

    let inline_only = body
        .split("<!-- multimodal-chunks -->")
        .next()
        .unwrap()
        .trim_end();
    let out = append_mm_chunks_to_text(inline_only, &[]);
    assert_eq!(out, inline_only);
    assert!(!out.contains("<!-- multimodal-chunks -->"));

    let chart_hits = body.matches(CHART_NEEDLE).count();
    assert!(
        chart_hits <= 1,
        "U-135-MM-ONCE expected at most one {CHART_NEEDLE}, got {chart_hits}"
    );
    let after = append_mm_chunks_to_text(inline_only, &[]);
    assert_eq!(
        after.matches(CHART_NEEDLE).count(),
        0,
        "append of empty leftover must not introduce a Chart Name sidecar"
    );
}
