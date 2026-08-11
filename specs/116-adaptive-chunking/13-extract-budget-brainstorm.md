# 13 — Extract Budget Brainstorm (Deep Dive + How to Implement)

> Brainstorm after [`12-extract-budget-first-principles.md`](12-extract-budget-first-principles.md).  
> Goal: decide **whether** and **how** to evolve per-chunk entity/relation budgets beyond today’s 40/100.

## 0. Current state (code-is-law)

| Mechanism | Where | Behavior |
|-----------|-------|----------|
| Defaults | `DEFAULT_MAX_EXTRACTION_ENTITIES=40`, `DEFAULT_MAX_EXTRACTION_RECORDS=100` | LR parity |
| Env | `EDGEQUAKE_MAX_EXTRACTION_ENTITIES`, `EDGEQUAKE_MAX_EXTRACTION_RECORDS` | Fleet override |
| Soft | `ExtractionCaps::prompt_quantity_limits_section` | “Do not fill the limit” |
| Hard | `apply_extraction_caps` after JSON/tuple parse | Truncate ents → drop orphan rels → trim rels to total |
| Gleaning | pipeline default 1 | Second pass can recover misses (separate response, own budget) |
| Workspace UI | SPEC-116 chunking card | **Does not** expose extract caps yet |

LightRAG mirror: `MAX_EXTRACTION_ENTITIES` / `MAX_EXTRACTION_RECORDS` (PR [#2950](https://github.com/HKUDS/LightRAG/pull/2950)).

---

## 1. Brainstorm axes (force the question open)

### Axis A — Soft vs hard

| Mode | Pros | Cons |
|------|------|------|
| Soft only (prompt) | No order-bias truncate | Models ignore; timeouts; endless loops on local LLMs |
| Hard only (truncate, no prompt) | Safety net | Model wastes tokens generating then discarded |
| **Soft + hard (today)** | Best of both | Truncate still order-biased |
| Soft + hard + **early-stop delimiter** | Saves tokens if model complies | Text-mode sensitive; JSON mode different |

**Consensus:** Soft+hard stays. Improve *what* hard keeps (Axis C), not remove hard.

### Axis B — What the number means

| Semantics | Meaning | When useful |
|-----------|---------|-------------|
| **Absolute K** (today) | ≤40 ents / ≤100 rows every response | Fairness, ops simplicity |
| **Proportional K(S)** | \(K \propto\) chunk token size | Large Fixed chunks get more budget; small adaptive chunks get less |
| **Density target** | Aim for ents/1k tokens | Research tuning; harder to explain |
| **Confidence threshold** | Keep ents with score ≥ τ | Needs model calibration; rare in GraphRAG extract |
| **Type quotas** | e.g. ≤10 PERSON, ≤15 CONCEPT | Domain schemas (SPEC-114); fights “CONCEPT spam” |
| **Global workspace budget** | Cap unique U growth | Wrong layer — kills multi-doc graphs; don’t |

**Consensus:** Keep absolute \(K\) as default (LR parity). Optional proportional or type quotas as **workspace policy** later — not fleet default change.

### Axis C — Selection under the budget (who survives truncate)

Today: **first \(K\) in model output order**.

| Strategy | Idea | Risk |
|----------|------|------|
| C0 FIFO (today) | Truncate list head | Late important ents die |
| C1 Prompt rank | Ask model to list **most central** first | Soft; model may ignore |
| C2 Post-score | Embed chunk↔entity; keep top-\(K\) by similarity / degree heuristic | Extra compute; heuristic bias |
| C3 Two-pass | Pass1 extract unbounded (or high K) → Pass2 rank/select ≤K | 2× cost |
| C4 Schema filter first | Drop OTHER / low-value types before truncate | Needs good types |
| C5 Relation-aware | Prefer entities that participate in ≥1 high-value relation | Keeps bridges; good for multi-hop |
| C6 Gleaning-aware | If truncated, set flag → force gleaning with “continue from …” | Recovers misses; +latency |

**Consensus for next evolution:** C1 (prompt: emit highest-value first) + C6 (truncation → gleaning hint) cheap; C5 if we add scores; avoid C3 by default (cost).

### Axis D — Coupling to chunk geometry (SPEC-116)

```ascii
  If S_chunk large and content dense:
      K=40 may under-cover  → options: Fixed smaller S, ↑K, gleaning
  If S_chunk small (adaptive 600–800):
      K=40 often over-budget room → model may still emit ~20–40
      N↑ × saturate → M↑ (product denser than LR)

  Co-design rule:
      useful_capacity ≈ K
      target: expected_true_ents(S) ≲ K   OR   gleaning≥1
```

**Consensus:** Acc-fair Fixed 1200/100 + K=40/100 is the **known-good pair** for LR compare. Adaptive + K=40 is intentional product densification risk — document it; don’t “fix” by raising K.

### Axis E — Coupling to LLM power ([10](10-llm-power-first-principles.md))

| Model behavior | Cap effect |
|----------------|------------|
| Weak / noisy | Cap limits junk volume (good) but may cut rare true ents |
| Strong, high-y | Hits ceiling often; **which** 40 matter (C1/C5) |
| Local unconstrained | Cap + max_tokens prevents pipeline death (LAW-B1) |
| Constrained JSON | Cap still needed; schema validity ≠ cardinality control |

### Axis F — Relations separately?

Today: entities first, then relationships until total rows ≤100.

| Idea | Note |
|------|------|
| Separate `max_relations` | Clearer than “100 − ents”; LR uses combined records |
| Prefer bridge relations | Multi-hop research; needs scoring |
| Ban orphan rels | Already done after entity truncate |

**Consensus:** Keep combined total for LR parity; optional `max_relations` as explicit workspace field later.

---

## 2. Decision matrix (should we limit?)

| Stakeholder goal | Limit budget? | Prefer |
|------------------|---------------|--------|
| Dual-SUT / Acc fair vs LightRAG | **Yes, keep 40/100** | Matched K |
| Stop local LLM timeouts | **Yes** | Soft+hard + max_tokens |
| Maximize mention vanity M | No / raise K | **Reject** as product goal |
| Maximize multi-hop QA | Yes, but **select better under K** | C1/C5 + Acc-fair N + schema |
| Dense medical chunk coverage | Maybe ↑K **or** ↓S **or** gleaning | Measure truncation rate first |
| Cost control | **Yes** | Keep K; don’t raise casually |

**Overall answer:** Limiting per-chunk budget is **good and should remain**. The improvement surface is **selection + observability + optional workspace policy**, not abolishing caps.

---

## 3. Recommended design (phased)

### Phase 0 — Keep (done)

- Soft prompt + hard 40/100  
- Env overrides  
- Metadata `extract_caps_applied` when truncated  

### Phase 1 — Observability (docs → small code later)

Emit / surface:

- `%` chunks with `extract_caps_applied`  
- `entities_before` distribution (p50/p95)  
- Correlation with chunk token size  

**Decision rule:** If p95 `entities_before` ≫ 40 on Acc-fair geometry → coverage risk → gleaning or K or S.  
If p95 ≈ 15–25 → budget is loose; densification is \(N\) or \(y\), not K.

### Phase 2 — Prompt ranking (low risk)

Extend quantity section:

```text
- Prefer central, repeatedly referenced, and relation-bearing entities.
- Emit highest-value entities first (truncation keeps the head of the list).
- Do not invent filler entities to approach the limit.
```

No Acc pin change; improves C0 bias without new knobs.

### Phase 3 — Truncation → gleaning hint (medium)

If `extract_caps_applied` and gleaning remaining:

- Continue prompt: “Previous response hit the entity limit; extract **additional** high-value entities not already listed: …”  
- Own budget K again (union + merge)

Aligns with LightRAG gleaning + continue semantics.

### Phase 4 — Workspace extract budget (product, optional)

Mirror SPEC-116 pattern (metadata only):

| Field | Values |
|-------|--------|
| `extract_max_entities` | null = inherit env/default 40 |
| `extract_max_records` | null = inherit 100 |
| Preset | “Match LightRAG (40/100)” |

**Do not** ship UI until Phase 1 metrics exist. Default remains inherit.

### Phase 5 — Smart truncate (research)

Keep top-K by:

1. Appears in ≥1 retained relationship, then  
2. Prompt order / centrality heuristic  

Still hard-capped; better bridges for multi-hop.

### Explicit non-goals

- Removing hard caps  
- Raising fleet default above 40/100 without Acc re-measure  
- Global “max entities per workspace”  
- Using K as a substitute for Acc-fair chunking  

---

## 4. Implementation sketch (when coding)

```ascii
  extract_caps.rs (SSOT)
       │
       ├─ ExtractionCaps::from_env()          // today
       ├─ ExtractionCaps::from_workspace_meta // Phase 4
       ├─ prompt_quantity_limits_section()    // Phase 2 text
       ├─ apply_extraction_caps()             // Phase 5 selection
       └─ truncation_stats → chunk metadata   // Phase 1

  workspace metadata (Phase 4)
       extract_max_entities / extract_max_records
       apply_* helper like apply_chunking_metadata

  gleaning (Phase 3)
       if caps_applied && gleaning_left → continue_missed_prompt
```

**Tests:**

- Unit: truncate order, orphan rel drop, total row cap (exist)  
- Contract: env override; workspace override precedence  
- E2E: Acc-fair geometry + count `%` truncated on gold MD  
- Acc gate: any K change requires SPEC-001 smoke / peer honesty  

**Precedence (if Phase 4):**

```text
document override (future) > workspace extract_* > fleet env > 40/100 defaults
```

Same shape as ChunkingPolicy LAW-116-2.

---

## 5. Experiment protocol (before changing defaults)

```text
Sample: SPEC-115 gold MD + Acc medical slice
Geometry: Fixed 1200/100 (Acc fair)
Arms:
  K40  — current
  K60  — raised
  K40+rank — Phase 2 prompt only
  K40+glean2 — more gleaning, same K
Report: N, M, U, %truncated, Acc / multi-hop subset, \$/doc
Reject arm if Acc↓ or U noise↑ without QA↑
```

---

## 6. Brainstorm leftovers (parked)

- Adaptive K by document domain (legal vs chat logs)  
- Separate vision/table extract budgets (dense tables)  
- Train small ranker for C2  
- Confidence from logprobs if provider exposes them  
- DEG-RAG-style post-graph denoising as complement to K (better “less is more”)

---

## 7. One-page conclusion

```ascii
  Is limiting Nb entities/relations per chunk a good idea?
      YES — for reliability, cost, LR parity, crude anti-junk

  Is today’s 40/100 + FIFO truncate perfect?
      NO — order bias; must co-design with N and gleaning

  First product move if unhappy with density/QA?
      1) Acc-fair geometry (SPEC-116)
      2) Measure truncation rate
      3) Prompt rank + gleaning
      4) Only then workspace K or model upsize

  Never: raise K to inflate card M
```
