# 00 — Why SPEC-105

After SPEC-091 typed cutover and SPEC-104 monitor naming fixes, **allow-paths** still prefer or recreate legacy stores under misconfiguration, and a naïve “typed-only forever” change would break **≤0.22 upgrades** (KV-era dual-read required until 125).

## Residuals

1. Unknown `EDGEQUAKE_VECTOR_BACKEND` → LegacyTables
2. Cutover guard holes when 131 bookkeeping vs empty census disagree
3. Workspace stats hardcode `eq_eq_default_vectors`
4. Stale health contract expecting `chunk_text_ssot=kv`
5. No post-drop **assert** migration after 141

## Non-goals

Replacing 125–131; deleting Dual enum; dropping `edgequake.*` views; SPEC-089 timeouts.
