//! E2E tests against a live EdgeQuake backend.
//!
//! Run with: `cargo test --test e2e_tests --features e2e`
//! Requires a running backend at EDGEQUAKE_BASE_URL (default: http://localhost:8080).
//!
//! These tests validate that SDK types and paths match the real API.

#![cfg(feature = "e2e")]

use edgequake_sdk::types::chat::ChatCompletionRequest;
use edgequake_sdk::types::documents::{DocumentListQuery, UploadDocumentRequest};
use edgequake_sdk::types::query::QueryRequest;
use edgequake_sdk::*;

fn e2e_client() -> Option<EdgeQuakeClient> {
    let base_url = match std::env::var("EDGEQUAKE_E2E_URL") {
        Ok(url) if !url.is_empty() => url,
        _ => return None,
    };
    let mut builder = EdgeQuakeClient::builder().base_url(&base_url);

    if let Ok(key) = std::env::var("EDGEQUAKE_API_KEY") {
        builder = builder.api_key(&key);
    }
    // WHY: Default migration-created tenant/user always available for E2E
    let tid = std::env::var("EDGEQUAKE_TENANT_ID")
        .unwrap_or_else(|_| "00000000-0000-0000-0000-000000000002".into());
    builder = builder.tenant_id(&tid);

    let uid = std::env::var("EDGEQUAKE_USER_ID")
        .unwrap_or_else(|_| "00000000-0000-0000-0000-000000000001".into());
    builder = builder.user_id(&uid);

    if let Ok(wid) = std::env::var("EDGEQUAKE_WORKSPACE_ID") {
        builder = builder.workspace_id(&wid);
    }

    builder
        .max_retries(0)
        .build()
        .ok()
}

// ── Health ───────────────────────────────────────────────────────

#[tokio::test]
async fn e2e_health() {
    let Some(c) = e2e_client() else {
        eprintln!("skip: EDGEQUAKE_E2E_URL not set");
        return;
    };
    let h = c.health().check().await.unwrap();
    assert_eq!(h.status, "healthy");
    println!(
        "Health: status={} version={:?} storage={:?}",
        h.status, h.version, h.storage_mode
    );
}

// ── Documents ────────────────────────────────────────────────────

#[tokio::test]
async fn e2e_documents_list() {
    let Some(c) = e2e_client() else {
        eprintln!("skip: EDGEQUAKE_E2E_URL not set");
        return;
    };
    let docs = c.documents().list().await.unwrap();
    println!("Documents: {} items", docs.documents.len());
}

// ── Graph ────────────────────────────────────────────────────────

#[tokio::test]
async fn e2e_graph_get() {
    let Some(c) = e2e_client() else {
        eprintln!("skip: EDGEQUAKE_E2E_URL not set");
        return;
    };
    let g = c.graph().get().await.unwrap();
    println!("Graph: {} nodes, {} edges", g.nodes.len(), g.edges.len());
}

#[tokio::test]
async fn e2e_graph_search() {
    let Some(c) = e2e_client() else {
        eprintln!("skip: EDGEQUAKE_E2E_URL not set");
        return;
    };
    let r = c.graph().search("test").await.unwrap();
    println!("Search: {} nodes found", r.nodes.len());
}

// ── Entities ─────────────────────────────────────────────────────

#[tokio::test]
async fn e2e_entities_list_and_create() {
    let Some(c) = e2e_client() else {
        eprintln!("skip: EDGEQUAKE_E2E_URL not set");
        return;
    };

    // Cleanup any leftover entity from previous runs.
    let _ = c.entities().delete("E2E_RUST_TEST").await;

    let list = c.entities().list().await.unwrap();
    println!("Entities: {} total (page {})", list.total, list.page);

    // Create
    let req = types::graph::CreateEntityRequest {
        entity_name: "E2E_RUST_TEST".into(),
        entity_type: "TEST".into(),
        description: "Created by Rust SDK E2E test".into(),
        source_id: "manual_entry".into(),
        metadata: None,
    };
    let created = c.entities().create(&req).await.unwrap();
    assert_eq!(created.status, "success");
    println!(
        "Created: {:?} status={}",
        created.entity.as_ref().map(|e| &e.entity_name),
        created.status
    );

    // Exists
    let exists = c.entities().exists("E2E_RUST_TEST").await.unwrap();
    assert!(exists.exists, "entity should exist after creation");

    // Get (detail)
    let detail = c.entities().get("E2E_RUST_TEST").await.unwrap();
    assert_eq!(detail.entity.entity_name, "E2E_RUST_TEST");
    println!(
        "Detail: entity_name={} degree={:?}",
        detail.entity.entity_name, detail.entity.degree
    );

    // Delete
    let del_result = c.entities().delete("E2E_RUST_TEST").await;
    if let Err(e) = &del_result {
        println!("Warning: delete failed: {e}");
    }
}

