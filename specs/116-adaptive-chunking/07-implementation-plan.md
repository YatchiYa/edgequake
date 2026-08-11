# 07 — Implementation Plan

## Phases

1. **Pipeline SSOT** — `ChunkingPolicy`, resolve, `build_chunker_config`, unit tests  
2. **Core metadata** — apply helper, create/update, response fields  
3. **API worker** — inject policy from workspace  
4. **OpenAPI + FE types** — codegen / hand types  
5. **UI** — card + wizard + payload  
6. **Tests** — contract, geometry e2e, Playwright  

## Edge-case matrix

| Case | Expected |
|------|----------|
| Inherit + env ON | Adaptive thresholds |
| Inherit + env OFF | Env fixed sizes |
| Adaptive + env OFF | Still adaptive (workspace wins) |
| Fixed 1200/100 | Exact sizes |
| Fixed overlap ≥ size | 400 |
| Doc chunk_options | Overrides workspace base |
| Clear mode to inherit | Keys removed |
| WS A Fixed / WS B Inherit | Isolated |

## Definition of done

See [09-acceptance.md](09-acceptance.md).
