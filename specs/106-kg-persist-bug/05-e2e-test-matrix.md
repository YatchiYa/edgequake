# 05 — E2E Test Matrix (SPEC-106)

| ID | Case | Harness | Status |
|----|------|---------|--------|
| E2E-106-01 | Upsert 2 nodes + edge → `get_edges_for_nodes_batch` returns edge (no 42883) | `e2e_spec106_graphid_edges_batch` | Required (soft-skip without DB) |
| E2E-106-02 | Source guard: no raw `src.vid = e.start_id` | same file `#[test]` | Always |
| E2E-106-03 | Empty `node_ids` → `[]` | same file | Soft-skip without DB |

```bash
export DATABASE_URL="$(cat /tmp/edgequake-db-url)"
# or: postgresql://edgequake:edgequake_secret@localhost:5432/edgequake
cargo test -p edgequake-storage --features postgres --test e2e_spec106_graphid_edges_batch
```
