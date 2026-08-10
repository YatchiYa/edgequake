# 04 — E2E test matrix (SPEC-114)

## Gates

| Gate ID | Layer | Command / Spec | Findings |
|---------|-------|----------------|----------|
| G-114-01 | Rust unit | `cargo test -p edgequake-core --lib normalize_type_list` | F-114-06 |
| G-114-02 | Rust unit | `cargo test -p edgequake-core --lib apply_relation_types` | F-114-01 |
| G-114-03 | Rust unit | `cargo test -p edgequake-pipeline --lib enforce_relation` | F-114-02 |
| G-114-04 | Rust API e2e | `cargo test -p edgequake-api --test e2e_spec114_relation_types` | F-114-01, F-114-10 |
| G-114-05 | FE unit | `bun test` presets / normalize / preview | F-114-03, F-114-04 |
| G-114-06 | Playwright | `e2e/spec114-kg-schema.spec.ts` dual panels + preview | F-114-05 |
| G-114-07 | Playwright | reconfigure persist relation types | F-114-07 |
| G-114-08 | Playwright | workspace card shows relations | F-114-08 |
| G-114-09 | Playwright | empty relations = free-form copy / no forced list | F-114-02/EC |
| G-114-10 | Non-regress | spec101 / spec096 / spec102 / entity selector | inheritance |
| G-114-11 | Rust unit | `normalize_relation_edges` / apply metadata | F-114-13 |
| G-114-12 | Rust unit | `enforce_relation_edge` empty/strict/permissive | F-114-13 |
| G-114-13 | Playwright | typed edge add/persist/lens; honest preview | F-114-12/14 |
| G-114-14 | Rust API e2e | `e2e_spec114_relation_edges` | F-114-13 |
| G-114-15 | Rust API mock ingest | `cargo test -p edgequake-api --test e2e_spec114_extraction_schema -- --test-threads=1` | Dual allowlists + typed edges on ingest → graph |
| G-114-16 | Pipeline e2e | `cargo test -p edgequake-pipeline --test e2e_spec114_gleaning_relations` | Gleaning enforces relation types + edges (mirror #276) |
| G-114-17 | Rust live Mistral | `make spec114-e2e-mistral-extract` (`#[ignore]`) | Soft EC matrix → allow-listed entity + relation labels |
| G-114-18 | Playwright | `e2e/spec114-kg-schema.spec.ts` edge cases + optional live smoke | EC chips/strict/clear/lens + soft live ingest |
| G-114-19 | Rust live Ollama | `make spec114-e2e-ollama-extract` (`qwen3.6:35b-a3b`, `#[ignore]`) | Same soft EC matrix as G-114-17 on local Ollama |

## Playwright scenarios (spec114-kg-schema)

1. Open reconfigure → Extraction → see entity panel + relation panel + preview.  
2. Select Manufacturing domain → both lists populated → preview updates.  
3. Add custom relation → chip appears → Apply → reload shows same.  
4. Clear relation types → server free-form (absent/empty).  
5. Toggle relation strict → persists.  
6. Language change remaps entity preset; relations unchanged.  
7. Review step lists relation_types in diff when changed.  
8. Max-50: UI blocks 51st relation type.
9. Typed-edge lens + remove entity chip drops dependent edges (EC-114-20/21).
10. Optional live smoke (skip unless `MISTRAL_API_KEY`): ingest short text → graph relation ∈ allow-list.

## Extraction / ingest scenarios (G-114-15…17)

1. Happy path — `PERSON`/`ORGANIZATION` + `WORKS_AT` + typed edge present in graph (mock).  
2. Strict relation remap — mock `EMPLOYS` → `RELATED_TO`/`WORKS_AT` (EC-114-02).  
3. Permissive pass-through — unknown relation kept (EC-114-03).  
4. Typed-edge violation — reversed endpoints remapped (EC-114-16/18).  
5. Empty relations — free-form label preserved (EC-114-01).  
6. Entity OTHER + relation still extracted under dual allowlists.  
7. Live Mistral / Ollama soft EC matrix (`make spec114-e2e-live-extract`): happy, free-form, strict closed-world, permissive, typed-edge, entity-OTHER — all graph labels ⊆ allow-list when strict.

## Rust API scenarios

1. PUT `relation_types` → GET round-trip normalized.  
2. PUT empty array clears key.  
3. PUT >50 silently caps at 50.  
4. `relation_types_strict: false` stored; `true` sparse-remove.  
5. `kg_schema_preset` round-trip; invalid/empty clears.  
6. Entity types path unchanged (non-regress #216).

## Definition of done

Every row in [02-cross-ref-matrix.md](02-cross-ref-matrix.md) has a green gate in CI or local `make`/`cargo`/`playwright` as documented in measurements when captured.
