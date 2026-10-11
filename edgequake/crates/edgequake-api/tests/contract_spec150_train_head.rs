//! SPEC-150: HEAD schema train contracts (migrations 160–169 + fossils).
//!
//! These are compile-time / source / manifest contracts — no live Postgres
//! required. Runtime epoch proof remains `make spec150-matrix`.

use std::fs;
use std::path::PathBuf;

fn migrations_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../migrations")
}

fn numbered_sql_versions() -> Vec<i64> {
    let mut versions = Vec::new();
    for ent in fs::read_dir(migrations_dir()).expect("migrations dir") {
        let ent = ent.expect("dirent");
        let name = ent.file_name().to_string_lossy().into_owned();
        if let Some((num, rest)) = name.split_once('_') {
            if rest.ends_with(".sql") {
                if let Ok(v) = num.parse::<i64>() {
                    versions.push(v);
                }
            }
        }
    }
    versions.sort_unstable();
    versions
}

#[test]
fn head_train_includes_160_through_169() {
    let versions = numbered_sql_versions();
    let max = *versions.last().expect("at least one migration");
    assert_eq!(max, 192, "HEAD schema train must end at 192");
    for v in 160..=192 {
        assert!(
            versions.contains(&v),
            "missing migration {v} in edgequake/migrations/"
        );
    }
}

#[test]
fn manifest_compat_serve_max_matches_head() {
    let raw = include_str!("../../../migrations/manifest.toml");
    let max_line = raw
        .lines()
        .find(|l| l.starts_with("compat_serve_max"))
        .expect("compat_serve_max");
    let digits: String = max_line.chars().filter(|c| c.is_ascii_digit()).collect();
    assert_eq!(digits, "192");
}

#[test]
fn migration_172_is_expand_without_a_serving_floor() {
    let manifest = include_str!("../../../migrations/manifest.toml");
    let v172 = manifest
        .split("version = 172")
        .nth(1)
        .expect("migration 172");
    assert!(v172.contains("phase = \"expand\""));
    assert!(
        !v172.contains("serving_floor"),
        "172 must not raise the serving floor"
    );
    let sql = include_str!("../../../migrations/172_request_path_lookup_indexes.sql");
    for index in [
        "idx_documents_metadata_track_id",
        "idx_cel_workspace_chunk_pattern",
        "idx_crl_workspace_chunk_pattern",
        "text_pattern_ops",
    ] {
        assert!(sql.contains(index), "missing {index}");
    }
    let lookup = include_str!("../src/services/task_document_sync.rs");
    assert!(lookup.contains("UNION ALL"));
    assert!(!lookup.contains("track_id = $1 OR metadata"));
    let tasks = include_str!("../../edgequake-tasks/src/postgres.rs");
    assert!(tasks.contains("p.created_at < t.created_at"));
    assert!(!tasks.contains("ROW_NUMBER()"));
}

#[test]
fn migration_173_indexes_injection_lists() {
    let manifest = include_str!("../../../migrations/manifest.toml");
    let v173 = manifest
        .split("version = 173")
        .nth(1)
        .expect("migration 173");
    assert!(v173.contains("phase = \"expand\""));
    assert!(!v173.contains("serving_floor"));
    let sql = include_str!("../../../migrations/173_injection_list_index.sql");
    assert!(sql.contains("idx_documents_workspace_injection"));
    assert!(sql.contains("metadata->>'source_type'"));
    let list = include_str!("../src/services/injection_relational.rs");
    assert!(list.contains("metadata->>'source_type' = 'injection'"));
    assert!(list.contains("ORDER BY created_at DESC, id"));
}

#[test]
fn migration_174_indexes_pdf_lists() {
    let manifest = include_str!("../../../migrations/manifest.toml");
    let v174 = manifest
        .split("version = 174")
        .nth(1)
        .expect("migration 174");
    assert!(v174.contains("phase = \"expand\""));
    assert!(!v174.contains("serving_floor"));
    let sql = include_str!("../../../migrations/174_pdf_workspace_created_index.sql");
    assert!(sql.contains("idx_pdf_documents_workspace_created"));
    assert!(sql.contains("(workspace_id, created_at DESC)"));
    let list = include_str!("../../edgequake-storage/src/adapters/postgres/pdf_list_query.rs");
    assert!(list.contains("workspace_id = $"));
    assert!(list.contains("ORDER BY created_at DESC"));
}

