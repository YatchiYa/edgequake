# 080 Phase G — Acc promote checklist

**Status:** Blocked · Beat gates unmet · **Acc Beat fishing STOP**  
**Parent:** [080](./080-beat-lightrag-evidence-roadmap.md) · [081](./081-beat-parity-first-principles.md) · [082](./082-gold-citation-compat.md) (G1 REJECT) · **[088](./088-beat-ctx-fact-er-program.md)** (ctx + Fact ER program)  
**Acc SSOT today:** medical-mid [`T110218Z`](../e2e/artifacts/history/medical-mid-20260815T110218Z/) · `publish/latest` (EQ Acc **0.792** / LR **0.786** tie; ctx **0.471**; Fact ER **0.847**)  
**Acc-law full (ran, not Beat):** [`T012004Z`](../e2e/artifacts/history/medical-full-20260816T012004Z/) · peer `ACC_E2OCC_086_MEDICAL_FULL_v1` (Acc **0.786/0.786** point tie; ctx **0.427** FAIL · chunk **1200/100**)  
**Best gap-close Acc CI:** E2-B5 [`T133053Z`](../e2e/artifacts/history/medical-mid-20260722T133053Z/) (Acc tie; ctx 0.491; Fact ER 0.917)  
**Published stance:** mid Parity **unfinished** — do not claim Beat; query-only 088 levers exhausted; Acc ingest **chunk 1200/100**

Do **not** replace Acc `publish/latest` until **all** boxes are checked on the **same** winner pack:

- [ ] medical-mid: Δ Acc CI excludes 0 with EQ ahead  
- [ ] medical-mid: `ctx_rel ≥ 0.50`  
- [ ] medical-mid: overall ER ≥ LR−0.03 **and** Fact ER ≥ LR−0.03  
- [x] medical-full n=2062 **ran** on Acc-law 086 (labeled) — **Beat gates unmet** (Acc point tie; ctx 0.427)  
- [ ] medical-full n=2062: same three Beat gates on a winner pack  
- [ ] Update `peers.json` `acc_headline` + [019](../019-business-eq-vs-lightrag-and-rag.md) (only when Beat)  
- [x] Keep gap-close / latency / Acc Fact / Acc-law full peers labeled  

Until then: no “EQ beats LightRAG” claim.
