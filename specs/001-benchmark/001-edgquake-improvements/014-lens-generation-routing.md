# 014 — Lens: Generation & Product Routing

**Priority:** Product path (orthogonal to Acc headline pins)  
**Cross-ref:** [012 Multi-hop](./012-lens-multihop-graph.md) · [005 Mode Map](../005-mode-map-and-pins.md) · SPEC-046

---

## 1. Observation

| Type | EQ Acc | LR Acc | Product reading |
|------|--------|--------|-----------------|
| Fact Retrieval | **0.752** | 0.654 | EQ mix competitive for local facts |
| Complex Reasoning | 0.715 | **0.776** | Prefer cleaner path/graph selection |
| Contextual Summarize | 0.836 | **0.866** | Needs broad **relevant** coverage |
| Creative Generation | **0.755** | 0.720 | EQ strong under gold-style pin |

Generation quality is already high (cos ≈ 0.96 both SUTs). The type split argues for **routing**, not a single always-on Mix profile in production.

---

## 2. First-principles diagnosis

- Acc pin forces LR-like always-on arms (`MIX_ARM_GATE=false`) for fairness — that is an **eval** constraint, not the optimal **product** default.
- Production Mix with intent gate (Factual → naive-lean) can be correct for UX while remaining a **labeled ablation** vs Acc.
- Generator pin (`answer_style=gold`) equalizes style; do not confuse style wins with retrieval wins.

---

## 3. July 2026 practice

- **Hybrid routing:** vector for facts; graph/path for multi-hop; skip retrieval when base knowledge suffices (agentic).
- Long context refines retrieved evidence; it does not replace retrieval for multi-million-token corpora.
- Measure per-intent quality (Fact / Reason / Summarize / Creative) — GraphRAG-Bench’s four levels exist for this.

---

## 4. EQ insertion points

| Area | File / knobs | Action |
|------|--------------|--------|
| Intent arm mask | `mix_weights.rs` `intent_arm_mask`, `mix_arm_gate_enabled` | Product: gate on; Acc: force off |
| Query modes | API / UI mode selector | Expose naive / local / global / mix / hybrid with honest labels |
| Answer style | Acc `BENCH001_ANSWER_STYLE` | Keep `gold` for Acc; product may use `concise` / `default` |
| Prompt builder | `engine_impl/prompt.rs`, `context_format.rs` | Type-conditioned instructions (summarize vs fact) as product feature |

---

## 5. Experiments (one confound each)

| # | Change | Success |
|---|--------|---------|
| G1 | Product profile: arm gate **on** (not Acc) | Fact latency↓; Fact Acc ≥ Acc-pin Fact − 0.03 on smoke |
| G2 | Router: Fact→naive-lean, Reason→mix+path_prune | Per-type Acc ≥ always-mix on smoke stratified |
| G3 | Summarize-specific token floor for chunks | Summarize Acc↑; ctx_rel not↓ |
| G4 | Acc remains `MIX_ARM_GATE=false` always | Headline fairness preserved |

---

## 6. Non-goals

- Do not publish Acc under production arm-gate-on as “fair vs LightRAG mix.”
- Do not change generator model mid-ablation without updating lineage.
- Do not use Creative Faithfulness alone as the shipping metric (needs L2).
