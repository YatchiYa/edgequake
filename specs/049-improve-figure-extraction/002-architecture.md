# 002 — Architecture (DRY / SOLID)

## Single responsibility modules (pdf2md)

```
pipeline/visual/
  types.rs           # VisualRegion, RegionKind, RegionSource, BBox
  geometry.rs        # union, iou, area_ok, pad — pure, no I/O
  object_cluster.rs  # L1: collect quads → overlap graph → components
  caption_label.rs   # Label attachment only (no detection)
  render_crop.rs     # Pdfium page render → crop_imm
  struct_tree.rs     # L0 proposer (trait + Pdfium FFI when available)
  mod.rs             # Orchestrator: L0 then L1; label; render
```

Legacy `extract_regions.rs` becomes a thin facade over `visual` (DRY).

## Traits (open/closed)

```rust
trait RegionProposer {
    fn propose(&self, page: &PdfPage, page_num: usize) -> Vec<RegionProposal>;
}

trait RegionLabeler {
    fn label(&self, page: &PdfPage, proposal: &mut RegionProposal);
}
```

- `StructTreeProposer` — L0  
- `ObjectClusterProposer` — L1  
- `CaptionGeometryLabeler` — attaches `Figure N` / `Table N` when text exists near bbox  
- `UnavailableStructTreeProposer` — empty L0 until high-level StructTree is wired

## EdgeQuake consumers (unchanged identity)

| Asset | Kind | Drawing? |
|-------|------|----------|
| `page-NNNN.png` | `page_full` | No |
| `page-NNNN-fig-MM.png` | `embedded_figure` | Yes |
| `page-NNNN-table-MM.png` | `table_crop` | Yes |
| `page-NNNN-chart.png` | `page_chart_crop` | Yes if ≤55% |

`region_assets::write_caption_region_assets` calls `extract_visual_regions_*`
(name kept for API stability; implementation is object-first).

## Dependency rule

- One Pdfium singleton (`render::get_pdfium`) — no second `FPDF_InitLibrary`
- One crop renderer — shared by L0/L1 proposals
- Classification Figure vs Table from object composition + optional label, not keywords alone