#[test]
fn migration_175_indexes_task_lists() {
    let manifest = include_str!("../../../migrations/manifest.toml");
    let v175 = manifest
        .split("version = 175")
        .nth(1)
        .expect("migration 175");
    assert!(v175.contains("phase = \"expand\""));
    assert!(!v175.contains("serving_floor"));
    let sql = include_str!("../../../migrations/175_task_workspace_created_index.sql");
    assert!(sql.contains("idx_tasks_tenant_workspace_created_track"));
    assert!(sql.contains("(tenant_id, workspace_id, created_at DESC, track_id DESC)"));
    let list = include_str!("../../edgequake-tasks/src/postgres.rs");
    assert!(list.contains("tenant_id = $"));
    assert!(list.contains("workspace_id = $"));
    assert!(list.contains("ORDER BY {sort_field} {sort_order}, track_id {sort_order}"));
}

#[test]
fn migration_176_indexes_conversation_sorts() {
    let manifest = include_str!("../../../migrations/manifest.toml");
    let v176 = manifest
        .split("version = 176")
        .nth(1)
        .expect("migration 176");
    assert!(v176.contains("phase = \"expand\""));
    assert!(!v176.contains("serving_floor"));
    let sql = include_str!("../../../migrations/176_conversation_sort_indexes.sql");
    assert!(sql.contains("idx_conversations_tenant_user_created"));
    assert!(sql.contains("idx_conversations_tenant_user_title"));
    assert!(sql.contains("(tenant_id, user_id, created_at DESC)"));
    assert!(sql.contains("(tenant_id, user_id, title)"));
    let list = include_str!("../../edgequake-storage/src/adapters/postgres/conversation.rs");
    assert!(list.contains("tenant_id = $1"));
    assert!(list.contains("user_id = $2"));
    assert!(list.contains("created_at {}"));
    assert!(list.contains("title {}"));
}

#[test]
fn migration_177_indexes_task_status_and_type_pages() {
    let manifest = include_str!("../../../migrations/manifest.toml");
    let v177 = manifest
        .split("version = 177")
        .nth(1)
        .expect("migration 177");
    assert!(v177.contains("phase = \"expand\""));
    assert!(!v177.contains("serving_floor"));
    let sql = include_str!("../../../migrations/177_task_status_type_created_indexes.sql");
    assert!(sql.contains("idx_tasks_tenant_workspace_status_created_track"));
    assert!(sql.contains("idx_tasks_tenant_workspace_type_created_track"));
    assert!(sql.contains("(tenant_id, workspace_id, status, created_at DESC, track_id DESC)"));
    assert!(sql.contains("(tenant_id, workspace_id, task_type, created_at DESC, track_id DESC)"));
    let list = include_str!("../../edgequake-tasks/src/postgres.rs");
    assert!(list.contains("status = $"));
    assert!(list.contains("task_type = $"));
    assert!(list.contains("ORDER BY {sort_field} {sort_order}, track_id {sort_order}"));
}

#[test]
fn migration_178_indexes_task_updated_pages() {
    let manifest = include_str!("../../../migrations/manifest.toml");
    let v178 = manifest
        .split("version = 178")
        .nth(1)
        .expect("migration 178");
    assert!(v178.contains("phase = \"expand\""));
    assert!(!v178.contains("serving_floor"));
    let sql = include_str!("../../../migrations/178_task_workspace_updated_index.sql");
    assert!(sql.contains("idx_tasks_tenant_workspace_updated_track"));
    assert!(sql.contains("(tenant_id, workspace_id, updated_at DESC, track_id DESC)"));
    let list = include_str!("../../edgequake-tasks/src/postgres.rs");
    assert!(list.contains("SortField::UpdatedAt => \"updated_at\""));
    assert!(list.contains("ORDER BY {sort_field} {sort_order}"));
}

