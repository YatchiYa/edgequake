# 09 — Edge Cases

| ID | Case | Expected |
|----|------|----------|
| EC-149-01 | Auth disabled | Connect without token; unscoped session may see global Job* events |
| EC-149-02 | Token refresh mid-session | Reset client; reconnect; replay subs |
| EC-149-03 | Logout | Disconnect; clear desiredSubs; no further connect until login |
| EC-149-04 | Subscribe before OPEN | Queue command; flush on open; also keep in desiredSubs |
| EC-149-05 | Subscribe unknown track | No leak; not added to set; no error frame required |
| EC-149-06 | Cap exceeded | Reject additional ids; keep existing |
| EC-149-07 | Client lagged | Warn Message event; polling/safety-net recovers |
| EC-149-08 | Clean close (1000) | Do not auto-reconnect |
| EC-149-09 | Dirty close | Backoff reconnect with fresh URL |
| EC-149-10 | Concurrent connect() | Second call no-ops while CONNECTING |
| EC-149-11 | Stale onclose after newer connect | Ignored via generation |
| EC-149-12 | ProgressSnapshot on per-track | Normalized or handled; not "unknown" warn spam |
| EC-149-13 | GraphStorageProgress | Emitted as progress; pdf hook consumes normalized |
| EC-149-14 | SSE abort on unmount | AbortSignal cancels fetch; no setState after unmount |
| EC-149-15 | Multi-replica API | Events may miss (known limitation); single replica ops default |

## Cross-refs

- Architecture: [04-target-architecture.md](04-target-architecture.md)
- Honest: [11-honest-assessment.md](11-honest-assessment.md)
