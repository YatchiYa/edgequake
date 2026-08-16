//! SPEC-049 contract — two-pass VLM figure filter (figure_filter.rs).
//!
//! Tests the filter logic end-to-end using MockProvider without any real LLM
//! or PDF file I/O:
//!   • Pass-1 correctly classifies and gates noise kinds.
//!   • Pass-2 is invoked only for kept (is_figure=true) crops.
//!   • Manifest is written and read back correctly.
//!   • Edge cases: all-noise input, all-kept input, JSON parse robustness.
//!
//! Live LLM test (gated on OPENAI_API_KEY) uses a real arXiv figure PNG.

mod common;

use std::path::PathBuf;
use std::sync::Arc;

use edgequake_llm::MockProvider;
use edgequake_pdf::figure_filter::{
    FigureCandidate, FigureFilter, FigureKind, FIGURE_FILTER_MANIFEST,
};
use edgequake_pdf::{load_manifest, write_manifest, FigureFilterResult};

// ── Fixture PNG (1×1 white pixel — valid PNG, minimal bytes) ─────────────────

const TINY_PNG: &[u8] = &[
    0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, // signature
    0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44, 0x52, // IHDR length + type
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, // 1×1
    0x08, 0x02, 0x00, 0x00, 0x00, 0x90, 0x77, 0x53, 0xde, // bit depth, colour, crc
    0x00, 0x00, 0x00, 0x0c, 0x49, 0x44, 0x41, 0x54, // IDAT length + type
    0x08, 0xd7, 0x63, 0xf8, 0xcf, 0xc0, 0x00, 0x00, 0x00, 0x02, 0x00, 0x01, 0xe2, 0x21, 0xbc,
    0x33, // data + crc
    0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82, // IEND
];

fn write_png(dir: &tempfile::TempDir, name: &str) -> PathBuf {
    let path = dir.path().join(name);
    std::fs::write(&path, TINY_PNG).unwrap();
    path
}

fn candidate(rel: &str, full: PathBuf, page: usize) -> FigureCandidate {
    FigureCandidate {
        rel_path: rel.to_string(),
        full_path: full,
        page_num: page,
        label: "Figure".to_string(),
        area_px: 100,
    }
}

// ── Contract: Pass-1 discards noise, Pass-2 runs only for kept ───────────────