#[test]
fn migration_192_indexes_scoped_drain_claims() {
    let manifest = include_str!("../../../migrations/manifest.toml");
    let v192 = manifest
        .split("version = 192")
        .nth(1)
        .expect("migration 192");
    assert!(v192.contains("phase = \"expand\""));
    assert!(!v192.contains("serving_floor"));
    let sql = include_str!("../../../migrations/192_drain_workspace_claim_index.sql");
    assert!(sql.contains("idx_outbox_events_workspace_unprocessed"));
    assert!(sql.contains("(workspace_id, available_at, created_at)"));
    assert!(sql.contains("idx_compensation_quarantine_due"));
    assert!(sql.contains("idx_compensation_quarantine_workspace_next"));
    let outbox = include_str!("../../edgequake-storage/src/outbox_drain.rs");
    assert!(outbox.contains("OUTBOX_CLAIM_SCOPED"));
    assert!(outbox.contains("workspace_id = $2"));
    assert!(!outbox.contains("$2::uuid IS NULL OR workspace_id = $2"));
    let quarantine = include_str!("../../edgequake-storage/src/compensation_drain.rs");
    assert!(quarantine.contains("QUARANTINE_CLAIM_OPEN"));
    assert!(quarantine.contains("QUARANTINE_CLAIM_SCOPED"));
    assert!(!quarantine.contains("$2::uuid IS NULL OR workspace_id = $2"));
}

#[test]
fn migration_191_indexes_document_list_by_tenant() {
    let manifest = include_str!("../../../migrations/manifest.toml");
    let v191 = manifest
        .split("version = 191")
        .nth(1)
        .expect("migration 191");
    assert!(v191.contains("phase = \"expand\""));
    assert!(!v191.contains("serving_floor"));
    let sql = include_str!("../../../migrations/191_document_tenant_created_index.sql");
    assert!(sql.contains("idx_documents_workspace_tenant_created"));
    assert!(sql.contains("(workspace_id, tenant_id, created_at DESC, id DESC)"));
    assert!(sql.contains("idx_documents_workspace_null_tenant_created"));
    assert!(sql.contains("WHERE tenant_id IS NULL"));
    let list = include_str!("../src/document_read_model.rs");
    assert!(list.contains("fn document_list_statement"));
    assert!(list.contains("tenant_id = $2"));
    assert!(list.contains("tenant_id IS NULL"));
    assert!(list.contains("ORDER BY created_at DESC, id DESC"));
    assert!(list.contains("fn document_status_statement"));
    assert!(list.contains("force_custom_plan"));
    assert!(
        !list.contains("$2::uuid IS NULL OR tenant_id IS NULL OR tenant_id = $2"),
        "list, status, and workspace delete must require the tenant predicate"
    );
}

#[test]
fn migration_190_indexes_chunk_search_by_legacy_key() {
    let manifest = include_str!("../../../migrations/manifest.toml");
    let v190 = manifest
        .split("version = 190")
        .nth(1)
        .expect("migration 190");
    assert!(v190.contains("phase = \"expand\""));
    assert!(!v190.contains("serving_floor"));
    let sql = include_str!("../../../migrations/190_chunk_legacy_key_index.sql");
    assert!(sql.contains("idx_chunks_legacy_chunk_key"));
    assert!(sql.contains("(metadata->>'legacy_chunk_key') IS NOT NULL"));
    let fts = include_str!("../../edgequake-storage/src/adapters/postgres/vector/fts.rs");
    assert!(fts.contains("TYPED_CHUNKS_FTS_SQL_IN_IDS"));
    assert!(fts.contains("OFFSET 0"));
    assert!(fts.contains("id_filter.is_some()"));
}

#[test]
fn migration_189_indexes_chunk_search_by_modality() {
    let manifest = include_str!("../../../migrations/manifest.toml");
    let v189 = manifest
        .split("version = 189")
        .nth(1)
        .expect("migration 189");
    assert!(v189.contains("phase = \"expand\""));
    assert!(!v189.contains("serving_floor"));
    let sql = include_str!("../../../migrations/189_chunk_modality_index.sql");
    assert!(sql.contains("idx_chunks_modality"));
    assert!(sql.contains("(metadata->>'modality') IS NOT NULL"));
    let fts = include_str!("../../edgequake-storage/src/adapters/postgres/vector/fts.rs");
    assert!(fts.contains("modalities.is_some()"));
    assert!(fts.contains("AND c.metadata->>'modality' = ANY($6)"));
}

