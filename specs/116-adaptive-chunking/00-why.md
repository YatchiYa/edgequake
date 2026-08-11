# 00 — Why SPEC-116

## Trigger

Partners and Acc need **LightRAG-fair chunk geometry** (adaptive off, 1200/100). Today that requires fleet env or per-upload `chunk_options`. Neither is a first-class workspace product control.

SPEC-115 live Mistral on the LightRAG paper: product adaptive → N≈16 / U≈491 vs fair/LR → N≈12–13 / U≈367–375 (~1.33× density from geometry alone).

## Why not only env?

| Approach | Gap |
|----------|-----|
| Fleet env | Affects every workspace; Acc/dev clash |
| Per-upload `chunk_options` | API-only; WebUI silent; easy to forget |
| **Workspace policy** | Inherit by default; explicit Fixed Acc-fair; future ingestions |

## Non-goals

- Change fleet default adaptive ON
- Change Acc publication env pins
- Rewrite document-card M→U metrics (SPEC-108 / 086)
- Auto-rebuild KG on save
- Schema migration for typed columns (metadata JSON only)

## Success

1. Workspace can **Inherit / Adaptive / Fixed** with Acc-fair preset.
2. Resolve is one SSOT in `edgequake-pipeline` (LAW-116-6).
3. UX matches Extraction Language: one card, future-only hint, rebuild toast.
4. Edge cases validated (overlap &lt; size, doc options last, tenant isolation).
5. Docs cross-ref 025 / 108 / 115 / 096 / 101; code is law.