#[tokio::test]
async fn contract_spec049_filter_discards_text_block() {
    let tmp = tempfile::tempdir().unwrap();
    let png = write_png(&tmp, "p01.png");
    let mock = Arc::new(MockProvider::new());

    // Pass-1 response: text_block (noise → discard)
    mock.add_response(r#"{"kind":"text_block","is_figure":false,"confidence":0.99}"#)
        .await;
    // Pass-2 must NOT be called — no response queued

    let filter = FigureFilter::new(Arc::clone(&mock) as Arc<dyn edgequake_llm::LLMProvider>);
    let results = filter
        .run(&[candidate("assets/p01.png", png, 1)])
        .await
        .unwrap();

    assert_eq!(results.len(), 1);
    assert!(!results[0].is_figure, "text_block should be discarded");
    assert_eq!(results[0].kind, FigureKind::TextBlock);
    assert!(results[0].description.is_empty(), "no Pass-2 for noise");
}

#[tokio::test]
async fn contract_spec049_filter_keeps_chart_and_describes() {
    let tmp = tempfile::tempdir().unwrap();
    let png = write_png(&tmp, "p02.png");
    let mock = Arc::new(MockProvider::new());

    // Pass-1: bar_chart (kept)
    mock.add_response(r#"{"kind":"bar_chart","is_figure":true,"confidence":0.98}"#)
        .await;
    // Pass-2: structured description
    mock.add_response("## Bar Chart\n\n| X | Value |\n|---|-------|\n| A | 10 |\n")
        .await;

    let filter = FigureFilter::new(Arc::clone(&mock) as Arc<dyn edgequake_llm::LLMProvider>);
    let results = filter
        .run(&[candidate("assets/p02.png", png, 2)])
        .await
        .unwrap();

    assert_eq!(results.len(), 1);
    assert!(results[0].is_figure);
    assert_eq!(results[0].kind, FigureKind::BarChart);
    assert!(
        !results[0].description.is_empty(),
        "Pass-2 description required"
    );
    assert!(results[0].description.contains("Bar Chart"));
}

#[tokio::test]
async fn contract_spec049_filter_mixed_batch() {
    let tmp = tempfile::tempdir().unwrap();
    let png_chart = write_png(&tmp, "chart.png");
    let png_logo = write_png(&tmp, "logo.png");
    let png_diag = write_png(&tmp, "diag.png");
    let mock = Arc::new(MockProvider::new());

    // Filter processes each candidate sequentially: Pass-1 then Pass-2 per crop.
    // Queue responses in that exact consumption order.
    mock.add_response(r#"{"kind":"bar_chart","is_figure":true}"#)
        .await; // P1 chart
    mock.add_response("Chart description").await; // P2 chart
    mock.add_response(r#"{"kind":"logo","is_figure":false}"#)
        .await; // P1 logo (no P2)
    mock.add_response(r#"{"kind":"architecture_diagram","is_figure":true}"#)
        .await; // P1 diag
    mock.add_response("Diagram description").await; // P2 diag

    let filter = FigureFilter::with_limits(
        Arc::clone(&mock) as Arc<dyn edgequake_llm::LLMProvider>,
        1,
        12,
    );
    let results = filter
        .run(&[
            candidate("assets/chart.png", png_chart, 1),
            candidate("assets/logo.png", png_logo, 1),
            candidate("assets/diag.png", png_diag, 2),
        ])
        .await
        .unwrap();

    assert_eq!(results.len(), 3);
    let kept: Vec<_> = results.iter().filter(|r| r.is_figure).collect();
    let discarded: Vec<_> = results.iter().filter(|r| !r.is_figure).collect();
    assert_eq!(kept.len(), 2, "chart + diagram kept");
    assert_eq!(discarded.len(), 1, "logo discarded");
    assert_eq!(discarded[0].kind, FigureKind::Logo);
    assert!(kept.iter().all(|r| !r.description.is_empty()));
}

// ── Contract: manifest I/O ────────────────────────────────────────────────────

#[test]
fn contract_spec049_manifest_round_trip() {
    let tmp = tempfile::tempdir().unwrap();
    let results = vec![
        FigureFilterResult {
            rel_path: "assets/p01-fig-01.png".into(),
            page_num: 1,
            label: "Figure 1".into(),
            kind: FigureKind::BarChart,
            is_figure: true,
            description: "A bar chart showing results.".into(),
        },
        FigureFilterResult {
            rel_path: "assets/p01-fig-02.png".into(),
            page_num: 1,
            label: "Figure".into(),
            kind: FigureKind::Logo,
            is_figure: false,
            description: String::new(),
        },
    ];

    write_manifest(tmp.path(), &results).unwrap();

    // Verify file exists
    assert!(tmp.path().join(FIGURE_FILTER_MANIFEST).exists());

    // Round-trip
    let loaded = load_manifest(tmp.path());
    assert_eq!(loaded.len(), 2);
    assert_eq!(loaded[0].rel_path, "assets/p01-fig-01.png");
    assert_eq!(loaded[0].kind, FigureKind::BarChart);
    assert!(loaded[0].is_figure);
    assert!(!loaded[1].is_figure);
    assert_eq!(loaded[1].kind, FigureKind::Logo);
}

#[test]
fn contract_spec049_missing_manifest_returns_empty() {
    let tmp = tempfile::tempdir().unwrap();
    let loaded = load_manifest(tmp.path());
    assert!(loaded.is_empty(), "missing manifest → empty vec");
}

// ── Contract: JSON parsing robustness ─────────────────────────────────────────

#[tokio::test]
async fn contract_spec049_pass1_tolerates_fenced_json() {
    let tmp = tempfile::tempdir().unwrap();
    let png = write_png(&tmp, "p.png");
    let mock = Arc::new(MockProvider::new());

    // Fenced response — should still parse correctly
    mock.add_response("```json\n{\"kind\":\"flowchart\",\"is_figure\":true}\n```")
        .await;
    mock.add_response("Flowchart description").await;

    let filter = FigureFilter::new(Arc::clone(&mock) as Arc<dyn edgequake_llm::LLMProvider>);
    let results = filter.run(&[candidate("p.png", png, 1)]).await.unwrap();

    assert_eq!(results[0].kind, FigureKind::Flowchart);
    assert!(results[0].is_figure);
}

#[tokio::test]
async fn contract_spec049_pass1_unknown_kind_is_conservative() {
    // Unknown kind → Other → is_figure=true (conservative: keep unknown)
    let tmp = tempfile::tempdir().unwrap();
    let png = write_png(&tmp, "p.png");
    let mock = Arc::new(MockProvider::new());

    mock.add_response(r#"{"kind":"totally_unknown_thing","is_figure":true}"#)
        .await;
    mock.add_response("Description of unknown thing").await;

    let filter = FigureFilter::new(Arc::clone(&mock) as Arc<dyn edgequake_llm::LLMProvider>);
    let results = filter.run(&[candidate("p.png", png, 1)]).await.unwrap();

    assert_eq!(results[0].kind, FigureKind::Other);
    assert!(results[0].is_figure, "Other is conservatively kept");
}

// ── Contract: FigureKind semantics ────────────────────────────────────────────

#[test]
fn contract_spec049_kind_is_figure_semantics() {
    // Real figures
    for kind in &[
        FigureKind::BarChart,
        FigureKind::LineChart,
        FigureKind::ScatterPlot,
        FigureKind::Heatmap,
        FigureKind::ArchitectureDiagram,
        FigureKind::Flowchart,
        FigureKind::SystemDemo,
        FigureKind::Illustration,
        FigureKind::TableVisual,
        FigureKind::Other,
    ] {
        assert!(kind.is_figure(), "{kind:?} should be kept");
    }
    // Noise
    for kind in &[
        FigureKind::Logo,
        FigureKind::IconLogo,
        FigureKind::TextBlock,
        FigureKind::DecorativeRule,
        FigureKind::Empty,
        FigureKind::Stamp,
        FigureKind::Signature,
        FigureKind::ScanArtefact,
        FigureKind::Watermark,
    ] {
        assert!(!kind.is_figure(), "{kind:?} should be discarded");
    }
}

// ── G-prune: discarded crops must not appear in assembled markdown ───────────

#[test]
fn contract_spec128_g_prune_assemble_omits_discarded() {
    use edgequake_pdf::figure_filter::{prune_figure_map, FigureFilterResult};
    use edgequake_pdf::vision_markdown::{assemble_vision_markdown_with_figures, VisionPageSlice};
    use edgequake_pdf::WrittenFigureAsset;
    use std::collections::HashMap;

    let fig = |rel: &str, idx: usize| WrittenFigureAsset {
        page_num: 1,
        index: idx,
        rel_path: rel.into(),
        width: 40,
        height: 40,
        bbox: Some((0.0, 0.0, 40.0, 40.0)),
    };
    let mut map = HashMap::new();
    map.insert(
        1usize,
        vec![
            fig("assets/keep-a.png", 1),
            fig("assets/drop-b.png", 2),
            fig("assets/keep-c.png", 3),
            fig("assets/drop-d.png", 4),
            fig("assets/keep-e.png", 5),
        ],
    );
    let results = vec![
        FigureFilterResult {
            rel_path: "assets/keep-a.png".into(),
            page_num: 1,
            label: String::new(),
            kind: FigureKind::BarChart,
            is_figure: true,
            description: "a".into(),
        },
        FigureFilterResult {
            rel_path: "assets/drop-b.png".into(),
            page_num: 1,
            label: String::new(),
            kind: FigureKind::Logo,
            is_figure: false,
            description: String::new(),
        },
        FigureFilterResult {
            rel_path: "assets/keep-c.png".into(),
            page_num: 1,
            label: String::new(),
            kind: FigureKind::Diagram,
            is_figure: true,
            description: "c".into(),
        },
        FigureFilterResult {
            rel_path: "assets/drop-d.png".into(),
            page_num: 1,
            label: String::new(),
            kind: FigureKind::Stamp,
            is_figure: false,
            description: String::new(),
        },
        FigureFilterResult {
            rel_path: "assets/keep-e.png".into(),
            page_num: 1,
            label: String::new(),
            kind: FigureKind::Photograph,
            is_figure: true,
            description: "e".into(),
        },
    ];
    let pruned = prune_figure_map(map, &results);
    assert_eq!(pruned.get(&1).map(|v| v.len()), Some(3));
    let pages = vec![VisionPageSlice {
        page_num: 1,
        markdown: "## Figure 1\n\n## Figure 2\n\n## Figure 3\n\n## Figure 4\n\n## Figure 5".into(),
    }];
    let md = assemble_vision_markdown_with_figures(
        &pages,
        true,
        false,
        Some("doc"),
        None,
        Some(&pruned),
        None,
    );
    assert!(md.contains("keep-a.png"));
    assert!(md.contains("keep-c.png"));
    assert!(md.contains("keep-e.png"));
    assert!(!md.contains("drop-b.png"));
    assert!(!md.contains("drop-d.png"));
    let hrefs = md.matches("](assets/").count();
    assert_eq!(
        hrefs, 3,
        "assemble must emit exactly 3 figure hrefs after prune"
    );
}

#[test]
fn contract_spec128_g_prune_charts_and_inject_pass2() {
    use edgequake_pdf::figure_filter::{
        inject_kept_descriptions, prune_chart_crop_paths, prune_figure_map, FigureFilterResult,
    };
    use edgequake_pdf::vision_markdown::{assemble_vision_markdown_with_figures, VisionPageSlice};
    use edgequake_pdf::WrittenFigureAsset;
    use std::collections::HashMap;

    let mut map = HashMap::new();
    map.insert(
        1usize,
        vec![WrittenFigureAsset {
            page_num: 1,
            index: 1,
            rel_path: "assets/page-0001-fig-01.png".into(),
            width: 40,
            height: 40,
            bbox: Some((0.0, 0.0, 40.0, 40.0)),
        }],
    );
    let mut charts = HashMap::new();
    charts.insert(1usize, "assets/page-0001-chart.png".into());
    charts.insert(2usize, "assets/page-0002-chart.png".into());
    let results = vec![
        FigureFilterResult {
            rel_path: "assets/page-0001-fig-01.png".into(),
            page_num: 1,
            label: String::new(),
            kind: FigureKind::ArchitectureDiagram,
            is_figure: true,
            description: "Pipeline: ingest → index → query.".into(),
        },
        FigureFilterResult {
            rel_path: "assets/page-0001-chart.png".into(),
            page_num: 1,
            label: "Chart".into(),
            kind: FigureKind::BarChart,
            is_figure: true,
            description: "| X | Y |\n|---|---|\n| A | 10 |".into(),
        },
        FigureFilterResult {
            rel_path: "assets/page-0002-chart.png".into(),
            page_num: 2,
            label: "Chart".into(),
            kind: FigureKind::Logo,
            is_figure: false,
            description: String::new(),
        },
    ];
    let pruned = prune_figure_map(map, &results);
    let charts = prune_chart_crop_paths(charts, &results);
    assert!(charts.contains_key(&1));
    assert!(
        !charts.contains_key(&2),
        "artefact chart must not reach vision"
    );
    let pages = vec![
        VisionPageSlice {
            page_num: 1,
            markdown: "## Figure 1\n\nArchitecture.".into(),
        },
        VisionPageSlice {
            page_num: 2,
            markdown: "Publisher mark.".into(),
        },
    ];
    let md = assemble_vision_markdown_with_figures(
        &pages,
        true,
        true,
        Some("doc"),
        Some(&charts),
        Some(&pruned),
        None,
    );
    let md = inject_kept_descriptions(&md, &results);
    assert!(md.contains("assets/page-0001-fig-01.png"));
    assert!(md.contains("assets/page-0001-chart.png"));
    assert!(!md.contains("assets/page-0002-chart.png"));
    assert!(md.contains("Pipeline: ingest → index → query."));
    assert!(md.contains("| A | 10 |"));
    assert!(
        md.contains("<drawing"),
        "kept crops must stay drawing/vision targets"
    );
}

// ── Live LLM test (gated on OPENAI_API_KEY) ───────────────────────────────────

#[tokio::test]
async fn e2e_spec049_figure_filter_with_real_provider() {
    let _api_key =
        match std::env::var("OPENAI_API_KEY").or_else(|_| std::env::var("MISTRAL_API_KEY")) {
            Ok(k) => k,
            Err(_) => {
                println!("skip: no OPENAI_API_KEY or MISTRAL_API_KEY set");
                return;
            }
        };

    // Use a real arXiv figure PNG from the spec data directory.
    let spec_png = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("specs/049-improve-figure-extraction/e2e/markdown/lighrad_2410.05779v3/assets/p03-fig-01.png");

    if !spec_png.exists() {
        println!("skip: spec PNG not found at {}", spec_png.display());
        println!("       Run the stress test first: cargo test --test spec049_stress");
        return;
    }

    // Build real provider via ProviderFactory
    let (provider, _embedding) =
        edgequake_llm::ProviderFactory::from_env().expect("create provider from env");

    let filter = FigureFilter::new(Arc::clone(&provider));
    let results = filter
        .run(&[FigureCandidate {
            rel_path: "assets/p03-fig-01.png".into(),
            full_path: spec_png,
            page_num: 3,
            label: "Figure 1".into(),
            area_px: 10_000,
        }])
        .await
        .expect("filter run");

    assert_eq!(results.len(), 1);
    let r = &results[0];
    println!(
        "Live result: kind={:?} is_figure={} desc_len={}",
        r.kind,
        r.is_figure,
        r.description.len()
    );
    // LightRAG Figure 1 is a system architecture diagram — must be kept
    assert!(r.is_figure, "architecture diagram must be kept by Pass-1");
    assert!(
        !r.description.is_empty(),
        "Pass-2 must produce a description"
    );
}