#[test]
fn migration_188_indexes_conversation_title_search_by_pin_and_archive() {
    let manifest = include_str!("../../../migrations/manifest.toml");
    let v188 = manifest
        .split("version = 188")
        .nth(1)
        .expect("migration 188");
    assert!(v188.contains("phase = \"expand\""));
    assert!(!v188.contains("serving_floor"));
    let sql = include_str!("../../../migrations/188_conversation_flag_title_fts.sql");
    assert!(sql.contains("idx_conversations_tenant_user_flags_title_fts"));
    assert!(sql.contains("is_pinned public.bool_ops"));
    assert!(sql.contains("is_archived public.bool_ops"));
    let list = include_str!("../../edgequake-storage/src/adapters/postgres/conversation.rs");
    assert!(list.contains("is_pinned = ${}"));
    assert!(list.contains("is_archived = ${}"));
    assert!(list.contains("to_tsvector('english', title) @@ plainto_tsquery('english'"));
}

#[test]
fn migration_187_indexes_conversation_title_search_in_a_folder() {
    let manifest = include_str!("../../../migrations/manifest.toml");
    let v187 = manifest
        .split("version = 187")
        .nth(1)
        .expect("migration 187");
    assert!(v187.contains("phase = \"expand\""));
    assert!(!v187.contains("serving_floor"));
    let sql = include_str!("../../../migrations/187_conversation_folder_title_fts.sql");
    assert!(sql.contains("idx_conversations_tenant_user_folder_title_fts"));
    assert!(sql.contains("folder_id public.uuid_ops"));
    let list = include_str!("../../edgequake-storage/src/adapters/postgres/conversation.rs");
    assert!(list.contains("folder_id = ${}"));
    assert!(list.contains("to_tsvector('english', title) @@ plainto_tsquery('english'"));
}

#[test]
fn migration_186_indexes_conversation_title_search_by_user() {
    let manifest = include_str!("../../../migrations/manifest.toml");
    let v186 = manifest
        .split("version = 186")
        .nth(1)
        .expect("migration 186");
    assert!(v186.contains("phase = \"expand\""));
    assert!(!v186.contains("serving_floor"));
    let sql = include_str!("../../../migrations/186_conversation_user_title_fts.sql");
    assert!(sql.contains("btree_gin"));
    assert!(sql.contains("WITH SCHEMA public"));
    assert!(sql.contains("idx_conversations_tenant_user_title_fts"));
    assert!(sql.contains("public.uuid_ops"));
    let list = include_str!("../../edgequake-storage/src/adapters/postgres/conversation.rs");
    assert!(list.contains("to_tsvector('english', title) @@ plainto_tsquery('english'"));
}

#[test]
fn migration_185_indexes_workspace_chunk_search() {
    let manifest = include_str!("../../../migrations/manifest.toml");
    let v185 = manifest
        .split("version = 185")
        .nth(1)
        .expect("migration 185");
    assert!(v185.contains("phase = \"expand\""));
    assert!(!v185.contains("serving_floor"));
    let sql = include_str!("../../../migrations/185_chunk_workspace_fts_index.sql");
    assert!(sql.contains("idx_chunks_workspace"));
    assert!(sql.contains("ON public.chunks (workspace_id)"));
    let fts = include_str!("../../edgequake-storage/src/adapters/postgres/vector/fts.rs");
    assert!(fts.contains("TYPED_CHUNKS_FTS_SQL_IN_WORKSPACE"));
    assert!(fts.contains("c.workspace_id = $2 OR c.workspace_id IS NULL"));
    assert!(fts.contains("workspace_uuid.is_some()"));
}

#[test]
fn migration_184_indexes_task_status_and_type_together() {
    let manifest = include_str!("../../../migrations/manifest.toml");
    let v184 = manifest
        .split("version = 184")
        .nth(1)
        .expect("migration 184");
    assert!(v184.contains("phase = \"expand\""));
    assert!(!v184.contains("serving_floor"));
    let sql = include_str!("../../../migrations/184_task_status_type_pair_indexes.sql");
    assert!(sql.contains("idx_tasks_tenant_workspace_status_type_created_track"));
    assert!(sql.contains("idx_tasks_tenant_workspace_status_type_updated"));
    assert!(sql
        .contains("(tenant_id, workspace_id, status, task_type, created_at DESC, track_id DESC)"));
    assert!(sql.contains("(tenant_id, workspace_id, status, task_type, updated_at DESC)"));
    let list = include_str!("../../edgequake-tasks/src/postgres.rs");
    assert!(list.contains("status = $"));
    assert!(list.contains("task_type = $"));
    assert!(list.contains("ORDER BY {sort_field} {sort_order}, track_id {sort_order}"));
    assert!(list.contains("SortField::UpdatedAt => \"updated_at\""));
}

