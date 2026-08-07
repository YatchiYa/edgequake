# issue-366 — Clear All leaves documents (v0.24.1)

**GH:** https://github.com/raphaelmansuy/edgequake/issues/366  
**Sibling:** [#360](https://github.com/raphaelmansuy/edgequake/issues/360) (same symptom; reported as 0.12.11, partner clarified **0.24.1**)  
**Reported on:** **v0.24.1** Docker + PostgreSQL  
**Status on HEAD:** **Confirmed defect** — fixed in this pack (LAW-111-9 + residual KV purge)

## WHY

Users must trust that Clear All empties the document list after refresh. Partner confirmed leftovers on the current published pin, so this is not a historical-only ticket.

## First principles

```text
List ⊆ Wipe
```

Every store that can populate `GET /documents` must be emptied (or ignored as non-authoritative) when wipe completes.

On v0.24.1 the dual-read path violated that:

| Surface | Wipe (RM1) | List |
|---------|------------|------|
| `public.documents` | set DELETE | membership SSOT (`wsdoc` relational) |
| Legacy `eq_*_kv` `*-metadata` | **skipped** | **suffix-scan fallback when membership empty** |

After wipe, membership returns `[]` (authoritative empty). List treated empty as “no index” and fell through to a global `-metadata` suffix scan → dual-write residue reappeared as “some documents remain”.

This is the classic [dual-write delete gap](https://www.abstractalgorithms.dev/dual-write-problem-and-solutions): primary store deleted, secondary left, reader prefers secondary when primary is empty.

## Root cause (code is law)

1. `workspace_document_wipe.rs` — RM1 skipped `PurgingDocumentKv` / residual KV purge.
2. `document_metadata_scan::load_scoped_document_metadata_entries*` — `if !keys.is_empty()` then fetch; **else** `keys_with_suffix("-metadata")`.
3. `workspace_document_index::relational_workspace_doc_ids` returns `Some(vec![])` for an empty workspace — callers discarded the “answered” signal.

False-green e2e risk: seeding only via facade upsert (typed shell) then asserting list empty does not prove the KV-resurrect path.

## Definitive fix (DRY / SOLID)

| Principle | Application |
|-----------|-------------|
| SRP | Membership listing returns `WorkspaceMetadataKeyList { authoritative }` — one type, one meaning |
| OCP | New families extend membership ports; do not add more suffix fallbacks |
| LSP | `authoritative && keys.is_empty()` means empty workspace for **all** list readers |
| ISP | Wipe planner still may suffix-scan to **discover residue to delete** (different use-case) |
| DIP | List depends on membership authority, not on KV table presence |
| DRY | One `WorkspaceMetadataKeyList`; wipe reuses `plan_workspace_document_kv_deletion` |

1. **LAW-111-9** — authoritative empty membership is terminal for reads (no global KV suffix resurrect).
2. **Wipe** — after typed set-delete, purge residual KV list surfaces via existing planner (idempotent post-125).
3. **E2E** — wipe → list 0; plant raw `eq_*_kv` residue → list still 0.

## E2E

- `e2e_spec111_clear_all_list_empty_pg` (E2E-111-08 + #366 arm)
- Unit: `authoritative_empty_list_is_terminal_for_readers`

## Ops note

On v0.24.1 without the fix: Clear All can look complete (WS toast) while refresh still shows ghosts. Upgrade to the release that carries this pack, or re-run Clear All after upgrade (residual KV purge cleans leftovers).
