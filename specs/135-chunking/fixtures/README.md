# Fixtures (SPEC-135)

Synthetic only. **Do not** vendor the live FreeToken paper (`free-token-2608-16157v1`
or any arXiv PDF/markdown).

| File | Gate |
|------|------|
| `freetoken_like.md` | `U-135-FILL`, `U-135-PROBE`, `U-135-TIKTOKEN`, `U-135-NO-COMMENT`, `U-135-KILL` |
| `freetoken_like.gold.json` | Closed `n_min`/`n_max`, `fill_p50_min`, probe ids, `n_legacy` band |
| `span_tiny.md` | `U-135-SPAN` |
| `oversize_page.md` | `U-135-NO-SPAN-OVERSIZE` |
| `mm_once.md` | `U-135-MM-ONCE` |
| `h1_block_span.md` | E1 negative (H1 blocks cross-page pack) |

SHA-256 of each `.md` is frozen in [../08-test-protocol.md](../08-test-protocol.md)
(and `sha256` inside `freetoken_like.gold.json`). Changing fixture bytes without
updating 08 + gold is a failed gate.

## Rules

- Unique probe strings must appear **verbatim** (see gold JSON).
- Page markers use `<!-- edgequake-page:N -->` (1-indexed).
- VLM inline blocks use `**Type:**` + `edgequake-figure-vision` so P0 dedupe can match.
- No copyrighted paper text. Lorem is unique per sentence (`EQ135_P{page}_S{i}`).
