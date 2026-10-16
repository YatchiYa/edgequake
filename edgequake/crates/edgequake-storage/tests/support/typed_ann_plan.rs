//! EXPLAIN the actual statement prepared by a typed embedding adapter.
//! Shared by isolated recall tests and the opt-in read-only live-data audit.

use edgequake_storage::traits::domain::VectorQuery;
use serde_json::{json, Value};
use sqlx::PgPool;
use std::time::Instant;
use uuid::Uuid;

pub async fn production_plan(pool: &PgPool, kind: &str, req: &VectorQuery, model: Uuid) -> Value {
    let table = if kind == "chunk" {
        "chunk_embeddings".to_string()
    } else {
        format!("{kind}_embeddings")
    };
    let name: String=sqlx::query_scalar("SELECT name FROM pg_prepared_statements WHERE statement LIKE $1 AND statement NOT LIKE '%pg_prepared_statements%' ORDER BY prepare_time DESC LIMIT 1")
        .bind(format!("%FROM {table} %")).fetch_one(pool).await.unwrap();
    assert!(name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_'));
    let vector = format!(
        "[{}]",
        req.embedding
            .iter()
            .map(f32::to_string)
            .collect::<Vec<_>>()
            .join(",")
    );
    let workspace = req.workspace_id.unwrap().into_uuid();
    let documents = req.document_ids.as_ref().map_or_else(
        || "NULL::uuid[]".into(),
        |ids| {
            format!(
                "ARRAY[{}]::uuid[]",
                ids.iter()
                    .map(|id| format!("'{id}'"))
                    .collect::<Vec<_>>()
                    .join(",")
            )
        },
    );
    let tenant = req.tenant_id.map_or_else(
        || "NULL::uuid".into(),
        |id| format!("'{}'::uuid", id.into_uuid()),
    );
    let modalities = if req.modalities.is_some() {
        "ARRAY['print']::text[]"
    } else {
        "NULL::text[]"
    };
    // The adapter's prepared statement is the query under test. Its parameter
    // list is whatever that statement declared, not a reconstructed call.
    let mut conn = pool.acquire().await.unwrap();
    let mut tx = edgequake_storage::adapters::postgres::LocalTimeoutTx::begin(&mut conn, 2_000)
        .await
        .unwrap();
    let owner: Uuid =
        sqlx::query_scalar("SELECT tenant_id FROM public.workspaces WHERE workspace_id=$1")
            .bind(workspace)
            .fetch_one(tx.as_mut())
            .await
            .unwrap();
    edgequake_storage::adapters::postgres::rls::enforce_tenant_context(
        tx.as_mut(),
        owner,
        Some(workspace),
        None,
    )
    .await
    .unwrap();
    let candidate_limit = edgequake_storage::adapters::postgres::AnnExactReorderPolicy::for_search(
        "relaxed_order",
        req.limit as usize,
    )
    .effective_candidate_k(req.limit as usize)
    .max(req.limit as usize);
    for statement in edgequake_storage::PgVectorStorage::search_tuning_statements(
        edgequake_storage::adapters::postgres::VectorIndexType::HNSW,
        candidate_limit,
        true,
        true,
    ) {
        sqlx::query(&statement).execute(tx.as_mut()).await.unwrap();
    }
    let prepared: (String, Vec<String>) = sqlx::query_as(
        "SELECT statement, parameter_types::text[] FROM pg_prepared_statements WHERE name = $1",
    )
    .bind(&name)
    .fetch_one(tx.as_mut())
    .await
    .unwrap();
    let model_id = model.to_string();
    let workspace_id = workspace.to_string();
    let sql = explain_prepared(
        &prepared.0,
        &prepared.1,
        &ExplainBinds {
            vector: &vector,
            model: &model_id,
            workspace: &workspace_id,
            documents: &documents,
            tenant: &tenant,
            modalities,
            candidate_limit,
        },
    );
    let mut generic_samples = Vec::new();
    let mut custom_samples = Vec::new();
    let mut plan = Value::Null;
    let mut generic_timed_out = false;
    for sample in 0..26 {
        for (mode, samples) in [
            ("force_generic_plan", &mut generic_samples),
            ("force_custom_plan", &mut custom_samples),
        ] {
            if mode == "force_generic_plan" && generic_timed_out {
                continue;
            }
            sqlx::query("SAVEPOINT plan_probe")
                .execute(tx.as_mut())
                .await
                .unwrap();
            sqlx::query("SELECT set_config('plan_cache_mode',$1,true)")
                .bind(mode)
                .execute(tx.as_mut())
                .await
                .unwrap();
            let start = Instant::now();
            let measured: Value = match sqlx::query_scalar(&sql).fetch_one(tx.as_mut()).await {
                Ok(value) => value,
                Err(error)
                    if mode == "force_generic_plan"
                        && error.as_database_error().and_then(|e| e.code()).as_deref()
                            == Some("57014") =>
                {
                    sqlx::query("ROLLBACK TO SAVEPOINT plan_probe")
                        .execute(tx.as_mut())
                        .await
                        .unwrap();
                    sqlx::query("RELEASE SAVEPOINT plan_probe")
                        .execute(tx.as_mut())
                        .await
                        .unwrap();
                    generic_timed_out = true;
                    continue;
                }
                Err(error) => panic!("{kind}/{mode} plan failed: {error}"),
            };
            sqlx::query("RELEASE SAVEPOINT plan_probe")
                .execute(tx.as_mut())
                .await
                .unwrap();
            if sample >= 5 {
                samples.push(start.elapsed().as_secs_f64() * 1000.0);
            }
            if mode == "force_custom_plan" {
                plan = measured;
            }
        }
    }
    generic_samples.sort_by(f64::total_cmp);
    custom_samples.sort_by(f64::total_cmp);
    println!(
        "TYPED_ANN_PLAN_COMPARISON {}",
        json!({
            "kind":kind, "workspace":workspace, "samples":21, "warmup":5,
            "generic_p50_ms":generic_samples.get(10), "generic_p95_ms":generic_samples.get(19),
            "generic_statement_deadline_exceeded":generic_timed_out,
            "custom_p50_ms":custom_samples[10], "custom_p95_ms":custom_samples[19],
            "generic_samples_ms":generic_samples, "custom_samples_ms":custom_samples,
            "planner_scan_switches_forced":false,
            "limits":"paired modes on the same scoped connection/data/probe; includes EXPLAIN planning and execution, not a production SLA"
        })
    );
    tx.commit().await.unwrap();
    plan
}

/// Bind the prepared statement from the types Postgres recorded, then EXPLAIN
/// that SQL. `$10` is replaced before `$1`. A cast already written after `$n`
/// is left in place.
struct ExplainBinds<'a> {
    vector: &'a str,
    model: &'a str,
    workspace: &'a str,
    documents: &'a str,
    tenant: &'a str,
    modalities: &'a str,
    candidate_limit: usize,
}

