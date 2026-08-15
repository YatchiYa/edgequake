# 08 — Test Protocol

## Layers

| Layer | What | Command / tool |
|-------|------|----------------|
| Unit | `LangfuseConfig`, auth header, endpoint URL, preview redact | `cargo test -p edgequake-observability --features otel` |
| Unit | Generation span helper runs | same |
| Unit | Session attrs + baggage allowlist + bind helpers | `cargo test -p edgequake-observability --lib langfuse_` |
| Unit | Usage helpers + cost denylist | `cargo test -p edgequake-observability --lib rag_span` / `langfuse_attrs` |
| Unit | Session deep-link href builder (WebUI) | `vitest` `langfuse-session-href.test.ts` |
| **CI always** | InMemorySpanExporter: usage ints + observation I/O + session bind + `with_llm_generation` | `cargo test -p edgequake-observability --lib inmemory_otel` |
| **CI always** | Stream path contract: source contains `with_rag_generation_span` | `cargo test -p edgequake-query --lib spec124_stream_genai` |
| **CI always** | Query pipeline meta + embed/rerank spans | `cargo test -p edgequake-query --lib spec124_pipeline_meta` |
| **CI always** | Gleaning wraps `with_llm_generation` | `cargo test -p edgequake-pipeline --lib gleaning_source_wraps_llm_generation` |
| **CI always** | Ingest chunking + KG meta | `cargo test -p edgequake-pipeline --lib spec124_ingest_stages` |
| **CI always** | PDF converting/pass_a/parse meta | `cargo test -p edgequake-api --lib spec124_ingest_converting` |
| **CI always** | Retrieval source sets `langfuse.observation.input` | `cargo test -p edgequake-observability --lib rag_span` |
| **CI always** | Aggregate | `make spec124-proof` |
| API | `/settings/langfuse` DTO shape; health fields | `cargo test -p edgequake-api` (+ contract if OpenAPI) |
| Playwright | Settings card states + Open link visibility | `edgequake_webui` e2e |
| Playwright | Sessions: two query turns → Langfuse session (when keys live) | `spec124-langfuse-sessions.spec.ts` |
| Optional live | Export + CLI fetch when `LANGFUSE_*` set | skip if unset — **not** sole gate |

## Required cases (map to [10-edge-cases.md](10-edge-cases.md))

1. No keys → `enabled=false`, no panic
2. Both keys + base → `enabled=true`, `ui_url` set
3. Only public key → disabled
4. Keys without otel feature (compile test / runtime warn path documented)
5. Deep link absent when disabled; present when enabled
6. `trace_id` field optional on query response
7. Query preview truncation does not panic on multibyte
8. Blank / missing session → no `gen_ai.conversation.id` (no synthesis)
9. Chat `conversation_id` co-emits Langfuse + GenAI session attrs
10. Two chat turns same conversation → Langfuse Sessions lists that id (live)
11. `record_gen_ai_usage` → exported span has input/output ints (in-memory)
12. Exported spans never contain cost denylist keys (E25)
13. Stream answer path instruments `with_rag_generation_span`
14. Retriever/generation/embedding/ingest export `langfuse.observation.input`/`output` (E27)
15. Gleaning LLM iterations use `with_llm_generation` / `extract-entities-glean`
16. `with_llm_generation` records usage + I/O without cost keys (in-memory)
17. Slugs co-emitted with tenant/workspace GUIDs; blank slug omitted (LAW-124-19)
18. `record_query_pipeline_meta` writes `langfuse.trace.metadata.mode` (and related) without cost keys
19. `record_ingest_parse_meta` writes parser/pass/pages on ingest span

## Playwright sketch

```typescript
test("langfuse settings card", async ({ page }) => {
  await page.goto("/settings");
  await expect(page.getByTestId("langfuse-settings-card")).toBeVisible();
  // Without keys: open link hidden
  await expect(page.getByTestId("langfuse-open-link")).toHaveCount(0);
});
```

## Live audit (when keys available)

```bash
# after query
npx langfuse-cli api traces list --limit 5
# compare to best-practices page (fetch fresh)
```

## Cross-refs

- Edges: [10-edge-cases.md](10-edge-cases.md)
- Acceptance: [09-acceptance.md](09-acceptance.md)
