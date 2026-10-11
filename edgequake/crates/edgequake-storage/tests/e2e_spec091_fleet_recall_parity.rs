//! SPEC-091 IW2 recall parity: typed `entity_embeddings` ANN recall@10
//! matches exact cosine baseline on a seeded corpus.
#![cfg(feature = "postgres")]

#[path = "support/postgres_test_config.rs"]
mod postgres_test_config;
#[path = "support/spec091_w3.rs"]
mod w3;

use edgequake_storage::embedding_family::EmbeddingFamily;
use edgequake_storage::traits::domain::{
    FleetEmbeddingIndex, FleetEmbeddingKey, FleetEmbeddingRow, ModelId, VectorQuery, WorkspaceId,
};
use postgres_test_config::{contract_pg_pool, require_or_skip_postgres};
use std::collections::HashSet;
use uuid::Uuid;

const DIM: usize = 1536;
const CORPUS: usize = 24;
const TOP_K: usize = 10;

fn cosine(a: &[f32], b: &[f32]) -> f32 {
    let mut dot = 0f32;
    let mut na = 0f32;
    let mut nb = 0f32;
    for (x, y) in a.iter().zip(b.iter()) {
        dot += x * y;
        na += x * x;
        nb += y * y;
    }
    if na == 0.0 || nb == 0.0 {
        return 0.0;
    }
    dot / (na.sqrt() * nb.sqrt())
}

#[tokio::test]
async fn e2e_spec091_fleet_recall_parity_entity_typed_vs_exact() {
    let Some(cfg) = require_or_skip_postgres("spec091_iw2_fleet_recall") else {
        return;
    };
    let _g = w3::w3_lock().lock().await;
    let pool = contract_pg_pool(&cfg).await;
    let ws = w3::seed_workspace(&pool, "fleet-recall").await;
    let tenant = w3::workspace_tenant(&pool, ws).await;

    let index = edgequake_storage::PgFleetEmbeddingIndex::new(pool.clone(), "iw2-recall-model");

    let mut corpus: Vec<(String, Vec<f32>)> = Vec::new();
    let mut rows: Vec<FleetEmbeddingRow> = Vec::new();
    for i in 0..CORPUS {
        let name = format!("ENTITY_{i}");
        let eid: Uuid = sqlx::query_scalar(
            "INSERT INTO entities (name, entity_type, workspace_id, tenant_id) VALUES ($1, 'concept', $2, $3) \
             RETURNING id",
        )
        .bind(&name)
        .bind(ws)
        .bind(tenant)
        .fetch_one(&pool)
        .await
        .expect("seed entity");
        let emb = w3::make_embedding(DIM, 700 + i as u32);
        let legacy_id = format!("entity:{name}");
        corpus.push((legacy_id.clone(), emb.clone()));
        rows.push(FleetEmbeddingRow {
            key: FleetEmbeddingKey::Entity(eid),
            workspace_id: WorkspaceId::new(ws),
            dimensions: DIM as i32,
            embedding: emb,
            legacy_vector_id: Some(legacy_id),
        });
    }
    index
        .upsert_batch(EmbeddingFamily::Entity, ModelId(Uuid::nil()), &rows)
        .await
        .expect("upsert");

    let probes = [0u32, 3, 11, 17];
    let mut total_recall = 0f32;
    for probe_seed in probes {
        let query_emb = w3::make_embedding(DIM, 900 + probe_seed);
        let mut exact: Vec<(String, f32)> = corpus
            .iter()
            .map(|(id, emb)| (id.clone(), cosine(&query_emb, emb)))
            .collect();
        exact.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        let exact_top: HashSet<String> =
            exact.iter().take(TOP_K).map(|(id, _)| id.clone()).collect();

        let hits = index
            .search(
                EmbeddingFamily::Entity,
                &VectorQuery {
                    model_id: ModelId(Uuid::nil()),
                    model_revision: "test-current".into(),
                    workspace_id: Some(WorkspaceId::new(ws)),
                    document_ids: None,
                    tenant_id: None,
                    modalities: None,
                    filter_ids: None,
                    vector_type: None,
                    embedding: query_emb,
                    limit: TOP_K as u32,
                },
            )
            .await
            .expect("typed search");
        let typed_top: HashSet<String> = hits.iter().map(|h| h.legacy_id.clone()).collect();
        let overlap = exact_top.intersection(&typed_top).count() as f32;
        total_recall += overlap / TOP_K as f32;
    }
    let mean_recall = total_recall / probes.len() as f32;
    assert!(
        mean_recall >= 0.9,
        "entity typed recall parity gate failed: {mean_recall:.3} < 0.9"
    );

    w3::cleanup_workspace(&pool, ws).await;
}

