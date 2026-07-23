# SPEC-083 — Implementation Roadmap

> **Docs**: [README](README.md) · [Laws](00-first-principles.md) · [Matrix](02-cross-ref-matrix.md) · [E2E](04-e2e-test-matrix.md)  
> **Rule**: Each PR names the law(s) restored and the defect IDs closed. Prefer shared primitives over one-off patches.

---

## Dependency graph

```
  Sprint0 SchemaReady ----------+
                                |
  Sprint1 Security/RLS/WS ------+--> safe multi-tenant baseline
                                |
  Sprint2 Graph identity -------+--> trustworthy KG writes
  Sprint2 Task/KV/status -------+
                                |
  Sprint3 LLM/tokens/query_vec -+--> reliable ingest + retrieval input
                                |
  Sprint4 Ops/CI/SDK/dead ------+--> shippable hygiene
                                |
  Sprint5 Accuracy gates -------+--> measured quality
```

---

## Sprint 0 — Stop the bleed (P0) — mostly done

| Item | Defects | Exit criteria | Status |
|------|---------|---------------|--------|
| Ops DDL on prod graphs | P0 | eq_* columns+indexes+triggers present | PARTIAL (P0) |
| `SchemaReadiness` + health | P0, X-03 | Boot gate or explicit fallback mode | X-03 FIXED |
| Property COALESCE fallback | X-03 | Chat works if columns missing/NULL | FIXED |
| Fix prefix-scan silent drops | X-03 | Non-backfilled edges visible via props | FIXED |
| Fix vacuous upsert contracts | C-20 | Assert eq_* ON CONFLICT | FIXED |

**Done when**: production chat no longer fails with `eq_source_id does not exist`; ingest not looping on missing arbiter.

---

## Sprint 1 — Security — mostly done

| Item | Defects | Exit criteria | Status |
|------|---------|---------------|--------|
| Real RLS transactions + FORCE | S-03, S-04, S-05, S-06 | E2E isolation without relying on WHERE alone | FIXED |
| WS identity + filter | S-01, C-24, X-23 | No cross-tenant events; Deletion* routed | FIXED (see register for X-23) |
| track_id ownership | S-02 | Foreign track → 404 | FIXED |
| JWT iss/aud/jti + Role fail-closed | S-07, S-08 | Logout kills access; unknown role 401 | FIXED |
| Startup/CORS/rate/upload/eval | S-09…S-13 | Fail-closed prod defaults | S-10 PARTIAL; others FIXED |
| Vision example | D-50 | Not openai by default | FIXED |
| IsolationPolicy doc+KV | X-37 | Written + KV cross-tenant test | FIXED |

---

## Sprint 2 — Data corruption & task integrity — mostly done

| Item | Defects | Exit criteria | Status |
|------|---------|---------------|--------|
| Entity normalize + dedup migration | C-14 | THE COMPANY / U+2019 / JOHN's one node | FIXED |
| Multigraph + weight + type + lineage | D-30…D-33, C-26 | Two rel types persist; lineage before cap | FIXED (see register) |
| Retries / cosine Result / audit partitions | C-18, C-28, D-45 | No zero-attempt; no panic; month+1 insert OK | C-18/C-28 FIXED; **D-45 CONFIRMED** |
| KV tx + status SSOT | C-22, C-23 | All-or-nothing upsert; dedup works | FIXED |
| Offsets / atomic split | C-15, C-16 | Lineage offsets correct; huge table splits | C-15 FIXED; **C-16 CONFIRMED** |
| Checkpoint + FSM | X-28, X-29 | Suffix change invalidates; Cancelled↛Success | FIXED |

---

## Sprint 3 — LLM, tokens, retrieval input — mostly done

| Item | Defects | Exit criteria | Status |
|------|---------|---------------|--------|
| Typed RetryExecutor + jitter + breaker | X-06, X-07, X-30 | Zero substring retry matches | FIXED |
| TokenEstimator tiktoken | D-53, D-34 | One gate; NeedsLlm always summarizes | FIXED |
| L2 + batch SSOT | X-10, X-08 | Ollama cosine sane; one batch env | FIXED (see register for X-08) |
| Gleaning options | C-17 | Uses CompletionOptions | FIXED |
| batch get_by_ids | C-21 | One round-trip | FIXED |
| query_vec question-only + min_score | D-38, D-39 | History not in embedding; threshold held | FIXED |
| JSON empty fail-closed; separators; markers | X-16, X-14, X-13 | No silent 0-entity success | X-16 FIXED; **X-13 CONFIRMED** |

---

## Sprint 4 — Debt & hygiene — partial

| Item | Defects | Exit criteria | Status |
|------|---------|---------------|--------|
| Config SSOT | X-36 | Single resolve precedence | FIXED |
| Dead code removal | §5, D-52, C-19 stub | LOC gone; tests green | D-52 FIXED; C-19 RETRACTED |
| CI/SDK/Makefile/sed | X-32, X-33, D-47…D-49, D-48 | Gates block; workflows at root | D-47/D-49/X-33/D-48 FIXED; **X-32 PARTIAL** |
| Progress/ETA/upload/OTEL/LRU | D-41, D-42, D-44, D-46, D-51, C-27 | Weighted progress; 50MiB SSOT | D-41/D-42/D-44/D-46/D-51/C-27 FIXED |
| OpenAPI/FE/middleware/comments | X-24…X-27, X-25, X-26 | Routes gate; middleware present | X-24/X-26/X-27 FIXED; X-25 PARTIAL |
| Fusion naming/docs; FTS; vector caps | D-35…D-37, D-40, X-04, X-05 | Names=behavior; stream stats | D-36 FIXED; rest CONFIRMED |
| Diamond llm; Anthropic URL; PDF concurrency | X-09, C-25, X-12 | Single version; URL ok; honest concurrency | X-09/C-25/X-12 FIXED (X-09 documents pdf2md diamond ≤2) |
| Migrations honesty | X-01, X-02 | Documented; drift fails loud | FIXED |
| Drain timeout; Scan/Reindex decision | X-31, X-11 | Shutdown budget; implement or remove | X-31 FIXED; X-11 FIXED (501) |

---

## Sprint 5 — Accuracy & explainability — done (full closure)

| Item | Defects | Exit criteria | Status |
|------|---------|---------------|--------|
| Live golden + Acc@N gate | X-34, X-35 | Nightly thresholds; honest publish | FIXED |
| ExplainTrace MVP | X-21 | API field populated from arms | FIXED |
| Louvain hierarchy optional | D-54 | Phase-2 behind flag | FIXED |
| Entity resolution enhancements | X-17 | Fuzzy behind `EDGEQUAKE_ENTITY_FUZZY` | FIXED |

---

## PR checklist (every fix)

- [ ] Defect IDs in PR title/body  
- [ ] Law(s) named  
- [ ] Shared primitive preferred over local hack  
- [ ] Contract or e2e from [04-e2e-test-matrix.md](04-e2e-test-matrix.md)  
- [ ] No new fail-open security default  
- [ ] `cargo test -p <crate>` / relevant e2e green  

---

## Out of scope for docs workstream

Application code changes land in follow-up PRs tracked by this roadmap. This directory is the specification of record.
