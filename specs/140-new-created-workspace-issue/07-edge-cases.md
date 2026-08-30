# 07 — Edge cases

> **Cross-refs**: [Fix](05-fix-plan.md) · [E2E](06-e2e-test-matrix.md)

| ID | Scenario | Mitigation | Test |
|----|----------|------------|------|
| EC-140-01 | n=0 workspaces | Empty copy; no auto-select crash | existing guard + 01 empty tenant not required |
| EC-140-02 | n=20 exactly | `items.len()==20`, `total==20`, no second request needed | 01 can assert `offset+len>=total` |
| EC-140-03 | n=21 | Default page 20, `total=21`; client fetches page 2 | E2E-140-01 / 04 |
| EC-140-04 | n=100 | One client page (`limit=100`) | fetchAllPages unit |
| EC-140-05 | n=101 | Two client pages | unit + HTTP `?limit=100` then offset=100 |
| EC-140-06 | Default Workspace + 3 named | All four in popover | E2E-140-03 (Default may exist) |
| EC-140-07 | 3 tenants × 1 workspace | Org list complete; workspace group follows selected org | E2E-140-05 |
| EC-140-08 | cmdk leftover search “73” | Remount Command on open | E2E-140-03 clears / remount |
| EC-140-09 | Quota Free (10) | 25-create uses `plan=pro` | E2E-140-01 |
| EC-140-10 | `is_active=false` | Still listed (current SQL); do not newly hide | no filter added |
| EC-140-11 | Missing `id` in JSON | Skip in merge; do not `Map.set(undefined)` | U-140-MERGE |
| EC-140-12 | `offset` past end | `items=[]`, `total` still COUNT | HTTP 01 `?offset=1000` |
| EC-140-13 | `limit=0` / missing | Default 20 via serde | handler default |
| EC-140-14 | `limit=1000` | Cap 100 | handler `.min(100)` |
| EC-140-15 | TenantProvider vs header | Same merge helper | code DRY |
| EC-140-16 | Optimistic create then stale 20 | Merge keeps extra id until refetch complete | merge unit |
| EC-140-17 | Wrong `X-Workspace-ID` header | List ignores it (tenant path param) | existing |
| EC-140-18 | Inactive tenant | Still listed if in `tenants` table | no new filter |
| EC-140-19 | Duplicate display names, distinct UUIDs | cmdk `value=id` | cmdk + unit |
| EC-140-20 | HashMap order (in-memory tenants) | `total` from COUNT/len, not order | E2E-140-02 |

## Not mitigated here

| Case | Follow-up |
|------|----------|
| User assigned only on another tenant | Switch org (Track C); no membership aggregator |
| >5000 workspaces | `fetchAllPages` safety cap 50×100; product should add typeahead later |
