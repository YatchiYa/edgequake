//! SPEC-114 G-114-19 — Live Ollama extract soft EC matrix (`qwen3.6:35b-a3b`).
//!
//! ```bash
//! export DATABASE_URL=postgres://edgequake:edgequake_secret@localhost:5432/edgequake
//! ollama pull qwen3.6:35b-a3b
//! ollama pull embeddinggemma:latest   # or nomic-embed-text
//! cargo test -p edgequake-api --features postgres --test e2e_spec114_ollama_extract -- --ignored --nocapture
//! # or: make spec114-e2e-ollama-extract
//! ```

#![cfg(feature = "postgres")]

mod common;
use common::spec013_postgres;
use common::spec114_live_extract::{self, LiveProviderKind, LiveScenario};

use serial_test::serial;

#[tokio::test]
#[ignore = "requires Ollama qwen3.6:35b-a3b + embed + DATABASE_URL — live Ollama extract matrix"]
#[serial]
async fn spec114_ollama_extract_live_matrix() {
    if !spec013_postgres::ollama_live_extract_available().await {
        eprintln!(
            "SKIP: need DATABASE_URL + ollama pull {} + embed model",
            spec013_postgres::OLLAMA_LLM_MODEL
        );
        return;
    }
    let Some(app) = spec013_postgres::create_postgres_ollama_app_or_skip().await else {
        eprintln!("SKIP: PostgreSQL Ollama app not available");
        return;
    };
    spec114_live_extract::run_full_live_matrix(&app, LiveProviderKind::Ollama).await;
}

#[tokio::test]
#[ignore = "requires Ollama qwen3.6:35b-a3b + embed + DATABASE_URL — live Ollama happy path"]
#[serial]
async fn spec114_ollama_extract_allowed_entity_and_relation() {
    if !spec013_postgres::ollama_live_extract_available().await {
        eprintln!(
            "SKIP: need DATABASE_URL + ollama pull {} + embed model",
            spec013_postgres::OLLAMA_LLM_MODEL
        );
        return;
    }
    let Some(app) = spec013_postgres::create_postgres_ollama_app_or_skip().await else {
        eprintln!("SKIP: PostgreSQL Ollama app not available");
        return;
    };
    spec114_live_extract::run_live_scenario(
        &app,
        LiveProviderKind::Ollama,
        LiveScenario::HappyDualAllowlist,
    )
    .await;
}
