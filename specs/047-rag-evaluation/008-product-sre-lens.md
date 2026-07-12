# 008 — Product / SRE Lens

**Cross-ref:** [000](./000-index.md) · [006](./006-mlops-lens.md) · [010](./010-smoke-then-full-runbook.md) · [012](./012-acceptance-criteria-and-scorecard.md)

---

## 1. Who consumes the result?

| Persona | Needs in <60s |
|---------|----------------|
| Eng lead | F1/Acc trend vs last EdgeQuake version |
| AI eng | Which slice is broken (chart? cross-page? unanswerable?) |
| SRE | Did the run finish validly? cost? error rate? |
| PM | Plain-language: “better / worse / inconclusive” |

`SUMMARY.md` must serve all four without opening JSONL.

---

## 2. Operability SLOs for the harness

| SLO | Target |
|-----|--------|
| Time to first smoke scorecard (warm cache) | < 1 working day |
| Resume after interrupt | no duplicate charges for completed docs |
| Doctor false-green rate | 0 (vision/embed misconfig must fail) |
| Scorecard parseability | JSON Schema validate 100% |

---

## 3. Risk register

| Risk | Impact | Mitigation |
|------|--------|------------|
| Vision cost blow-up | $$ | smoke first; cost gate; concurrency limits |
| NC license misuse | Legal | research-only notice; no PDF redistribution |
| False marketing vs LVLM leaderboard | Trust | mandatory banner |
| Flaky extractor | Noise | retry; dual mode; pin model versions |
| Backend OOM on large PDFs | Run death | per-doc isolation; continue on fail |
| Stale `supports_vision=false` | Silent quality loss | doctor fail-closed |

---

## 4. Easy evaluation UX (product requirement)

`SUMMARY.md` template:

```markdown
# SPEC-047 {stage} — {date}

> RAG adaptation banner...

## Verdict
- valid: yes/no
- Overall Acc: 0.xx (n=N)
- Overall F1: 0.xx
- Gates: PASS/FAIL

## Progression
| slice | this run | previous smoke | delta |
| ... |

## Top failures (5)
1. doc / question / gold / pred / score

## Ops
- ingest_coverage, cost_usd, p95_latency, errors
```

Optional: tiny HTML or terminal table via `bench047 report --pretty`.

---

## 5. Version comparison workflow

```bash
bench047 report artifacts/full --compare artifacts/full-prev
```

Diff: Acc, F1, slices, ingest_coverage. Flag regressions > noise band.

---

## 6. Product/SRE acceptance

- [ ] SUMMARY readable by non-ML person  
- [ ] Cost gate on core/full  
- [ ] Risk banner present  
- [ ] Compare mode defined  

Next: [009 Implementation Plan](./009-implementation-plan.md).
