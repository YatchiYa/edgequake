# Cluster 06 — Ops, CI, SDK, config hygiene

> **Sprint**: 4  
> **Laws**: LAW-3, LAW-8, LAW-2  
> **Defects**: D-47/D-49/D-50/D-41/C-27/X-24 FIXED · X-32 PARTIAL · D-42/D-44/D-45/D-48/X-09/X-33/X-36… CONFIRMED/PARTIAL

---

## WHY

Ops/CI/SDK drift made releases fragile. Several hygiene items are **FIXED**: `postgres-start` alias (D-47), portable `SED_INPLACE` (D-49), vision env example (D-50), weighted progress (D-41), LRU `last_accessed` (C-27), AUTO_RESUME comment (X-24). **X-32 PARTIAL**: critical CI still uses `continue-on-error` on postgres/e2e jobs. Backlog remains for config SSOT, SDK versions, audit partitions, upload SSOT, nested SDK workflows.

## ROOT CAUSE → STATUS

```
  D-47 postgres-start alias     FIXED
  D-49 SED_INPLACE              FIXED
  D-50 vision example           FIXED
  D-41 weighted progress        FIXED
  C-27 LRU touch                FIXED
  X-24 AUTO_RESUME comment      FIXED
  X-32 decorative CI residual   PARTIAL
  D-42/D-45/D-48/X-09/X-33/X-36 CONFIRMED backlog
  D-44 upload SSOT              PARTIAL
```

## SOLUTION

| Area | Status |
|------|--------|
| Makefile alias + portable sed | FIXED |
| Weighted progress | FIXED |
| LRU touch | FIXED |
| Blocking CI everywhere | PARTIAL (X-32) |
| Config `resolve()` / SDK 0.20 / audit partitions | CONFIRMED backlog |

## E2E / contracts

**Present:** `contract_makefile_has_postgres_start_alias`, `contract_no_sed_i_empty_string`, `unit_progress_weighted`, `contract_env_example_vision_not_openai_by_default`  
**Backlog:** `contract_ci_no_continue_on_error_critical`, `contract_frontend_test_must_run`, `contract_config_precedence`, `contract_upload_limit_ssot_50mib`, `contract_single_edgequake_llm_version`, `e2e_progress_survives_restart`, `e2e_audit_insert_next_month_partition`, `contract_sdk_major_matches_server_policy`