#[tokio::test]
async fn e2e_spec091_fleet_entity_search_by_id_and_document() {
    let Some(cfg) = require_or_skip_postgres("spec091_iw2_fleet_id") else {
        return;
    };
    let _g = w3::w3_lock().lock().await;
    let pool = contract_pg_pool(&cfg).await;
    let ws = w3::seed_workspace(&pool, "fleet-id").await;
    let tenant = w3::workspace_tenant(&pool, ws).await;
    let doc_a = Uuid::new_v4();
    let doc_b = Uuid::new_v4();
    let index = edgequake_storage::PgFleetEmbeddingIndex::new(pool.clone(), "iw2-id-model");

    let mut ids = Vec::new();
    for (i, doc) in [(0u32, doc_a), (1, doc_b)] {
        let name = format!("FLEET_ID_{i}");
        let eid: Uuid = sqlx::query_scalar(
            "INSERT INTO entities (name, entity_type, workspace_id, tenant_id, source_ids) \
             VALUES ($1, 'concept', $2, $3, $4) RETURNING id",
        )
        .bind(&name)
        .bind(ws)
        .bind(tenant)
        .bind(vec![doc])
        .fetch_one(&pool)
        .await
        .expect("seed entity");
        ids.push((eid, name));
    }
    let rows: Vec<FleetEmbeddingRow> = ids
        .iter()
        .enumerate()
        .map(|(i, (eid, name))| FleetEmbeddingRow {
            key: FleetEmbeddingKey::Entity(*eid),
            workspace_id: WorkspaceId::new(ws),
            dimensions: 768,
            embedding: w3::make_embedding(768, 40 + i as u32),
            legacy_vector_id: Some(format!("entity:{name}")),
        })
        .collect();
    index
        .upsert_batch(EmbeddingFamily::Entity, ModelId(Uuid::nil()), &rows)
        .await
        .expect("upsert");

    let base = VectorQuery {
        model_id: ModelId(Uuid::nil()),
        model_revision: "test-current".into(),
        workspace_id: Some(WorkspaceId::new(ws)),
        document_ids: None,
        tenant_id: None,
        modalities: None,
        filter_ids: None,
        vector_type: Some("entity".into()),
        embedding: w3::make_embedding(768, 40),
        limit: 5,
    };
    let by_uuid = index
        .search(
            EmbeddingFamily::Entity,
            &VectorQuery {
                filter_ids: Some(vec![ids[0].0.to_string()]),
                ..base.clone()
            },
        )
        .await
        .expect("uuid search");
    assert_eq!(by_uuid.len(), 1);
    assert_eq!(by_uuid[0].legacy_id, format!("entity:{}", ids[0].1));

    let by_name = index
        .search(
            EmbeddingFamily::Entity,
            &VectorQuery {
                filter_ids: Some(vec![format!("entity:{}", ids[1].1)]),
                ..base.clone()
            },
        )
        .await
        .expect("name search");
    assert_eq!(by_name.len(), 1);
    assert_eq!(by_name[0].legacy_id, format!("entity:{}", ids[1].1));

    let by_document = index
        .search(
            EmbeddingFamily::Entity,
            &VectorQuery {
                document_ids: Some(vec![doc_a]),
                ..base.clone()
            },
        )
        .await
        .expect("document search");
    assert_eq!(by_document.len(), 1);
    assert_eq!(by_document[0].legacy_id, format!("entity:{}", ids[0].1));

    let excluded = index
        .search(
            EmbeddingFamily::Entity,
            &VectorQuery {
                document_ids: Some(vec![doc_a]),
                filter_ids: Some(vec![ids[1].0.to_string()]),
                ..base
            },
        )
        .await
        .expect("id outside the document");
    assert!(excluded.is_empty());

    w3::cleanup_workspace(&pool, ws).await;
}
