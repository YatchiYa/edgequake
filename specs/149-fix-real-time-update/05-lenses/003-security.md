# Lens 003 — Security

## Invariants

1. Upgrade requires Origin allow-list when fail-closed (prod).
2. Upgrade requires valid JWT/API key when `auth_enabled`.
3. Track events leave the server only after ownership check at subscribe time (and optional re-check on sensitive cancel).
4. No timing/oracle: rejected subscriptions do not name foreign track ids.

## Threat notes

| Threat | Mitigation |
|--------|------------|
| Cross-tenant progress sniffing | HashSet only contains owned tracks |
| Token in query logs | Existing redaction / short TTL JWT; future ticket for first-message auth |
| Origin spoofing | Exact string match CORS list |
| Subscribe spam | Cap set size (e.g. 256); idempotent add |

## Explicit non-changes

- Do not disable auth for WS "to make demo work"
- Do not broadcast all workspace events without subscribe (noise + leak surface)
