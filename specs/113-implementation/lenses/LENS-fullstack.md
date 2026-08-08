# LENS — Full Stack Developer (SPEC-113)

## Path under study

```text
  UI / API  reasoning_effort? ──► role resolve (SPEC-109)
        │
        ▼
  edgequake-llm OllamaProvider
        │
        ├─ async CapabilityResolver  (/api/show | cache | tags warm)
        │
        ▼
  map_think(effort, ThinkingSupport) ──► ChatRequest.think
        │
        ▼
  POST /api/chat  ──►  stream/non-stream parse (thinking field optional)
```

## Gaps on 0.10.4 (code is law)

| Layer | Gap | Fix wave |
|-------|-----|----------|
| Auto think | Name substring SSOT | A |
| Sync `resolve_think` | Cannot await show | A (async pre-resolve) |
| Discovery | Caps parsed but unused by chat | C |
| Static clamp | `contains("qwen")` folklore | A4 / C |
| Cache | None | B |
| Error UX | Generic provider error | D (optional map message) |

## Implementation discipline

- **DRY:** one caps parser for discovery + chat.
- **SRP:** resolver ≠ wire mapper ≠ HTTP chat client.
- **Do not** sprinkle `contains("qwen3")` in EdgeQuake API crate.
- Prefer wiremock e2e in llm crate; EdgeQuake integration after dep bump.

## Local verify loop

```bash
cd ../edgequake-llm
cargo test ollama -- --nocapture
# after fix: assert VL fixture outbound JSON has no "think"
```

## Dependency boundary

```text
  edgequake (app)  ──depends──►  edgequake-llm 0.10.4
                                      │
                                      └── fix HERE, then bump pin
```
