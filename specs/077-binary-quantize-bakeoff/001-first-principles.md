# SPEC-077 — First principles (binary quantize + rerank)

## Why binary quantization

`binary_quantize(embedding)` maps each dimension to 1 bit (sign). Indexed as `bit(D)` with **Hamming** distance (`<~>` / `bit_hamming_ops`):

- Index footprint ≪ halfvec HNSW (order ~1/30 vs float32)
- Recall drops unless you **oversample** candidates then re-rank with the original embedding

Official pgvector pattern (README):

```sql
-- Index (expression)
CREATE INDEX ON items USING hnsw (
  (binary_quantize(embedding)::bit(D)) bit_hamming_ops
);

-- Query: binary candidates → exact reorder
SELECT * FROM (
  SELECT * FROM items
  ORDER BY binary_quantize(embedding)::bit(D)
        <~> binary_quantize($q::halfvec)::bit(D)
  LIMIT candidate_k
) t
ORDER BY embedding <=> $q::halfvec
LIMIT top_k;
```

## EdgeQuake placement

| Layer | Role |
|-------|------|
| Wave-2 halfvec + partial HNSW | **Supported default** @100k |
| Exact reorder (SPEC-076) | Opt-in re-rank on same-type ANN candidates |
| Binary + rerank (this pack) | Opt-in **study** for huge shared tables / RAM cliffs — not default |

Filter law still applies: bake-off measures **workspace-filtered** recall@20. Unfiltered demos are not a promote path.

## Env (harness / future opt-in)

| Env | Default | Meaning |
|-----|---------|---------|
| `EDGEQUAKE_BINARY_QUANTIZE` | off | Study/harness enable |
| `EDGEQUAKE_BINARY_CANDIDATE_K` | `200` | Inner Hamming LIMIT (≫ top_k) |

## Honesty

- Smoke N archives directionality; **does not** raise floors.
- Do not silent-flip existing DBs to binary expression indexes.
