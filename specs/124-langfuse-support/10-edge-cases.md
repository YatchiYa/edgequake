# 10 — Edge Cases

| ID | Case | Mitigation | Test |
|----|------|------------|------|
| E1 | No Langfuse env | Exporter off; UI unsatisfied; app healthy | unit + API |
| E2 | Only one of pk/sk | `enabled=false`; requirements show which missing | unit |
| E3 | Keys set, binary without `otel` | Boot warn; DTO `otel_feature_built=false`; no Open | unit/API |
| E4 | Invalid keys / HTTP 401 | Batch export fails soft; request OK; log warn | manual/live |
| E5 | Langfuse timeout / down | Non-blocking batch; timeout on exporter client | design + log |
| E6 | Dual Jaeger + Langfuse | Both processors registered | unit config assert |
| E7 | `LANGFUSE_HOST` vs `BASE_URL` | Prefer BASE_URL; fallback HOST; default EU cloud | unit |
| E8 | Trailing slash on base URL | Normalize before join `/api/public/otel` | unit |
| E9 | PII / secrets in span I/O | Explicit I/O; `query_preview` truncate; never log keys | unit |
| E10 | Shutdown without flush | `ObservabilityGuard` shutdown | drop path |
| E11 | Attribute propagation missing | Set on HTTP/root and children helpers | code review + live |
| E12 | Open link when not configured | Button not rendered | Playwright |
| E13 | Open link when configured | `href === ui_url` | Playwright/API |
| E14 | Per-trace link without trace_id | No button | UI |
| E15 | Mock LLM provider | Spans emit with model/provider `mock` | unit/integration |
| E16 | Acc / high QPS | Batch processor; EnvFilter bounds volume | load note |
| E17 | Self-hosted Langfuse HTTP | `LANGFUSE_BASE_URL=http://localhost:3000` | unit URL |
| E18 | US / JP / HIPAA regions | Base URL from env only | docs |
| E19 | EDGEQUAKE_LANGFUSE_ENABLED=0 with keys | Force disabled | unit |
| E20 | Concurrent init | Single `init_observability` at boot | existing pattern |
| E21 | Missing conversation_id on chat create-path | New UUID assigned before bind — always has session | unit/live |
| E22 | `/query` without `session_id` | No session attrs (LAW-124-11) | unit |
| E23 | Quoted / blank `session_id` on `/query` | Trim + unquote; empty → unset | unit |
| E24 | Stream spawn loses baggage | `with_langfuse_identity_async` in spawn | e2e sessions |
| E25 | Cost attrs on spans | `COST_ATTR_DENYLIST` + in-memory exporter assert absence | unit CI |
| E26 | Stream answer missing generation | `with_rag_generation_span` on stream path + contract grep | unit CI |
| E27 | Observation Input/Output null in Langfuse UI | Set `langfuse.observation.input`/`output` (LAW-124-16..18) | InMemory CI |
| E28 | Langfuse 3.1.1 OTLP 404 | `EDGEQUAKE_LANGFUSE_API=auto` → ingestion; 404-only probe | live `make spec124-langfuse-3.1-e2e` |
| E29 | `{retriever,embedding,chain}-create` envelopes | `langfuse_v31_envelope_type` SSOT → span-create | unit + live 3.1.1 |
| E30 | HTTP 207 with `errors[]` treated as success | `ingestion_http_outcome` fails the batch | unit |
| E31 | `--force` price sync still POSTs | PUT `/api/public/models/{id}` | script |

## Cross-refs

- Tests: [08-test-protocol.md](08-test-protocol.md)
- Sessions: [12-sessions-and-genai.md](12-sessions-and-genai.md)
- Tokens: [13-metadata-tokens-and-coverage.md](13-metadata-tokens-and-coverage.md)
- Observation I/O: [14-observation-io-and-full-observe.md](14-observation-io-and-full-observe.md)
- Observability lens: [05-lenses/008-observability.md](05-lenses/008-observability.md)
