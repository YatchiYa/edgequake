# Lens 003 — Database Expert

## Stake

Overlay is **per-page, per-region**, workspace-isolated, cascade-deleted with the document. JSON-in-document is the wrong grain.

## Rejected homes

| Candidate | Why not |
|-----------|---------|
| `pdf_documents` JSON array | 1:1 row bloat; overlay fetches one page |
| `pdf_documents.extraction_errors` | Error log |
| `document_mm_assets` | BYTEA images |
| `chunks.metadata` | Chunk-scoped; layout is page-scoped |
| `document_artifacts.kind` | Closed CHECK; one blob for all pages |
| `documents.metadata` | Overloaded; not per-page |

## Target DDL (migration ~148)

```ascii
  document_pages
    page_id            UUID PK
    document_id        UUID NOT NULL FK documents(id) ON DELETE CASCADE
    workspace_id       UUID NOT NULL FK workspaces
    page_number        INT NOT NULL CHECK (page_number >= 1)
    width_pt           DOUBLE PRECISION NOT NULL
    height_pt          DOUBLE PRECISION NOT NULL
    rotation           SMALLINT NOT NULL DEFAULT 0
    cropbox_pdf        JSONB NULL          -- {x0,y0,x1,y1} if ≠ MediaBox
    raster_width_px    INT NULL
    raster_height_px   INT NULL
    layout_model       TEXT NULL           -- pp-doclayout-v3@sha256:...
    layout_status      TEXT NOT NULL       -- pending|extracted|skipped|failed
    created_at / updated_at TIMESTAMPTZ
    UNIQUE (document_id, page_number)

  page_layout_regions
    region_id          UUID PK
    page_id            UUID NOT NULL FK document_pages ON DELETE CASCADE
    document_id        UUID NOT NULL FK documents ON DELETE CASCADE   -- denorm RLS
    workspace_id       UUID NOT NULL
    class              TEXT NOT NULL       -- canonical taxonomy
    source             TEXT NOT NULL       -- l0_struct|l1_paint|l2_layout|l3_vlm|derived
    bbox_pdf           JSONB NOT NULL      -- {x0,y0,x1,y1} PDF user space
    confidence         REAL NULL
    reading_order      INT NULL
    asset_path         TEXT NULL           -- mm-assets relative path if linked
    extra              JSONB NOT NULL DEFAULT '{}'
    created_at         TIMESTAMPTZ
```

Indexes:

- `document_pages (workspace_id, document_id)`
- `page_layout_regions (document_id, page_id)`
- `page_layout_regions (document_id, class)`

RLS: copy `document_mm_assets` workspace policies (fail-closed). Grant `edgequake` DML.

Do **not** store `bbox_norm` in SQL (derived at read; LAW-128-4).

## Processing window

Prefer insert after `document_id` is linked. If crash-resume needs pages earlier, add nullable `pdf_id UUID REFERENCES pdf_documents(pdf_id)` and unique `(pdf_id, page_number)` until document_id is set — only if WP-6 resume tests demand it. Default: write with `document_id`.

## Reprocess / delete

```ascii
  DELETE FROM document_pages WHERE document_id = $1;  -- cascades regions
  -- then rewrite
  document deletion: existing ON DELETE CASCADE
```

Do not TRUNCATE. Match mm-assets rewrite semantics.

## Chunk columns (non-goal)

Migration 066 added `chunks.page_start` / `page_end` but writers fill JSONB only. Overlay **must not** wait on backfill. Optional later DRY: write typed columns — separate spec.

## Size

Assume ~20–80 regions/page × 500 pages ≈ 40k rows — fine with indexes. Lazy GET per page. No GIN on `bbox_pdf` in v1 (no spatial queries).

## Cross-refs

- Architecture: [../04-target-architecture.md](../04-target-architecture.md)
- Edges: [../10-edge-cases.md](../10-edge-cases.md) (RLS, cascade)
- SPEC-091 sidecars: [../../091-simplify-data-layer/](../../091-simplify-data-layer/)
