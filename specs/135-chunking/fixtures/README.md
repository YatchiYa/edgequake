# Fixtures (SPEC-135)

Synthetic only. **Do not** vendor the live FreeToken paper (`free-token-2608-16157v1`
or any arXiv PDF/markdown).

Gold and packer are **unchanged** for v0.26.1 (CLI/docs patch; no fixture-byte
change). Shipped in **v0.26.0**; v0.26.1 must match the frozen hashes below.

| File | Gate |
|------|------|
| `freetoken_like.md` | `U-135-FILL`, `U-135-PROBE`, `U-135-TIKTOKEN`, `U-135-NO-COMMENT`, `U-135-KILL` |
| `freetoken_like.gold.json` | Closed `n_min`/`n_max`, `fill_p50_min`, probe ids, `n_legacy` band |
| `span_tiny.md` | `U-135-SPAN` |
| `oversize_page.md` | `U-135-NO-SPAN-OVERSIZE` |
| `mm_once.md` | `U-135-MM-ONCE` |
| `h1_block_span.md` | E1 negative (H1 blocks cross-page pack) |

## Frozen SHA-256 (file bytes)

Copied from [../08-test-protocol.md](../08-test-protocol.md). Also `sha256`
inside `freetoken_like.gold.json`. Changing fixture bytes without updating 08 +
gold is a failed gate.

| File | SHA-256 |
|------|---------|
| `freetoken_like.md` | `0f3b59fffe97a005c5d063075845699e1c42eda8d92fa7cab78efcd580c33be5` |
| `span_tiny.md` | `6c35a71bf672ce91f26b2bbfb04ba46958555b7cc6d7885be445cdd1605d1f44` |
| `oversize_page.md` | `0e840925e3134fb10e2149bb7ee976f9b920e2e2f7dfad602258559d06ba1c72` |
| `mm_once.md` | `322742ae94d56a3c0d712c40b5a9b05146472fca31c3a1366190aece29f89a1c` |
| `h1_block_span.md` | `90ac09bc62c76c5f7e7e4a6e83d5077fc00c2f231fcc97fe9cbcde8c2f907a8f` |

## Gold (`freetoken_like.gold.json`)

Packed Pdf at Fixed **1200/100** on `freetoken_like.md` (measured 2026-08-23;
**unchanged for v0.26.1**):

| Field | Value |
|-------|-------|
| `n` | 30 |
| `n_min` / `n_max` | 18 / 36 |
| `fill_p50` | 0.6725 |
| `fill_p50_min` | 0.55 |
| `n_legacy` | 20 (`EDGEQUAKE_PDF_PACK=0`, Recursive word-count) |
| `probe_fig` / `probe_prose` | `PROBE_FIG_A` / `PROBE_PROSE_A` (same chunk) |

`U-135-KILL` uses frozen `n_legacy=20` — do **not** require packed `N > n_max` on
this fixture. Rollback is distinguished by `N == n_legacy` plus word-count tokens.

## Recompute hashes

```bash
shasum -a 256 specs/135-chunking/fixtures/*.md
```

Mismatch ⇒ fail the suite (do not “update expected” inside the test).

## Rules

- Unique probe strings must appear **verbatim** (see gold JSON).
- Page markers use `<!-- edgequake-page:N -->` (1-indexed).
- VLM inline blocks use `**Type:**` + `edgequake-figure-vision` (or
  `<!-- edgequake-figure-vision:{rel_path} -->` from Pass-B) so P0 dedupe can match.
- No copyrighted paper text. Lorem is unique per sentence (`EQ135_P{page}_S{i}`).

## Run contract tests

```bash
cargo test -p edgequake-pipeline --test contract_spec135_pdf_pack
cargo test -p edgequake-api --test contract_spec135_mm_once
```
