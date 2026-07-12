# 003 — Lens: Product Owner

**Job:** decide what “done” means for transparent ingest UX  
**Cites:** FP-01…FP-10 · DEF-01…DEF-10

---

## 1. Jobs-to-be-done

| JTBD | User moment | Success look |
|------|-------------|--------------|
| **J1** Trust the wait | Uploaded a 100+ page PDF | Knows stage, N/M, ETA band, can leave |
| **J2** Spot stuck work | Soft-reprocess overnight | Sees Queued vs Working vs Stuck; one-click reprocess |
| **J3** Avoid double spend | Pipeline Busy + Completed rows | Never re-uploads thinking it failed |
| **J4** Audit cost | After batch | Cost column matches live cost events |
| **J5** Recover partial | Some chunks failed | Partial failure with retry path |

---

## 2. Outcomes (measurable)

| ID | Outcome | Gate |
|----|---------|------|
| PO-01 | Single busy semantics | Acc test: Busy ⇒ ≥1 active doc OR ≥1 processing task |
| PO-02 | Stage parity | Banner stage == row `current_stage` for active doc |
| PO-03 | Reprocess honesty | After reprocess click, stage resets within 2s poll |
| PO-04 | No dead API calls | Zero 404 on `/ingestion/*/progress` in network log |
| PO-05 | Soft-reprocess clarity | Mode badge: entities / merge / full (SPEC-047 P7e) |

---

## 3. Anti-goals

- Gamifying progress with fake percentages
- Hiding failures behind “Completed”
- Forcing users into Details dialog for basic “what’s happening”
- Shipping a redesign without fixing the progress contract

---

## 4. Priority (RICE-lite)

| Item | Reach | Impact | Confidence | Effort | Rank |
|------|-------|--------|------------|--------|------|
| Progress contract + WS bridge | All ingest users | High | High | M | **P0** |
| Busy/banner/row SSOT | All | High | High | S | **P0** |
| Reprocess stage reset | Soft-reprocess | Med | High | S | **P0** |
| Unified run panel UI | Power users | Med | Med | M | P1 |
| i18n completeness | Non-EN | Low | High | S | P2 |
| Timeline history (Fivetran-like) | Ops | Med | Med | L | P3 |

---

## 5. Narrative for stakeholders

> Ingestion is our most expensive user wait. Today the UI is a **projection of three incomplete backends**. SPEC-048 makes progress a **product contract**: one stage vocabulary, one busy rule, live counters where the system already knows N/M (pages, chunks, merge). Quality Acc (SPEC-047) is orthogonal — this is about **not losing users during the wait**.
