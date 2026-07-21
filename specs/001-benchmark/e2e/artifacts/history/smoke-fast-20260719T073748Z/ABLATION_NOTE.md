# P3 fusion ablation — INVALID Acc

Attempted `EDGEQUAKE_MIX_FUSION=round_robin` on warm workspace `ba945742-…`
after P0 n=40. Restart re-entered the insert task; saga compensation rolled
back vectors (`knowledge-graph merge error(s) during persist`). EQ answers
empty → Acc invalid.

**Decision:** keep production default `EDGEQUAKE_MIX_FUSION=rrf`.
Code path + `contract_rrf_fusion` cover the ablation switch for a future
clean query-only run.
