# 08 — Test Protocol

## Unit

| ID | Assert |
|----|--------|
| U1 | PDF resolver matrix: each layer wins over lower |
| U2 | Unset cascade → Vision, `allows_auto_route=false` |
| U3 | `auto` → `allows_auto_route=true` (when env routing on) |
| U4 | `should_try_edgeparse_before_vision` false for Vision/EdgeParse; true for Auto |
| U5 | FE PDF mirror matches Rust for same inputs |
| U6 | Fallback denied for resolved Vision |
| U7 | `resolve_llm_choice` request > workspace > tenant > env |
| U8 | `resolve_embedding_choice` workspace wins + dimension cascade |
| U9 | `resolve_vision_llm_choice` upload > ws vision > tenant vision > ws llm > env |
| U10 | FE `resolve-model-choice` mirrors U7–U9 |
| U11 | LAW-123-8: painted LLM fields without metadata → tenant/env wins |
| U12 | Vision Option unset + tenant vision → source=tenant |

## API e2e

| ID | Assert |
|----|--------|
| A1 | WS none + env unset + omit upload → lineage **vision** on born-digital |
| A2 | WS vision + env edgeparse → vision |
| A3 | Upload edgeparse + WS vision → edgeparse |
| A4 | Tenant edgeparse + WS none → edgeparse |
| A5 | WS auto + born-digital → may edgeparse; documented auto |
| A6 | N-file multi same as single per file |
| A7 | `/pdf/batch` same resolver as `/pdf` |

## WebUI e2e

| ID | Assert |
|----|--------|
| W1 | Settings Resolves to Vision → upload → detail Vision |
| W2 | Multi-file Workspace Default (Vision) → all Vision |
| W3 | Large admission EdgeParse does not rewrite non-large files |

## Regression

- Update `spec038_*` tests that expected implicit-Vision auto-route.
- Keep explicit EdgeParse / large admission happy paths.
- Acc extract/keyword env-first pins unchanged.