// ── Relationships ────────────────────────────────────────────────

#[tokio::test]
async fn e2e_relationships_list() {
    let Some(c) = e2e_client() else {
        eprintln!("skip: EDGEQUAKE_E2E_URL not set");
        return;
    };
    let r = c.relationships().list().await.unwrap();
    println!("Relationships: {} total (page {})", r.total, r.page);
}

// ── Query ────────────────────────────────────────────────────────

#[tokio::test]
async fn e2e_query_execute() {
    let Some(c) = e2e_client() else {
        eprintln!("skip: EDGEQUAKE_E2E_URL not set");
        return;
    };
    let req = types::query::QueryRequest {
        query: "What is EdgeQuake?".into(),
        top_k: Some(3),
        ..Default::default()
    };
    let r = c.query().execute(&req).await.unwrap();
    println!(
        "Query: answer_len={} sources={}",
        r.answer.as_deref().unwrap_or("").len(),
        r.sources.len()
    );
}

// ── Chat ─────────────────────────────────────────────────────────

#[tokio::test]
async fn e2e_chat_completions() {
    let Some(c) = e2e_client() else {
        eprintln!("skip: EDGEQUAKE_E2E_URL not set");
        return;
    };
    // WHY: EdgeQuake chat API uses `message` (singular string), not messages array
    let req = types::chat::ChatCompletionRequest {
        message: "Say hello".into(),
        stream: Some(false),
        ..Default::default()
    };
    match c.chat().completions(&req).await {
        Ok(r) => println!(
            "Chat: content_len={} conversation_id={:?}",
            r.content.as_deref().unwrap_or("").len(),
            r.conversation_id
        ),
        Err(e) => println!("Chat: error (may need LLM configured): {e}"),
    }
}

// ── Tasks ────────────────────────────────────────────────────────

#[tokio::test]
async fn e2e_tasks_list() {
    let Some(c) = e2e_client() else {
        eprintln!("skip: EDGEQUAKE_E2E_URL not set");
        return;
    };
    let r = c.tasks().list().await.unwrap();
    println!("Tasks: {} total", r.total);
}

// ── Pipeline ─────────────────────────────────────────────────────

#[tokio::test]
async fn e2e_pipeline_status() {
    let Some(c) = e2e_client() else {
        eprintln!("skip: EDGEQUAKE_E2E_URL not set");
        return;
    };
    let r = c.pipeline().status().await.unwrap();
    println!(
        "Pipeline: is_busy={} pending={} processing={}",
        r.is_busy, r.pending_tasks, r.processing_tasks
    );
}

#[tokio::test]
async fn e2e_pipeline_metrics() {
    let Some(c) = e2e_client() else {
        eprintln!("skip: EDGEQUAKE_E2E_URL not set");
        return;
    };
    let r = c.pipeline().metrics().await.unwrap();
    println!(
        "Queue metrics: depth={} processing={}",
        r.queue_depth, r.processing
    );
}

// ── Costs ────────────────────────────────────────────────────────

#[tokio::test]
async fn e2e_costs_summary() {
    let Some(c) = e2e_client() else {
        eprintln!("skip: EDGEQUAKE_E2E_URL not set");
        return;
    };
    let r = c.costs().summary().await.unwrap();
    println!(
        "Costs: total=${:.4} tokens={}",
        r.total_cost_usd, r.total_tokens
    );
}

#[tokio::test]
async fn e2e_costs_budget() {
    let Some(c) = e2e_client() else {
        eprintln!("skip: EDGEQUAKE_E2E_URL not set");
        return;
    };
    let r = c.costs().budget().await.unwrap();
    println!("Budget: current_spend=${:.4}", r.current_spend_usd);
}

// ── Models ───────────────────────────────────────────────────────

#[tokio::test]
async fn e2e_models_list() {
    let Some(c) = e2e_client() else {
        eprintln!("skip: EDGEQUAKE_E2E_URL not set");
        return;
    };
    let catalog = c.models().list().await.unwrap();
    println!("Models: {} providers", catalog.providers.len());
    for p in &catalog.providers {
        println!(
            "  - {} ({:?}): {} models",
            p.name,
            p.display_name,
            p.models.len()
        );
    }
}

#[tokio::test]
async fn e2e_models_providers_health() {
    let Some(c) = e2e_client() else {
        eprintln!("skip: EDGEQUAKE_E2E_URL not set");
        return;
    };
    let health = c.models().providers_health().await.unwrap();
    println!("Provider health: {} providers", health.len());
}

// ── Tenants ──────────────────────────────────────────────────────

#[tokio::test]
async fn e2e_tenants_list() {
    let Some(c) = e2e_client() else {
        eprintln!("skip: EDGEQUAKE_E2E_URL not set");
        return;
    };
    let resp = c.tenants().list().await.unwrap();
    println!("Tenants: {} items", resp.items.len());
    for t in &resp.items {
        println!("  - {} (slug={:?} plan={:?})", t.name, t.slug, t.plan);
    }
}

