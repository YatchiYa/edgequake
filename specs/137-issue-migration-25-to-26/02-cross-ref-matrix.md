# 02 — Cross-ref matrix

| This pack | Other SSOT | Contract |
|-----------|------------|----------|
| LAW-137-1 consent | [`drop_confirmed`](../../edgequake/src/main.rs) → `migrate_console::CONFIRM_DROP_FLAGS` | E2E-137-02/03 |
| LAW-137-2 unknown flags | `dispatch_migrate` apply branch | E2E-137-03 |
| LAW-137-3 drop SQL | [`125`](../../edgequake/migrations/125_spec091_kv_drop.sql) [`126`](../../edgequake/migrations/126_spec091_vector_drop.sql) [`131`](../../edgequake/migrations/131_spec091_fleet_vector_drop.sql) | Do not edit unless LAW-M3 |
| LAW-137-4 abort class | `classify_migrate_abort` / `print_drop_abort_hint` | E2E-137-04/05 |
| LAW-137-5 149 | [`149_tasks_document_id_column.sql`](../../edgequake/migrations/149_tasks_document_id_column.sql) | E2E-137-01 |
| LAW-137-6 guard | [`run_guard`](../../edgequake/src/migrate_advisor_cli.rs) | E2E-137-08 |
| LAW-137-7 AGE | [AGE drop_graph](https://age.apache.org/age-manual/master/intro/graphs.html) | E2E-137-07 |
| LAW-137-8 tags | `migration_class_tag` | unit + preflight |
| ExpandableOnly | [`migration_bootstrap/mod.rs`](../../edgequake/crates/edgequake-api/src/state/migration_bootstrap/mod.rs) | SPEC-091 C3 |
| 142 defer | [`legacy_store_census.rs`](../../edgequake/crates/edgequake-storage/src/legacy_store_census.rs) | SPEC-105 LAW-L5 |
| Advisor ≡ 125 | `contract_spec091_advisor_matches_125_guard` | SPEC-091 LAW-C3 |
| Advisor ≡ 126 | `contract_spec091_advisor_matches_126_guard` | SPEC-091 W4 |
| Advisor ≡ 131 | `e2e_spec111_provenance_parity` | SPEC-111 LAW-C3 |
| Checksum repair | [`checksum_repair.rs`](../../edgequake/crates/edgequake-api/src/state/migration_bootstrap/checksum_repair.rs) | SPEC-111 LAW-MIG |
| 0.26 runbook | [`upgrade-to-0.26.0.md`](../../docs/operations/upgrade-to-0.26.0.md) | this spec |
| 091 ladder | [`spec091-upgrade-from-v0.22.0.md`](../../docs/operations/spec091-upgrade-from-v0.22.0.md) · [`upgrade-to-0.24.2.md`](../../docs/operations/upgrade-to-0.24.2.md) | leftover drops |
| sqlx skip+apply | [Migrator](https://docs.rs/sqlx/latest/sqlx/migrate/struct.Migrator.html) · [PR 1030](https://github.com/launchbadge/sqlx/pull/1030) | ExpandableOnly then All |
| Pack template | [SPEC-110](../110-migration-issue/) · [SPEC-111](../111-issues/) | structure |

## Divergence rule

If this matrix and a code comment disagree, **code + contract test** win. Update
this file in the same PR as the code change.
