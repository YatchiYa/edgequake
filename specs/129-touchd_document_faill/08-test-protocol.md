# 08 — Test protocol

## T1 — Unit (always)

`document_shell` / `documents_column_status` tests:

| Input | `normalize` | `relational_documents_status_for_write` |
|-------|-------------|----------------------------------------|
| `re_embedding` | `processing` | `processing` |
| `completed` | `completed` | `indexed` |
| `queued` | `pending` | `pending` |
| `merging` | `processing` | `processing` |
| `deleting` | `deleting` | `deleting` |
| `""` | `processing` | `processing` |

## T2 — Postgres e2e (`e2e_spec129_touch_status_check`)

Pattern: `e2e_spec098_delete_sql_status_constraint.rs`

1. Insert doc `status='failed'`.
2. Raw `UPDATE ... 're_embedding'` → expect error containing check / `documents_valid_status`.
3. `PostgresPdfStorage::touch_document_status(id, "re_embedding")` → Ok; SELECT status = `processing`.
4. `touch(..., "queued")` → `pending`.
5. `touch(..., "merging")` → `processing`.
6. `touch(..., "deleting")` → `deleting`.
7. `touch(..., "completed")` → `indexed`.

Skip if `DATABASE_URL` unset.

## T3 — Source contracts

- `extraction.rs` still contains `"re_embedding"` (honesty).
- Touch impls contain `relational_documents_status_for_write` (or import of it).
- No bare `else { status }` pass-through in touch without normalize.

## T4 — Optional live

Reprocess crash-checkpoint doc; grep logs: no `touch_document_status failed` WARN.

## Commands

```bash
cargo test -p edgequake-storage --lib relational_documents_status
cargo test -p edgequake-api --test e2e_spec129_touch_status_check
cargo test -p edgequake-api --lib re_embedding_stage_string_is_honest
```

## Cross-refs

- Acceptance: [09-acceptance.md](09-acceptance.md)
- Edges: [10-edge-cases.md](10-edge-cases.md)
