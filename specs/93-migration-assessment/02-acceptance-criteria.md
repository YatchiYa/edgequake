# 02 — Acceptance Criteria

Machine-checkable gates. The soak harness maps each AC to explicit PASS/FAIL lines and `verdict.json`.

| ID | Criterion | Harness assert |
| --- | --- | --- |
| **AC-M-01** | Source is published v0.22.0; pre-upgrade max migration **&lt; 125** (expect ≤105) | `SELECT max(version) FROM _sqlx_migrations` before migrate |
| **AC-M-02** | Realism corpus: ≥5 tenants, ≥15 workspaces, ≥600 docs (or documented override in report) | Seed counters + `SPEC93_PROFILE` recorded |
| **AC-M-03** | `migrate dry-run` shows irreversible **125**, does **not** advance ledger | Log markers + max version equality |
| **AC-M-04** | `migrate --confirm-drop` applies through **≥137**; **125/126/131** recorded; KV-drop message present | `_sqlx_migrations` + confirm log greps |
| **AC-M-05** | Post-drop: zero `public.eq_%_kv`; HEAD `/health` healthy with relational flags | SQL count + health JSON |
| **AC-M-06** | Multi-tenant isolation + wipe: disjoint doc IDs; wipe one WS leaves sibling + other tenants intact | Cross-tenant `comm`; post-wipe list counts |
| **AC-M-07** | Assets/list non-500; fence-on query path non-500 for a seeded workspace | HTTP status asserts |
| **AC-M-08** | Matrix: **pg16 + pg17 + pg18** each GREEN with reports under this pack | `matrix-summary.md` |

## Profile note

| Profile | Satisfies |
| --- | --- |
| `realism` (default for `make spec93-migration-assessment`) | AC-M-01..08 when matrix completes |
| `smoke` (`make spec091-upgrade-soak`) | AC-M-01, AC-M-03..07 only (AC-M-02/08 waived) |
