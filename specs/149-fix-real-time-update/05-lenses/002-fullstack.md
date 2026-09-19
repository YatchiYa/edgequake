# Lens 002 — Fullstack

See parent summary in [001-product-owner.md](001-product-owner.md) section "Fullstack" and [04-target-architecture.md](../04-target-architecture.md).

### Module boundaries

| Module | Owns |
|--------|------|
| `websocket-manager.ts` | Singleton + URL resolve + reset on session |
| `progress-websocket.ts` | Transport, backoff, desiredSubs, generation |
| `progress-event-normalizer.ts` | Wire → domain events |
| `websocket.rs` | Upgrade, commands, filter |
| `websocket_types.rs` | DTOs + broadcaster |
| `stream-client.ts` | Authenticated SSE frames |

### Dependencies to avoid

- Hooks calling `new WebSocket` directly
- Second EventSource factory in documents.ts / document-core.ts
- Ad-hoc JSON field remaps in cache helpers
