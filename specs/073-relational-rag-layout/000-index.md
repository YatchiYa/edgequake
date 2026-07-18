# SPEC-073 — Relational doc/chunk + pgvector (first principles)

**Status:** Assessment complete + **July 2026 industry scale playbook** (no floor raise; no silent schema flip)  
**Depends on:** SPEC-063 (physics), SPEC-064/068 (Wave-2), SPEC-069 (dedicated HNSW), SPEC-070/072 (DiskANN), SPEC-058/059 (retract)  
**Goal:** Explain clearly how **workspace → document → chunk → embedding** in Postgres + pgvector improves **reliability** and **scalability**, using first principles and **industry best practices (July 2026)**, then map EdgeQuake’s dual-store reality.

## Question

How does “document and chunk in a relational database + pgvector, with documents linked to a workspace” improve reliability and scale — and what should EdgeQuake keep, harden, or defer?

## Pack

| Doc | Content |
|-----|---------|
| [`001-first-principles.md`](001-first-principles.md) | Four units, physics, filter–index law |
| [`002-edgequake-mapping.md`](002-edgequake-mapping.md) | Ideal model ↔ KV / vectors / AGE / relational sidecar |
| [`003-reliability-scalability.md`](003-reliability-scalability.md) | Mechanisms + measured cliffs |
| [`004-recommendations.md`](004-recommendations.md) | Do / do not; retract checklist |
| [`005-industry-scale-playbook.md`](005-industry-scale-playbook.md) | **July 2026 scale ladder** (clear, ordered) |
| [`006-research-evidence-improvements.md`](006-research-evidence-improvements.md) | Official + research evidence → P0/P1/P2 storage improvements |
| [`007-adr-relational-rag-layout.md`](007-adr-relational-rag-layout.md) | **ADR-073** — formal multi-lens architecture decision (Accepted) |

**Start here for “what did we decide?”:** [`007` ADR](007-adr-relational-rag-layout.md).  
**Start here for “how do we scale?”:** [`005`](005-industry-scale-playbook.md).  
**Start here for “what should we improve next?”:** [`006`](006-research-evidence-improvements.md).  
**Start here for “what does the data model look like?”:** [`docs/deep-dives/data-layer.md`](../../docs/deep-dives/data-layer.md) §1–3 (Mermaid + ASCII) · [`002`](002-edgequake-mapping.md).

## Locked answers (TL;DR)

1. **Workspace linkage + denorm filter columns** make multi-tenant ANN reliable (index ≡ filter) and scalable (bounded subgraph).
2. **Document** = ownership/delete; **chunk** = retrieval; **embedding** = ANN — conflating them causes integrity and capacity bugs.
3. Industry scales in order: **schema denorm → HNSW → halfvec → fix filter trap (partial/partition/iterative_scan) → residency → DiskANN → hybrid/rerank → external ANN last**.
4. EdgeQuake’s **split SSOT** (relational + KV + vectors + AGE) is deliberate; paid for with **saga retract**.
5. **No floor raise** from this pack. Product path: Wave-2 @100k default; opt-in DiskANN @150k (`q_list≥400`). Industry “~10M/node” bands are **not** EdgeQuake claims.

## Related

- Claim SSOT: [`docs/product-limits.md`](../../docs/product-limits.md)
- Data plane: [`docs/deep-dives/data-layer.md`](../../docs/deep-dives/data-layer.md)
- Physics: [`specs/063-architecture-capacity-assessment/001-first-principles.md`](../063-architecture-capacity-assessment/001-first-principles.md)
- DiskANN promote: [`specs/072-diskann-recall-pareto/e2e/artifacts/RUN_NOTES.md`](../072-diskann-recall-pareto/e2e/artifacts/RUN_NOTES.md)

## Checklist

- [x] Pack docs 000–004
- [x] July 2026 industry scale playbook (`005`)
- [x] Research evidence + improvement brainstorm (`006`)
- [x] Formal multi-lens ADR (`007`)
- [x] EdgeQuake dual-store mapping with citations
- [x] Locked recommendations (no silent unify; floors unchanged)
- [x] Cross-link data-layer; `make product-limits-check` green