fn explain_prepared(statement: &str, types: &[String], binds: &ExplainBinds<'_>) -> String {
    let mut uuid_i = 0usize;
    let mut text_array_i = 0usize;
    let mut values = Vec::with_capacity(types.len());
    for ty in types {
        let normalized = ty.to_ascii_lowercase();
        let value = if normalized.contains("vector") {
            format!("'{}'", binds.vector)
        } else if normalized.contains("uuid") && normalized.contains("[]") {
            binds.documents.to_string()
        } else if normalized.contains("uuid") {
            uuid_i += 1;
            match uuid_i {
                1 => format!("'{}'::uuid", binds.model),
                2 => format!("'{}'::uuid", binds.workspace),
                _ => binds.tenant.to_string(),
            }
        } else if normalized.contains("text") && normalized.contains("[]") {
            text_array_i += 1;
            if text_array_i == 1 {
                binds.modalities.to_string()
            } else {
                "NULL::text[]".to_string()
            }
        } else if normalized.contains("text") {
            "NULL::text".to_string()
        } else if normalized.contains("int") {
            binds.candidate_limit.to_string()
        } else {
            format!("NULL::{ty}")
        };
        values.push(value);
    }
    let mut sql = statement.to_string();
    for index in (1..=values.len()).rev() {
        sql = substitute_parameter(&sql, index, &values[index - 1]);
    }
    format!("EXPLAIN (FORMAT JSON) {sql}")
}

fn substitute_parameter(sql: &str, index: usize, value: &str) -> String {
    let needle = format!("${index}");
    let mut out = String::with_capacity(sql.len() + value.len());
    let mut cursor = 0usize;
    while let Some(found) = sql[cursor..].find(&needle) {
        let at = cursor + found;
        let after = at + needle.len();
        let continues_number = sql[after..]
            .chars()
            .next()
            .is_some_and(|ch| ch.is_ascii_digit());
        out.push_str(&sql[cursor..at]);
        if continues_number {
            out.push_str(&needle);
            cursor = after;
            continue;
        }
        let casts = sql[after..].trim_start().starts_with("::");
        out.push_str(if casts {
            without_trailing_cast(value)
        } else {
            value
        });
        cursor = after;
    }
    out.push_str(&sql[cursor..]);
    out
}

fn without_trailing_cast(value: &str) -> &str {
    let bytes = value.as_bytes();
    let mut quoted = false;
    let mut last = None;
    let mut index = 0usize;
    while index + 1 < bytes.len() {
        if bytes[index] == b'\'' {
            if quoted && index + 1 < bytes.len() && bytes[index + 1] == b'\'' {
                index += 2;
                continue;
            }
            quoted = !quoted;
            index += 1;
            continue;
        }
        if !quoted && bytes[index] == b':' && bytes[index + 1] == b':' {
            last = Some(index);
        }
        index += 1;
    }
    match last {
        Some(cut) => &value[..cut],
        None => value,
    }
}