// ── Lineage ──────────────────────────────────────────────────────

#[tokio::test]
async fn e2e_lineage_for_entity() {
    let Some(c) = e2e_client() else {
        eprintln!("skip: EDGEQUAKE_E2E_URL not set");
        return;
    };
    // Expected to 404 for non-existent entity
    match c.provenance().for_entity("NONEXISTENT_ENTITY").await {
        Ok(r) => println!("Provenance: entity={}", r.entity_name),
        Err(e) => println!("Lineage: expected error for nonexistent entity: {e}"),
    }
}

// ── Document Lineage (OODA-21) ───────────────────────────────────

#[tokio::test]
async fn e2e_document_lineage() {
    let Some(c) = e2e_client() else {
        eprintln!("skip: EDGEQUAKE_E2E_URL not set");
        return;
    };
    let docs = c.documents().list().await.unwrap();
    if let Some(doc) = docs.documents.first() {
        match c.documents().get_lineage(&doc.id).await {
            Ok(lineage) => {
                println!(
                    "Document lineage: doc={} metadata={:?} lineage={:?}",
                    lineage.document_id,
                    lineage.metadata.is_some(),
                    lineage.lineage.is_some()
                );
                assert_eq!(lineage.document_id, doc.id);
            }
            Err(e) => println!("Document lineage: error (may not have lineage data): {e}"),
        }
    } else {
        println!("Document lineage: skipped (no documents)");
    }
}

#[tokio::test]
async fn e2e_document_metadata() {
    let Some(c) = e2e_client() else {
        eprintln!("skip: EDGEQUAKE_E2E_URL not set");
        return;
    };
    let docs = c.documents().list().await.unwrap();
    if let Some(doc) = docs.documents.first() {
        match c.documents().get_metadata(&doc.id).await {
            Ok(metadata) => {
                println!(
                    "Document metadata: {} keys",
                    metadata.as_object().map(|o| o.len()).unwrap_or(0)
                );
                // Metadata should be an object with at least an id
                assert!(metadata.is_object(), "metadata should be a JSON object");
            }
            Err(e) => println!("Document metadata: error: {e}"),
        }
    } else {
        println!("Document metadata: skipped (no documents)");
    }
}

#[tokio::test]
async fn e2e_chunk_lineage() {
    let Some(c) = e2e_client() else {
        eprintln!("skip: EDGEQUAKE_E2E_URL not set");
        return;
    };
    let docs = c.documents().list().await.unwrap();
    if let Some(doc) = docs.documents.first() {
        // Try chunk-0 for this document
        let chunk_id = format!("{}-chunk-0", doc.id);
        match c.chunks().get_lineage(&chunk_id).await {
            Ok(lineage) => {
                println!(
                    "Chunk lineage: chunk={} doc={:?} entities={}",
                    lineage.chunk_id,
                    lineage.document_id,
                    lineage.entity_names.len()
                );
                assert_eq!(lineage.document_id, Some(doc.id.clone()));
            }
            Err(e) => println!("Chunk lineage: error (chunk may not exist): {e}"),
        }
    } else {
        println!("Chunk lineage: skipped (no documents)");
    }
}

#[tokio::test]
async fn e2e_shared_live_scenario() {
    let Some(c) = e2e_client() else {
        eprintln!("skip: EDGEQUAKE_E2E_URL not set");
        return;
    };
    let health = c.health().check().await.unwrap();
    assert_eq!(health.status, "healthy");
    let setup = c.operations().setup_status(None, None).await.unwrap();
    assert!(setup.contains("needs_setup"), "{setup}");
    let socket = c.operations().ws_pipeline_progress();
    assert!(socket.starts_with("ws") && socket.ends_with("/ws/pipeline/progress"), "{socket}");
    let listed = c
        .documents()
        .list_with_query(&DocumentListQuery {
            page: Some(1),
            page_size: Some(5),
            ..Default::default()
        })
        .await
        .unwrap();
    for item in &listed.documents {
        let _ = &item.display_status;
    }
    let mut upload = UploadDocumentRequest::text("Shared SDK live scenario.");
    upload.title = Some("shared-live".into());
    let uploaded = c.documents().upload(&upload).await.unwrap();
    assert!(!uploaded.id.is_empty());
    c.graph().get().await.unwrap();
    c.entities().list().await.unwrap();
    c.query()
        .execute(&QueryRequest {
            query: "What is a knowledge graph?".into(),
            top_k: Some(3),
            ..Default::default()
        })
        .await
        .unwrap();
    c.connections().list().await.unwrap();
    c.chat()
        .completions(&ChatCompletionRequest {
            message: "ping".into(),
            top_k: Some(2),
            ..Default::default()
        })
        .await
        .unwrap();
}
