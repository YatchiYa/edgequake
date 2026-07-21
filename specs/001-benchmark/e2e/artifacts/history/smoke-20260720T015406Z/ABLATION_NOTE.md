# Ablation — P3b_keyword_lexical_boost_v1

**Step:** p3b  
**Pins:** BM25 Acc + `KEYWORD_LEXICAL_BOOST=1` + `POPULAR_NODE_FALLBACK=0`  
**Workspace:** `8b359190-0733-4949-994c-f39eca074d79`  
**Archive:** `smoke-20260720T015406Z`

## Results

| Metric | Value | Gate | Result |
|--------|-------|------|--------|
| EQ Acc | 0.713 | audit | ~P3a (BM25 tax) |
| Summarize evidence_recall EQ/LR | 0.900 / 0.983 | ≥0.95 or ≥LR−0.03 | **miss** (0.900) |
| EQ ctx_rel | 0.375 | — | BM25 |
| keyword_lexical_boost | true | labeled | ✅ |
| popular_node_fallback | false | labeled | ✅ |

## Verdict

- [ ] Gate met
- [x] Gate missed — lexical boost alone on BM25 regresses Summarize recall vs P3a (0.950)

**Note:** Do not Acc-promote lexical alone. Re-evaluate inside P4 S1+gw+retrieval package (P2b already cleared Complex + ctx_rel 0.50).
