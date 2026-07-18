# SPEC-080 — First principles (tiny-slice exact)

pgvector 0.8 improved cost estimation so small tables often prefer **exact** (seq/btree) over HNSW — 100% recall, often faster.

Wave-2 planner bias (`SET LOCAL enable_seqscan = off`) was added (SPEC-067) to keep filtered ANN on partial HNSW at mid-scale. On **tiny** hot workspaces that bias over-forces HNSW.

**Fix:** when `count(workspace) ≤ EDGEQUAKE_ANN_EXACT_MAX_ROWS` (default 2000), skip bias statements and let the planner choose.

Does not disable HNSW indexes; does not raise floors; does not silent-flip Wave-2 defaults.
