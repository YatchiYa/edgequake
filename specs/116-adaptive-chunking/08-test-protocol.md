# 08 — Test Protocol

## Unit (pipeline)

`cargo test -p edgequake-pipeline --lib adaptive_chunking`  
`cargo test -p edgequake-pipeline --test contract_spec025_adaptive_chunking`  
`cargo test -p edgequake-pipeline --lib chunking_policy` (new)

Matrix: Inherit/Adaptive/Fixed × env on/off × doc options last.

## API contract

Create/update/get workspace with chunking fields; 400 on bad overlap; clear → inherit.

## Geometry e2e (postgres or pipeline)

Two workspaces, same gold MD (~61KB): Fixed 1200 vs Adaptive → assert `chunk_count` Adaptive &gt; Fixed (expect ~16 vs ~12 on Recursive).

## Playwright

1. Open `/workspace`  
2. Edit → Acc-fair chip → Fixed 1200/100 visible  
3. Save → badge shows Fixed  
4. Future-only hint present  

## Honesty

Label geometry-only vs live-LLM arms; do not compare EQ card M to LR U without saying so.
