# SPEC-083 — First-Principles Defect Remediation

> **Product pin**: EdgeQuake v0.20.2  
> **Source register**: `edgequake-v0-20-2-registre-des-defauts.md` (2026-07-22)  
> **Docs status**: Complete study pack (code audit 2026-07-23; full-pack sync same day)  
> **Implementation**: Follow [03-implementation-roadmap.md](03-implementation-roadmap.md) — this tree is the law for fixes

## Verification status (SSOT)

See [01-defect-register.md](01-defect-register.md) Summary: **89 FIXED / 0 PARTIAL / 0 CONFIRMED / 1 RETRACTED** (90 studies).

Full closure (Waves 0–D, 2026-07-23): every open ID FIXED with named matrix evidence (C-19 remains RETRACTED). Battery: `spec083_matrix_contracts` + ignored `e2e_postgres_rls` + wave unit gates + clippy `-D warnings` on storage/api/pipeline/query.

---

## Start here

1. Read [00-first-principles.md](00-first-principles.md) — eight laws + SOLID/DRY primitives  
2. Skim [01-defect-register.md](01-defect-register.md) — every ID with HEAD audit status  
3. If you own production chat/ingest breakage → [INCIDENT-PROD-DIAGNOSIS.md](INCIDENT-PROD-DIAGNOSIS.md) + [clusters/00-schema-readiness/](clusters/00-schema-readiness/)  
4. Pick a defect → [`defects/<ID>.md`](defects/) (WHY → root → solution → e2e)  
5. Plan work via [03-implementation-roadmap.md](03-implementation-roadmap.md) + [02-cross-ref-matrix.md](02-cross-ref-matrix.md)  
6. Test via [04-e2e-test-matrix.md](04-e2e-test-matrix.md)

```
  Register (90 IDs*) -----> Audit (code is law) -----> Defect study
       |                                              |
       v                                              v
  Cluster summary <----- Laws / SOLID-DRY -----> Sprint roadmap
       |                                              |
       +-----------------> E2E matrix <---------------+
```

\* Source PDF said 91; register skips **D-43** (D-42→D-44). This pack has **90** stable IDs (P0, S-01…S-13, C-14…C-28, D-30…D-42, D-44…D-54, X-01…X-37) plus dead-code inventory in cluster 08.

---

## Directory map

| Path | Role |
|------|------|
| [00-first-principles.md](00-first-principles.md) | Laws, decisions, ID SSOT |
| [01-defect-register.md](01-defect-register.md) | Canonical register + audit columns |
| [02-cross-ref-matrix.md](02-cross-ref-matrix.md) | Defect ↔ file ↔ law ↔ sprint ↔ tests |
| [03-implementation-roadmap.md](03-implementation-roadmap.md) | Sprint 0–5 exit criteria |
| [04-e2e-test-matrix.md](04-e2e-test-matrix.md) | Edge cases + e2e per cluster |
| [INCIDENT-PROD-DIAGNOSIS.md](INCIDENT-PROD-DIAGNOSIS.md) | P0 eq_* production incident |
| [_template-defect-study.md](_template-defect-study.md) | Study template |
| [clusters/](clusters/) | Thematic WHY/ROOT/SOLUTION packs |
| [defects/](defects/) | One study per defect ID |

---

## Clusters

| Cluster | Focus | Primary IDs |
|---------|-------|-------------|
| [00-schema-readiness](clusters/00-schema-readiness/) | P0 eq_* | P0, X-03, C-20 |
| [01-tenant-isolation](clusters/01-tenant-isolation/) | RLS + WS | S-01…S-06, X-37, C-24, X-23 |
| [02-auth-transport](clusters/02-auth-transport/) | JWT/CORS/upload | S-07…S-13 |
| [03-graph-identity](clusters/03-graph-identity/) | Normalize/merge | C-14, D-30…D-34, X-15, X-17 |
| [04-pipeline-reliability](clusters/04-pipeline-reliability/) | Chunk/LLM/retry | C-15…C-18, C-21…C-23, X-06…X-10, X-13…X-19, X-28…X-31 |
| [05-query-fusion](clusters/05-query-fusion/) | Retrieval scores | D-35…D-40, X-04, X-05, X-20, X-22 |
| [06-ops-ci-sdk](clusters/06-ops-ci-sdk/) | Makefile/CI/SDK | D-41…D-54, X-01, X-02, X-11, X-12, X-24…X-27, X-32, X-33, X-36 |
| [07-accuracy-explain](clusters/07-accuracy-explain/) | Quality gates | X-21, X-34, X-35, D-54 |
| [08-dead-code](clusters/08-dead-code/) | Removals | §5 inventory, C-19, D-52 |

---

## Sprint snapshot

| Sprint | Goal |
|--------|------|
| **0** | Stop the bleed — eq_* DDL + fallback + boot gate |
| **1** | Security — real RLS, WS tenant, auth fail-closed, D-50 |
| **2** | Data corruption — C-14, retries, cosine Result, audit partitions, lineage order, FSM/checkpoint |
| **3** | LLM/tokens — typed retry, tokenizer, L2, gleaning options, query_vec |
| **4** | Debt — config SSOT, CI/SDK, dead code, ops hygiene |
| **5** | Accuracy — golden gates, Acc@N, ExplainTrace, Louvain hierarchy |

---

## Regenerating defect stubs

```bash
python3 docs/083-improvements/_gen_defects.py
```

Hand-edited depth lives in cluster folders and incident/roadmap docs; per-ID files are generated from `_gen_defects.py` catalog (edit catalog, re-run).
