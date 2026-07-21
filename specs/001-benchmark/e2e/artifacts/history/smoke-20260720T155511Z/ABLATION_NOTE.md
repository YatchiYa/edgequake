# Ablation — 052 REL_CHUNK_IDS_QUERY_PARITY + a1fp on B6

**Step:** a1fp on B6 after plural relation chunk ids at query  
**Workspace:** `58ffe7da-d181-4a31-8941-9621b051a678`  
**Archive:** `smoke-20260720T155511Z`

## Gates

| Gate | Target | Result |
|------|--------|--------|
| Acc | ≥ 0.781 (peer ≥ 0.801) | **0.759** ✗ (pre-052 B6 was 0.725) |
| ctx_rel | ≥ 0.50 | **0.506** ✓ |
| Fact ER | ≥ 0.83 | **0.80** ✗ |
| Complex Acc | (info) | **0.852** |

## Verdict

- [x] Query law closed (plural edge `source_chunk_ids` → Mix)
- [x] Acc still below promote — keep B5+`a1fp` Acc peer
- Keep code always-on (not a flag)
