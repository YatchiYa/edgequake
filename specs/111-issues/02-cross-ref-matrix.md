# 02 — Cross-ref matrix

| ID | Symptom | Code locus | Spec / issue | Law |
|----|---------|------------|--------------|-----|
| X-364-A | dry-run “legacy rows un-migrated” forever | `advisor/types.rs` `chunk_retirable` | #364, SPEC-091 W4 | LAW-111-2 |
| X-364-B | DROP zeros counts | `126_spec091_vector_drop.sql` DELETE | #364 | LAW-111-2 |
| X-364-C | verify rejects regenerate | `verify.rs` `1e-3` | #364 secondary, #363 workaround | LAW-111-4 |
| X-364-D | weak parity test | `e2e_spec091_vector_retire.rs` | #364 | LAW-111-3 |
| X-363-A | exact name join | `fleet_embedding_backfill.rs` | #363 | LAW-111-6 |
| X-363-B | scan≠write success | `runner.rs` / `lease.rs` processed+=scanned | #363 | LAW-111-4 |
| X-362-A | `id::text` | `advisor/residue.rs` | #362 | LAW-111-5 |
| X-362-B | same in 125 | `125_spec091_kv_drop.sql` | #362, LAW-C3 | LAW-111-3 |
| X-362-C | correct cast exists | residue chunk/wsdoc arms | #362 | DRY |
| X-366-A | list suffix-fallback on empty membership | `document_metadata_scan.rs` | #366, #360 | LAW-111-9 |
| X-366-B | wipe skipped residual KV | `workspace_document_wipe.rs` RM1 | #366, #360 | LAW-111-9 |
| X-366-C | membership authority type | `WorkspaceMetadataKeyList` | #366 | DRY / ISP |
| X-360-A | durable wipe ancestor | `workspace_document_wipe.rs`, #309 | #360, SPEC-050 | LAW-111-7 |
| X-360-B | UI await | `clear-documents-dialog.tsx` | #360 | — |
| X-361-A | concurrency caps | `pdf_processing.rs`, SPEC-090 | #361 | LAW-111-7 |
| X-110 | migrate 118 ON CONFLICT | SPEC-110 | same partners / migrate train | ship vehicle |

## Dependency graph

```ascii
  #363 coverage truth ──► #364 retirable (needs real coverage signal)
         │
         └──────────────► verify policy (copy vs regenerate)
  #362 advisor completable ──► dry-run can show KV GREEN/RED at all
  SPEC-110 (118) ──► partners reach drop waves where 362–364 matter
```
