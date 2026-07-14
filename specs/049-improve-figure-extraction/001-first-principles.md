# 001 — First principles

## Axiom 0

A PDF has **no** native `Figure` / `Table` / `Chart` paint types. Visuals are:

| Construct         | Paint                             | Typical use                |
| -------------------| -----------------------------------| ----------------------------|
| Image XObject     | `Do` `/Subtype /Image`            | Raster figures             |
| Form XObject      | `Do` `/Subtype /Form` + BBox×CTM  | Vector figures             |
| Path / Shading    | stroke/fill                       | Axes, ruled tables         |
| Text              | `Tj`/`TJ`                         | Captions, cell glyphs      |
| Tagged StructTree | `Figure`/`Table`/`Caption` + MCID | Authoritative when present |

## Cascade of truth

| Layer | Signal | Authority |
|-------|--------|-----------|
| **L0** | StructTree `Figure` / `Table` | Spec-defined (ISO §14.7) |
| **L1** | Page-space object quads + overlap clustering | Paint placement (ISO §8.10) |
| **L2** | Calibrated layout model (optional) | Statistical; pinned weights |
| **L3** | VLM page-normalized boxes | Residual only |

Higher layers **never** overridden by lower ones for the same visual.

## Banned as primary detectors

- Searching `"Figure "` / `"Fig. "` / `"Table "` to invent a crop
- Magic vertical ceilings (e.g. +420 pt above caption)
- Pass-A English keyword lists (`text_suggests_chart`) as sole chart proposal
- Inventing `assets/page-NNNN-fig-MM.png` when the file was never written

## Allowed invariants (not detectors)

- Reject region area ∉ `[MIN_AREA_FRAC, MAX_AREA_FRAC]` (default 0.02–0.55)
- Never emit Drawing for `page-NNNN.png` (full-page viewer context)
- Never invent missing on-disk asset paths; strip stale hrefs
- Chart residual only if page has **no** fig and **no** table proposals

## Caption role

Captions are **labels** attached to an already-proposed region (nearest caption
text by geometry). They are not the search key that creates the region.

## Text-native tables vs visual crops (modality split)

A “table” in a PDF is not one thing. Split by **paint modality**:

| What the PDF actually has | Correct channel | Visual crop (`-table-` PNG)? |
|---------------------------|-----------------|------------------------------|
| Cell glyphs + whitespace / ruled paths that serialize to MD/HTML | **Text** ingest (pdf→md / layout table extract) | **No** — pixels add no new facts; inventing a crop duplicates text |
| Image XObject of a table (scan, screenshot, exported sheet) | **Visual** region → `-table-` / fig asset + VLM/caption | **Yes** — text layer missing or unreliable |
| Form / heavy vector grid that does not round-trip to structured text | **Visual** residual (L1 cluster) only if area invariant passes | **Yes**, only when text channel loses structure |

**Axiom:** Visual extraction exists to recover meaning that **text extraction cannot**.
If the table’s cells are already selectable text (Ideas Tables 1–6 style), the
RAG answer lives in markdown chunks — do **not** require a PNG Drawing asset.

Failure mode to avoid: keyword “Table N” → invent near-full-page crop → viewer
shows a page dump and retrieval indexes noise.