#[test]
fn migration_183_indexes_conversation_filter_sort_pages() {
    let manifest = include_str!("../../../migrations/manifest.toml");
    let v183 = manifest
        .split("version = 183")
        .nth(1)
        .expect("migration 183");
    assert!(v183.contains("phase = \"expand\""));
    assert!(!v183.contains("serving_floor"));
    let sql = include_str!("../../../migrations/183_conversation_filter_sort_indexes.sql");
    for index in [
        "idx_conversations_tenant_user_folder_created",
        "idx_conversations_tenant_user_folder_title",
        "idx_conversations_tenant_user_pinned_created",
        "idx_conversations_tenant_user_pinned_title",
        "idx_conversations_tenant_user_unfiled_created",
        "idx_conversations_tenant_user_unfiled_title",
        "idx_conversations_tenant_user_archived_created",
        "idx_conversations_tenant_user_archived_title",
    ] {
        assert!(sql.contains(index), "missing {index}");
    }
    assert!(sql.contains("(tenant_id, user_id, folder_id, created_at DESC)"));
    assert!(sql.contains("(tenant_id, user_id, folder_id, title)"));
    assert!(sql.contains("(tenant_id, user_id, is_pinned, created_at DESC)"));
    assert!(sql.contains("(tenant_id, user_id, is_pinned, title)"));
    assert!(sql.contains("(tenant_id, user_id, created_at DESC)"));
    assert!(sql.contains("WHERE folder_id IS NULL"));
    assert!(sql.contains("(tenant_id, user_id, is_archived, created_at DESC)"));
    assert!(sql.contains("(tenant_id, user_id, is_archived, title)"));
    let list = include_str!("../../edgequake-storage/src/adapters/postgres/conversation.rs");
    assert!(list.contains("folder_id = $"));
    assert!(list.contains("is_pinned = $"));
    assert!(list.contains("folder_id IS NULL"));
    assert!(list.contains("is_archived = $"));
    assert!(list.contains("\"created_at\" => format!(\"created_at {}\", sort_order)"));
    assert!(list.contains("\"title\" => format!(\"title {}\", sort_order)"));
}

#[test]
fn migration_182_indexes_task_status_and_type_updated_pages() {
    let manifest = include_str!("../../../migrations/manifest.toml");
    let v182 = manifest
        .split("version = 182")
        .nth(1)
        .expect("migration 182");
    assert!(v182.contains("phase = \"expand\""));
    assert!(!v182.contains("serving_floor"));
    let sql = include_str!("../../../migrations/182_task_status_type_updated_indexes.sql");
    assert!(sql.contains("idx_tasks_tenant_workspace_status_updated"));
    assert!(sql.contains("idx_tasks_tenant_workspace_type_updated"));
    assert!(sql.contains("(tenant_id, workspace_id, status, updated_at DESC)"));
    assert!(sql.contains("(tenant_id, workspace_id, task_type, updated_at DESC)"));
    let list = include_str!("../../edgequake-tasks/src/postgres.rs");
    assert!(list.contains("status = $"));
    assert!(list.contains("task_type = $"));
    assert!(list.contains("SortField::UpdatedAt => \"updated_at\""));
}

#[test]
fn migration_181_indexes_conversation_unfiled_pages() {
    let manifest = include_str!("../../../migrations/manifest.toml");
    let v181 = manifest
        .split("version = 181")
        .nth(1)
        .expect("migration 181");
    assert!(v181.contains("phase = \"expand\""));
    assert!(!v181.contains("serving_floor"));
    let sql = include_str!("../../../migrations/181_conversation_unfiled_updated_index.sql");
    assert!(sql.contains("idx_conversations_tenant_user_unfiled_updated"));
    assert!(sql.contains("(tenant_id, user_id, updated_at DESC)"));
    assert!(sql.contains("WHERE folder_id IS NULL"));
    let list = include_str!("../../edgequake-storage/src/adapters/postgres/conversation.rs");
    assert!(list.contains("folder_id IS NULL"));
    assert!(list.contains("updated_at {}"));
}

