# ISSUE — GH-350 Bulk Upload / Bulk Delete / agtype Persist

> **GitHub**: [#350](https://github.com/raphaelmansuy/edgequake/issues/350)  
> **Status**: Fixed + gated (SPEC-098)  
> **Laws**: LAW-098-6/9/10 + AGE schema-qualify defense-in-depth

## Symptoms (reporter)

1. Bulk upload fails  
2. Selected bulk delete does not work  
3. Knowledge graph persist: `Batch query failed: … type "agtype" does not exist`

## Five WHYs (agtype)

1. Why does persist fail? Merge `get_nodes_batch` → `batch_sql_query` casts with unqualified `::agtype`.  
2. Why is the type missing? Pool hygiene pins `search_path TO public`; `agtype` lives in `ag_catalog`.  
3. Why did session setup not always save it? Session `LOAD`/`SET search_path` mitigates, but any skip/drift resurfaces #350.  
4. Why wasn’t this closed with OODA-224? Rebuild/clear was gated; ingest merge read path was not.  
5. Why harden now? Native writes already use `::ag_catalog.agtype`; reads must DRY-match ([AGE setup](https://age.apache.org/age-manual/master/intro/setup.html)).

## Five WHYs (bulk delete)

See [ISSUE-delete-list-dual-ssot.md](ISSUE-delete-list-dual-ssot.md) / SPEC-098 Symptom C. Predecessor [#317](https://github.com/raphaelmansuy/edgequake/issues/317).

## Five WHYs (bulk upload)

1. Why “bulk upload fails”? Often secondary to KG persist (agtype / fleet / cardinality).  
2. Why WebUI ≠ batch API? Dropzone uses N× concurrent single-file admits; `/upload/batch` is SDK/API.  
3. Why no gate? API batch (SPEC-014/#236) was gated; WebUI multi-file was not.  
4. Why harden now? Prove the user path without requiring multipart rewrite.

## Proof matrix

| Symptom | Fix | Gate |
|---------|-----|------|
| agtype persist | `::ag_catalog.agtype` in `nodes_ops/read.rs` batch SQL | `contract_spec350_agtype_batch_get` · `e2e_spec350_agtype_batch_get_nodes` |
| Selected bulk delete | SPEC-098 dual-SSOT admit + FE sessions | `e2e_spec098_batch_delete_admit_deleting` · Playwright `spec098-bulk-delete-honesty` (seeds ≥2 docs) |
| WebUI multi-file upload | Existing dropzone path + e2e | Playwright `spec350-bulk-upload-webui` |

## Operator note

Postgres without Apache AGE still fails closed at boot unless `EDGEQUAKE_ALLOW_NO_GRAPH=1`. That is misconfiguration, not a product regression. Use the EdgeQuake AGE image (`ghcr.io/raphaelmansuy/edgequake-postgres`).
