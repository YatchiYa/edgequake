# 04 — Target Architecture

Reuse the SPEC-125 packer. Do not fork a second greedy algorithm. Change
**where** Pdf cuts, **how often** a figure is indexed, and **how** page is
stored and cited.

## Pipeline

```ascii
  PDF bytes
       │
       ▼
  Pass-A markdown + <!-- edgequake-page:N --> + inline VLM
       │
       ▼
  P0  enrich_processed_text_with_mm_chunks
       │  skip sidecar for asset id already inlined in body
       │  (Type: / edgequake-figure-vision / matching [Chart Name] id)
       │  still persist structured mm metadata in KV
       │  never emit <!-- multimodal-chunks --> if zero sidecars remain
       ▼
  P1  PageAware(MarkdownPack tiktoken)     inner default
       │  pack units until count_tokens ≥ budget
       │  atomic = do not split interior; DO pack with neighbor
       │  skip comment-only units
       │  still hard-split pages if CROSS_PAGE_PACK=0
       ▼
  P2  optional remainder pack across page markers (soft units)
       │  if remainder(N)+head(N+1) ≤ budget
       │     and not blocked (H1 / oversize atomic / script change / kill)
       │     → one chunk, page_start=N, page_end=M
       ▼
  Persist  Chunk row + metadata
       │  page_start / page_end COLUMNS bound (not JSON-only)
       │  token_count = count_tokens(content)
       ▼
  Extract N much smaller; citations span-capable
```

## TODAY vs TARGET

```ascii
  TODAY
    PageAware(Recursive word-count)
      → emit per page × per atomic region
      → append <!-- multimodal-chunks --> + [Chart Name] copies
      → N=70, p50=230, page columns NULL

  TARGET
    PageAware(MarkdownPack tiktoken)
      → pack units until budget
      → page span only when remainder would be an orphan
      → MM indexed once (inline OR sidecar, not both)
      → N~24-32 on the trigger class, p50 ≥ ~800 @ 1200
```

## Packer reuse (no fork)

```ascii
  markdown_pack.rs          SSOT greedy pack (LAW-125 + LAW-135-3)
       ▲
       │
       ├── MarkdownChunking          .md files (SPEC-125)
       └── PageAwareChunking.inner   PDF converted MD (SPEC-135)

  RecursiveCharacterChunking
       └── Acc R + EDGEQUAKE_PDF_PACK=0 only
```

Page markers are **soft units** for the packer when P2 is on: a marker is a
preferred cut, not a mandatory flush. When P2 is off, markers remain hard
splits (P1-only geometry).

## MM-once decision

```ascii
                    Pass-A body contains
                    VLM for asset X?
                         │
              yes        │        no
               │         │         │
               ▼         │         ▼
        skip sidecar X   │   append sidecar X
        (LAW-135-5)      │   (LAW-047 intent)
                         │
              EDGEQUAKE_MM_CHUNKS=0 → skip ALL sidecars
```

Detection (v1, deterministic): asset id appears in inlined figure block
(`edgequake-figure-vision` / `**Type:**` neighborhood) **or** body already
contains `[Chart Name]<id>` / `[Figure Name]<id>`. False-negative (sidecar
still appended) is allowed; false-positive (dropping a figure never inlined)
is **not**. Tests: `U-135-MM-ONCE`.

## Page span

```ascii
  page N remainder 80 tok + page N+1 start 80 tok  ≤ 1200
       and continuation of same section
       → ONE chunk
         page_start = N
         page_end   = N+1
         deeplink   #page=N

  page N 2500 tok (oversize)
       → still splits inside the page
         no silent drop
         U-135-NO-SPAN-OVERSIZE
```

Blocked (see [10-edge-cases.md](10-edge-cases.md)):

- next unit is a new `#` H1
- next unit is an oversize atomic
- language/script change
- `EDGEQUAKE_PDF_CROSS_PAGE_PACK=0`

## Persistence target

```ascii
  ChunkResult.page_*
       │
       ├─ metadata JSON     (keep — lineage KV compat)
       └─ domain Chunk.page_start / page_end   NEW fields
              │
              ▼
         INSERT public.chunks (..., page_start, page_end, ...)
         NOT NULL when markers present on that chunk
```

OpenAPI / query / UI: `page_end` **may** be `>` `page_start`. Badge `p.3–4`.
Deeplink `#page={page_start}`.

## Kill switches (default ON when unset)

| Env | ON (unset) | OFF (`=0` / false) |
|-----|------------|---------------------|
| `EDGEQUAKE_PDF_PACK` | Packer inner | Recursive inner (pre-135) |
| `EDGEQUAKE_PDF_CROSS_PAGE_PACK` | P2 span | Hard page emit (P1) |
| `EDGEQUAKE_MM_CHUNKS` | Sidecars allowed (then 135-5 dedupe) | No sidecar append |

Inline dedupe has **no** extra env: it is the correct default whenever
sidecars would duplicate Pass-A VLM.

## Observability

`ingest.chunking` output JSON gains:

```json
{
  "chunks": 28,
  "token_min": 140,
  "token_p50": 980,
  "token_max": 1200,
  "fill_p50": 0.82,
  "orphan_heading_chunks": 0,
  "mm_sidecar_appended": false
}
```

`fill_p50 < 0.4` on docs ≥ 8k tiktoken → warn + metric, ingest continues.

## UI (see 06)

- Hierarchy / lineage: `p.3–4` when span; else `p.3`
- Viewer: open `page_start`
- Workspace card: one line that PDF packing fills the token budget (future-only)

## What does not change

- Workspace 1200/100 policy (SPEC-116)
- Acc Recursive / TokenBased on non-PDF
- SPEC-125 markdown path (already packing)
- Pass-A / Pass-B prompts
- Auto-rebuild (still explicit)