#[test]
fn migration_180_indexes_conversation_pinned_pages() {
    let manifest = include_str!("../../../migrations/manifest.toml");
    let v180 = manifest
        .split("version = 180")
        .nth(1)
        .expect("migration 180");
    assert!(v180.contains("phase = \"expand\""));
    assert!(!v180.contains("serving_floor"));
    let sql = include_str!("../../../migrations/180_conversation_pinned_updated_index.sql");
    assert!(sql.contains("idx_conversations_tenant_user_pinned_updated"));
    assert!(sql.contains("(tenant_id, user_id, is_pinned, updated_at DESC)"));
    let list = include_str!("../../edgequake-storage/src/adapters/postgres/conversation.rs");
    assert!(list.contains("is_pinned = $"));
    assert!(list.contains("updated_at {}"));
}

#[test]
fn migration_179_indexes_conversation_folder_pages() {
    let manifest = include_str!("../../../migrations/manifest.toml");
    let v179 = manifest
        .split("version = 179")
        .nth(1)
        .expect("migration 179");
    assert!(v179.contains("phase = \"expand\""));
    assert!(!v179.contains("serving_floor"));
    let sql = include_str!("../../../migrations/179_conversation_folder_updated_index.sql");
    assert!(sql.contains("idx_conversations_tenant_user_folder_updated"));
    assert!(sql.contains("(tenant_id, user_id, folder_id, updated_at DESC)"));
    let list = include_str!("../../edgequake-storage/src/adapters/postgres/conversation.rs");
    assert!(list.contains("folder_id = $"));
    assert!(list.contains("updated_at {}"));
}

#[test]
fn fossils_001_and_019_are_registered() {
    let m = edgequake_migrate_manifest::load();
    let m001 = m.migration.iter().find(|e| e.version == 1).expect("m001");
    let m019 = m.migration.iter().find(|e| e.version == 19).expect("m019");
    assert!(
        !m001.fossils.is_empty(),
        "migration 001 must list the v0.11.0 fossil (issue #195)"
    );
    assert!(
        m001.fossils.iter().any(|f| !f.dev_only),
        "001 fossil must be production-accept"
    );
    assert!(
        !m019.fossils.is_empty(),
        "migration 019 must list the v0.10.6–v0.10.12 fossil"
    );
    assert!(
        m019.fossils.iter().any(|f| !f.dev_only),
        "019 fossil must be production-accept"
    );
}

#[test]
fn apply_uses_sequential_progress_path() {
    let src = include_str!("../src/state/migration_bootstrap/apply.rs");
    assert!(
        src.contains("apply_pending_sequentially"),
        "CLI must apply migrations one-by-one for operator progress"
    );
    assert!(
        src.contains("print_upgrade_path"),
        "CLI must print UPGRADE PATH before apply"
    );
    assert!(
        src.contains("record_migration_step"),
        "each step must record migration_run_step telemetry when table exists"
    );
}

#[test]
fn progress_module_release_map_covers_head() {
    // Source contract (no `postgres` feature required for this integration test).
    let src = include_str!("../src/state/migration_bootstrap/progress.rs");
    assert!(
        src.contains("167..=168 => \"v0.32.0–v0.32.2\""),
        "release_for_schema must map 167–168 to v0.32.x"
    );
    assert!(
        src.contains("169 => \"v0.33.0\""),
        "release_for_schema must map 169 to v0.33.0"
    );
    assert!(
        src.contains("170..=192 => \"v0.35.0\""),
        "release_for_schema must map 170–192 to v0.35.0"
    );
    assert!(
        src.contains("149 => \"v0.26.0–v0.26.10\""),
        "release_for_schema must map 149 to v0.26.x"
    );
    assert!(
        src.contains("150..=159 => \"v0.27.0\""),
        "release_for_schema must map 150–159 to v0.27.0"
    );
}

#[test]
fn sequential_apply_is_restartable_by_design() {
    // Idempotent re-run / interrupted-retry: each step is its own sqlx transaction
    // + ledger row. Re-running `edgequake migrate` after a crash resumes at the
    // first unapplied version (sqlx skips applied checksums).
    let src = include_str!("../src/state/migration_bootstrap/apply.rs");
    assert!(src.contains("one.run(pool)"));
    assert!(src.contains("locking: false"));
    assert!(
        src.contains("finish_migration_run(pool, run_id, \"error\""),
        "failed step must close migration_run as error so operators can resume"
    );
}
