# Acc #2 fig-as-chart — final smoke assessment (2026-07-15)

Artifact: `specs/047-rag-evaluation/e2e/artifacts/smoke-chart8-026-fig-as-chart-20260715-1707/`  
Tag: `chart8-026-fig-as-chart-20260715-1707` · Build: `20260715.090657` · Profile: `P0_mm_ite` · Protocol: `026-hardened-2026-07-15`

## Headline vs coexist (`…-1547`)

| Metric | Fig-as-chart | Coexist | Δ |
|---|---:|---:|---:|
| Acc | **0.562** | 0.563 | −0.001 |
| F1 | **0.480** | 0.457 | **+0.022** |
| Chart exclusive Acc | **0.286** | 0.143 | **+0.143** |
| Chart multi Acc | 0.318 | 0.318 | 0 |
| Chart `a_in_e_long` | **0.214 FAIL** | **0.214 FAIL** | **0** |
| Table `a_in_e_long` | 0.353 FAIL | 0.353 FAIL | 0 |
| page_hit@5 | 0.747 | 0.773 | −0.027 |

**Wave 1 claim: NO.** Gate Chart `a_in_e_long ≥ 0.50` unchanged. Acc alone is not a W1 win.

## Crop telemetry (lever worked operationally)

| Doc | Coexist wr | Acc #2 wr | promoted |
|---|---:|---:|---:|
| political | 0 | **12** | 12 |
| 2311 | 4 | **14** | 10 |
| PIP Seniors | — | **16** | 9 |

Fig-as-chart correctly promotes ink-empty fig assets → chart crops. Coverage ↑ does **not** move Chart long-needle fidelity.

## Causal read (why Chart gate flat)

1. **Fidelity Chart long outcomes are bit-identical** to coexist (0 differing rows). Promoting crops did not add any new long-needle Chart hits.
2. **ChartEx Acc 0.143→0.286 is W4 extract**, not W1: only flip is 2311 MMMU (`["MMMU"]`→`MMMU`). Same gold still `a_in_e_long=false` (needle `"mmmu"` not in page-4 MD).
3. **Political ChartEx still Acc=0**: page 5 now has the confidence **table** with both gold domain strings, but:
   - Specialize dump still leads with the **wrong** chart (ethics ratings).
   - Pred picks other domains; gold list fidelity needle is the whole list string → `a_in_e` false even when members are on-page (measurement gap for list gold).
4. **Hard Chart misses remain numeric/list facts not in MD**: `541`, `128`, pie years `1981…`, Indonesia operator lists, etc.
5. Paired Acc Δ vs coexist: `other_answerable +0.024`, `unanswerable −0.017`, `list_gold −0.008` — noise, not Chart representation.

## Vs crop-expand (`…-0535`)

Acc +0.056 / F1 +0.077 / ChartEx flat at 0.286 — stack (coexist+extract+fig-as-chart) beats crop-expand on Acc/F1; **Chart long gate still 0.214**.

## Next levers (do not claim W1 yet)

1. **Specialize densify / wrong-chart first**: gold-page residual ranking so ChartEx page gets the *correct* chart specialize, not ethics bar.
2. **Numeric chart OCR/table densify** for needles like 541/128.
3. **Optional**: list-gold fidelity = all members in evidence (diagnostic); do not loosen gate without protocol note.
4. Defer W3-quote until Chart `a_in_e_long` moves.

## Verdict

Fig-as-chart is **necessary plumbing proof** (wr 0→12 on political; 4→14 on 2311) and a modest **F1** bump, but **not** a Wave 1 Chart representation win. Next Acc only after a lever that changes Chart long-needle hits.
