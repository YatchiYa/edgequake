# 004 — Product Owner Lens

**Spec:** SPEC-057  
**Key question:** Does the product give users control, trust, and fair multi-tenant throughput?

---

## Scope

Business outcomes of ingestion reliability: time-to-indexed, cancel/reprocess controllability, tenant fairness, cost predictability. Out of scope: extraction quality metrics (RAG eval — SPEC-047).

---

## Value propositions at risk

| Promise | Failure mode | Business impact |
| ------- | ------------ | --------------- |
| “Upload and it will process” | Stuck `processing` after restart | Support load; lost confidence |
| “You can stop a bad job” | Cancel → Failed / unclear state | Users fear Cancel; keep burning LLM $ |
| “Shared cluster is fair” | One tenant fills workers | Other tenants look broken |
| “Large docs work” | 2h timeout / Vision bill shock | Churn on enterprise PDFs |
| “Failed means actionable” | `unknown` + blind retry | Wasted quota; no self-serve fix |

---

## Controllability scorecard

| Capability | Today | Target | REQ |
| ---------- | ----- | ------ | --- |
| Cancel single task | Shipped (API + UI) | Stopping… → Cancelled everywhere | REQ-057-05, 03 |
| Cancel all in-flight | `POST /pipeline/cancel` | Same status honesty | REQ-057-04 |
| Resume after crash | Opt-in hydrate / Reprocess | Explicit product policy + UI | REQ-057-01, 05 |
| Fairness across tenants | Park limiter | Runtime-correct clamp | REQ-057-09 |
| Cost circuit breakers | Taxonomy + local clamp | Permanent classes complete | REQ-057-13 |

---

## Multi-tenant SLOs (proposed product SLOs)

| SLO | Definition | Signal |
| --- | ---------- | ------ |
| Fair start | P95 time from enqueue to first stage progress under multi-tenant load | `tenant_park_waiters`, stage timestamps |
| Cancel latency | P95 time from cancel API to terminal Cancelled < 30s (excl. current HTTP RTT) | task status transitions |
| Restart honesty | After process restart, no doc stays `processing` > reconcile window without terminal or requeue | orphan reconcile metrics |
| Permanent fail clarity | 100% of permanent classes expose `failure_class` + `recommended_action` | KV metadata audit |

---

## Prioritization (PO view)

```text
  P0  Trust & controllability     (cancel status truth, failure_class)
  P1  Restart durability          (no silent lost work)
  P2  Throughput / large docs     (split phases, adaptive timeout)
  P3  Horizontal scale            (multi-instance delivery)
```

Ship P0 before marketing “production multi-tenant ingestion.” P1 before promising HA restarts. P2 before enterprise 500+ page SLAs. P3 before multi-replica workers.

---

## Recommendations

1. Product copy: default restart policy = **Reprocess** (matches `EDGEQUAKE_STARTUP_AUTO_RESUME=0`); document when to enable auto-resume.  
2. Treat Cancelled as a first-class analytics event (not Failed).  
3. Expose queue-metrics fairness signals in admin UI (park waiters).  
4. Tie large-PDF admission warnings to EdgeParse recommendation (SPEC-038).

**Out of scope:** Pricing model for vision pages; SSO/RBAC for cancel permissions.

Next: [005-ux-expert-lens.md](./005-ux-expert-lens.md)
