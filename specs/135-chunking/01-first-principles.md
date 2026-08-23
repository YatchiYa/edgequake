# 01 — First principles (LAW-135-1 … LAW-135-12)

> **Layering (do not collapse):**
>
> ```ascii
>   SPEC-116  HOW LARGE     workspace 1200/100 policy (unchanged)
>   SPEC-125  WHERE TO CUT  markdown (.md files)
>   SPEC-135  WHERE TO CUT  PDF-converted markdown + INDEX MM ONCE
>   SPEC-033  HOW TO CITE   page_start, page_end; span allowed after 135
> ```

## LAW-135-1 — Budget is a pack target

Token budget is a **pack target**. Structure (page marker, ATX heading, table,
figure, code fence) is a **constraint** on where a cut *may* occur, not a
reason to emit a chunk.

```ascii
  WRONG   structure unit  →  emit chunk
  RIGHT   accumulate units until tiktoken ≥ budget
          then cut at the last legal boundary
```

## LAW-135-2 — Atomic ≠ flush-before-neighbor

**Atomic** means: do not split the unit *internally* (do not cut a table row
in half; do not split a VLM figure block).

It does **not** mean: flush this unit as its own chunk before the next
paragraph.

Pack figure + caption + following prose when combined tiktoken ≤ budget.
`U-135-PROBE` is the unfakable proof: `PROBE_FIG_A` and `PROBE_PROSE_A` must
share a chunk when they fit.

## LAW-135-3 — PDF inner strategy is the SPEC-125 packer

`ChunkStrategy::Pdf` / `PageAwareChunking` inner default is
`markdown_pack.rs` (tiktoken pack-to-budget).

Recursive **word-count** remains Acc **R** only (`ChunkStrategy::Recursive`,
SPEC-026 `recursive_token_len`). Product PDF must not use it as the inner
splitter unless `EDGEQUAKE_PDF_PACK=0`.

SPEC-125 E10/E30 (“PDF stays Recursive”) is **reversed** by this spec.

## LAW-135-4 — Length SSOT is tiktoken cl100k

For product PDF and markdown packing, length = `count_tokens` (`cl100k_base`).

`ChunkResult.tokens` **must** equal `count_tokens(content)`. Word-count
proxies (`len/0.75`) are Acc R only. `U-135-TIKTOKEN` is the gate.

## LAW-135-5 — Index each figure once

If Pass-A already inlined VLM text for an asset (`**Type:**`,
`edgequake-figure-vision`, figure caption in the page body), do **not** append
a LightRAG-style `[Chart Name]<id>` sidecar for the same asset.

LightRAG appends sidecars because its **F** window never saw VLM text.
EdgeQuake already inlined it. Follow LightRAG **intent** (MM once), not
mechanical double-append.

```ascii
  inline VLM present for asset X  →  skip sidecar for X
  no inline VLM for asset X       →  sidecar still allowed (LAW-047)
  EDGEQUAKE_MM_CHUNKS=0           →  skip all sidecar append
```

`U-135-MM-ONCE` is the gate.

## LAW-135-6 — Control-plane comments are not extract units

Never emit a chunk whose trimmed content is only:

- `<!-- edgequake-page:N -->`
- `<!-- multimodal-chunks -->`
- an empty HTML comment

Markers are **control plane**. `U-135-NO-COMMENT` is the gate.

## LAW-135-7 — Honor min_chunk_size

No extract job below `min_chunk_size` (clamped to budget) except:

- last remainder of the document
- a single oversize atomic that cannot split (table/figure/code over budget)

P2 exists so remainder of page N does not become an orphan extract job when
page N+1 can legally continue the same section.

## LAW-135-8 — Page is attribution (P2)

Prefer packing **within** a page. If remainder of page N + start of page N+1
fit the budget **and** both are under-floor or continuation of the same
section, pack them and set:

```ascii
  page_start = N
  page_end   = M     (M ≥ N)
```

Deep-link / PDF viewer opens **`page_start`**. Citation badge may show
`p.N–M` when `M > N`.

Blocked when: next unit is a new `#` H1; next unit is an oversize atomic;
language/script change; `EDGEQUAKE_PDF_CROSS_PAGE_PACK=0`.

`U-135-SPAN` / `U-135-NO-SPAN-OVERSIZE` are the gates.

## LAW-135-9 — Persist page columns, not only JSON

`page_start` / `page_end` must be written on **relational `chunks` columns**.

Today `relational_chunk_writer.rs` copies pages into `metadata` JSON while
the `Chunk` domain type has no page fields and INSERT leaves columns NULL.
Live FreeToken rows had `page_start` NULL. SPEC-033 citation cannot use the
table.

`E2E-135-01` is the gate: `SELECT page_start, page_end, count(*)` matches gold.

## LAW-135-10 — Observability: fill, not just N

`ingest.chunking` (SPEC-124) must emit:

| Field | Meaning |
|-------|---------|
| `chunks` | emit count |
| `token_min` / `token_p50` / `token_max` | tiktoken spread |
| `orphan_heading_chunks` | heading-only (SPEC-125) |
| **`fill_p50`** | `token_p50 / budget` |
| `mm_sidecar_appended` | bool — whether any sidecar was concatenated |

Fail-open: if `fill_p50 < 0.4` on docs ≥ 8k tiktoken, **log + metric**, do
**not** abort ingest.

## LAW-135-11 — Future ingestions only + kill switches

LAW-116-4 holds: changing pack policy does not rewrite existing chunks.
Rebuild KG is explicit.

| Env | Default (unset) | `=0` / false |
|-----|-----------------|--------------|
| `EDGEQUAKE_PDF_PACK` | ON — packer inner | Recursive inner (pre-135) |
| `EDGEQUAKE_PDF_CROSS_PAGE_PACK` | ON — P2 span | P1 only, hard page emit |
| `EDGEQUAKE_MM_CHUNKS` | ON — sidecars allowed | no sidecar append |
| Inline MM dedupe | ON when inline VLM present | (no separate env; follows 135-5) |

`U-135-KILL` is the gate.

## LAW-135-12 — Acc honesty

Acc **R** and **F** on **non-PDF** text are unchanged. Existing
`contract_spec026_recursive_chunking` and `e2e_spec116_chunk_geometry` stay green
(`U-135-ACC-R`).

Acc **PDF** geometry **will** change (N down, fill up). Publication either:

1. re-runs medical-mid dual-SUT, or
2. pins `EDGEQUAKE_PDF_PACK=0` for Acc PDF docs.

This spec does **not** claim Acc-neutral PDF geometry. See
[12-honest-assessment.md](12-honest-assessment.md).

## Invariants (quick)

```ascii
  I-135-A  Pdf default inner = markdown_pack (unless PDF_PACK=0)
  I-135-B  ChunkResult.tokens == count_tokens(content)
  I-135-C  no comment-only chunks
  I-135-D  each figure id in at most one extract unit (inline XOR sidecar)
  I-135-E  page_end ≥ page_start; both columns NOT NULL when markers present
  I-135-F  fill_p50 observed; warn if < 0.4 on large docs
  I-135-G  kill switches restore pre-135 geometry on the same fixture
```
