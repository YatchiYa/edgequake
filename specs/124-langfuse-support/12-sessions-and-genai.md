# 12 — Sessions & OTEL GenAI

> Binding doc for Langfuse **Sessions** (Observability → Sessions). Complements [04-target-architecture.md](04-target-architecture.md).

## Why Sessions was empty

Traces exported successfully, but no span set:

- `langfuse.session.id` / `session.id` (Langfuse mapper → `sessionId`)
- `gen_ai.conversation.id` (OTEL GenAI v1.37 conversation/thread id)

Without those attributes, Langfuse cannot group turns into a Session.

## Session identity (product SSOT)

| Source | Role |
|--------|------|
| Chat `conversation_id` (UUID) | **Session id** — durable, sent by WebUI every turn |
| Auth `user_id` | Langfuse `userId` |
| Tenant / workspace | `langfuse.trace.metadata.*` only |
| `/query` without `session_id` | **No session attrs** (never invent) |
| Optional `/query` `session_id` | Client-provided durable id only |

**LAW-124-11 — No synthetic conversation ids.** Do not use `request_id`, OTEL TraceId, workspace id, or content hashes as `gen_ai.conversation.id` / Langfuse session when the product has no conversation. OTEL GenAI forbids inventing conversation ids.

## Attribute map

| Concern | Attributes (same value where noted) |
|---------|--------------------------------------|
| Session | `langfuse.session.id`, `session.id`, `gen_ai.conversation.id` |
| User | `langfuse.user.id`, `user.id` |
| Tenant | `langfuse.trace.metadata.tenant_id` |
| Workspace | `langfuse.trace.metadata.workspace_id` |

## Propagation (DRY)

```ascii
  chat resolve conversation_id
       │
       ├─ bind_langfuse_identity → stamp current OTEL span (HTTP / chat_stream)
       └─ with_langfuse_identity_async (stream spawn)
              │
              └─ LangfuseBaggageSpanProcessor.on_start
                     → copy allowlisted baggage → child span attrs
```

Ownership (SRP):

| Module | Role |
|--------|------|
| `langfuse_attrs.rs` | Keys + `LangfuseTraceIdentity` |
| `baggage_span_processor.rs` | Allowlisted baggage → attributes |
| `langfuse_context.rs` | Bind / async scope helpers |
| Chat / query handlers | Call bind after identity is known |

## Verify in Langfuse UI

1. Ensure `LANGFUSE_*` keys + `export_active` (Settings card).
2. Send **two** chat turns with the **same** `conversation_id`.
3. Wait ~5–15s for batch export.
4. Open **Observability → Sessions** — session id = conversation UUID.
5. Open the session — expect ≥2 traces (`HTTP` / chat) with retrieval + generation.

API check (optional):

```bash
# Basic auth = base64(pk:sk)
curl -sS -u "$LANGFUSE_PUBLIC_KEY:$LANGFUSE_SECRET_KEY" \
  "$LANGFUSE_BASE_URL/api/public/sessions?limit=10"
```

## References

- [Langfuse OTEL property mapping](https://langfuse.com/docs/opentelemetry/get-started) — `sessionId` via `langfuse.session.id` / `session.id`; propagate to all spans
- [OTEL GenAI spans — `gen_ai.conversation.id`](https://github.com/open-telemetry/semantic-conventions/blob/main/docs/gen-ai/gen-ai-spans.md) — conditionally required; do not invent
- SPEC-124 laws: [01-first-principles.md](01-first-principles.md)
- E2E: `edgequake_webui/e2e/spec124-langfuse-sessions.spec.ts`

## Cross-refs

- Architecture: [04-target-architecture.md](04-target-architecture.md)
- Tests: [08-test-protocol.md](08-test-protocol.md)
- Acceptance: [09-acceptance.md](09-acceptance.md)
- Edges: [10-edge-cases.md](10-edge-cases.md)
