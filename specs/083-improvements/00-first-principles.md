# SPEC-083 — First Principles

> **Status**: Active  
> **Cross-refs**: [README](README.md) · [Register](01-defect-register.md) · [Roadmap](03-implementation-roadmap.md)

---

## 1. WHY this document exists

The v0.20.2 defect register lists **91** issues. Treating them as a flat bug list produces duplicated fixes and regressions. First principles collapse them into a small set of **laws**. Every defect study maps to at least one law; every implementation PR must name the law it restores.

---

## 2. The eight laws

```
  LAW-1  Isolation is a DB invariant, not a hope in application WHERE
  LAW-2  Schema readiness is a boot gate; hot paths never assume optional columns
  LAW-3  One SSOT per concern (config, tokens, retries, status, fusion score, GUC)
  LAW-4  Fail-closed on security / identity / unknown roles / insecure defaults
  LAW-5  Typed errors > string matching (LLM, breaker, ingestion failure class)
  LAW-6  Identity before strip/case (normalize order is a contract)
  LAW-7  Cap after lineage; never truncate evidence then derive scope from remainder
  LAW-8  Tests prove production defaults, not privileged test cascades
```

### ASCII: laws → system surfaces

```
                    +------------------+
                    |     LAW-3 SSOT   |
                    +--------+---------+
                             |
       +---------------------+---------------------+
       |                     |                     |
       v                     v                     v
 +-----------+         +-----------+         +-----------+
 | LAW-1/4   |         | LAW-2/6/7 |         | LAW-5     |
 | Auth+RLS  |         | Graph+KG  |         | LLM retry |
 +-----+-----+         +-----+-----+         +-----+-----+
       |                     |                     |
       +----------+----------+----------+----------+
                  |
                  v
           +-------------+
           | LAW-8 CI/E2E|
           +-------------+
```

---

## 3. SOLID mapping (how we implement)

| Letter | Meaning here | Shared primitives (DRY) |
|--------|--------------|-------------------------|
| **S** | One module owns one invariant | `SchemaReadiness`, `TenantContext`, `EntityId::normalize`, `RetryExecutor`, `TokenEstimator`, `ScoreScale`, `DocumentStatus` |
| **O** | Extend via policy enums, not `if provider` sprawl | `WeightPolicy`, `FusionMode`, `IsolationPolicy` |
| **L** | Subtypes honor contracts (e.g. stream stats ⊇ sync stats) | `QueryStats` SSOT |
| **I** | Narrow traits (`KVStorage::get_by_ids`, not N+1 loops) | storage traits |
| **D** | App depends on typed `LlmError`, not string soup | `retry_strategy()` |

Anti-patterns banned by this spec:

- Second copy of normalize / retry / GUC setter / score fusion
- Fail-open defaults in prod paths
- Contract tests that OR-match vacuous strings
- Heuristic token counts when tiktoken is already in the workspace

---

## 4. Locked architectural decisions

1. **Tenant isolation SSOT = real RLS** — `BEGIN` + `set_config(..., true)` inside the transaction; unify GUC to `app.current_*`; `FORCE ROW LEVEL SECURITY` including `document_originals`; remove `tenant_id IS NULL OR …`; keep app WHERE as defense-in-depth.
2. **eq_* readiness** — boot gate + property fallback until green; offline DDL with `lock_timeout` + retry.
3. **Entity identity** — NFC → casefold → strip articles/possessives (ASCII + U+2019) → UPPER; then dedup migration.
4. **Retry/LLM** — only `LlmError::retry_strategy()` + jitter + breaker; ban substring matching.
5. **Config** — one `EdgeQuakeConfig::resolve()` precedence chain.
6. **Multigraph** — unique edge `(source, target, relation_type)`.
7. **Fusion** — names match behavior (`sparse_first`, `rrf`); typed `ScoreScale`.
8. **This workstream** — documentation under `docs/083-improvements/` first; code follows the roadmap.

---

## 5. ID SSOT (critical)

Register **X-01…X-12** are official complementary defects (migration 002, checksums, eq_* fallback, FTS, LLM retry, …).

Page-6 *unnumbered* items are promoted to **X-13…X-27** only. Do **not** remap page-6 symptoms onto X-01…X-12.

Register skips **D-43** (jumps D-42 → D-44). **D-50** VISION default is **FIXED** (`.env.example` no longer hardcodes openai). **D-54** = Louvain phase-1 (still CONFIRMED backlog). **C-19** = RETRACTED.

---

## 6. Verification standard (“code is law”)

Every defect study includes:

1. File:line locus verified against HEAD  
2. Audit status: CONFIRMED | PARTIAL | FIXED | RETRACTED  
3. WHY → root cause → ASCII diagram → SOLID/DRY solution  
4. Edge cases + named e2e/contract tests  

Audits performed 2026-07-23 against the v0.20.2 lineage informed [01-defect-register.md](01-defect-register.md).
