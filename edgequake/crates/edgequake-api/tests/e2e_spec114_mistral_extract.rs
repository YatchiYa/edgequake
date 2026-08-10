//! SPEC-114 G-114-17 — Live Mistral extract soft EC matrix.
//!
//! ```bash
//! export DATABASE_URL=postgres://edgequake:edgequake_secret@localhost:5432/edgequake
//! export MISTRAL_API_KEY=...
//! cargo test -p edgequake-api --features postgres --test e2e_spec114_mistral_extract -- --ignored --nocapture
//! # or: make spec114-e2e-mistral-extract
//! ```

#![cfg(feature = "postgres")]

mod common;
use common::spec013_postgres;
use common::spec114_live_extract::{self, LiveProviderKind, LiveScenario};

use serial_test::serial;

fn mistral_available() -> bool {
    std::env::var("MISTRAL_API_KEY")
        .map(|k| !k.is_empty())
        .unwrap_or(false)
        && spec013_postgres::database_url().is_some()
}

#[tokio::test]
#[ignore = "requires MISTRAL_API_KEY + DATABASE_URL — live Mistral extract matrix"]
#[serial]
async fn spec114_mistral_extract_live_matrix() {
    if !mistral_available() {
        eprintln!("SKIP: MISTRAL_API_KEY or DATABASE_URL not set");
        return;
    }
    let Some(app) = spec013_postgres::create_postgres_mistral_app_or_skip().await else {
        eprintln!("SKIP: no PostgreSQL DATABASE_URL / MISTRAL_API_KEY configured");
        return;
    };
    spec114_live_extract::run_full_live_matrix(&app, LiveProviderKind::Mistral).await;
}

#[tokio::test]
#[ignore = "requires MISTRAL_API_KEY + DATABASE_URL — live Mistral happy path"]
#[serial]
async fn spec114_mistral_extract_allowed_entity_and_relation() {
    if !mistral_available() {
        eprintln!("SKIP: MISTRAL_API_KEY or DATABASE_URL not set");
        return;
    }
    let Some(app) = spec013_postgres::create_postgres_mistral_app_or_skip().await else {
        eprintln!("SKIP: no PostgreSQL DATABASE_URL / MISTRAL_API_KEY configured");
        return;
    };
    spec114_live_extract::run_live_scenario(
        &app,
        LiveProviderKind::Mistral,
        LiveScenario::HappyDualAllowlist,
    )
    .await;
}
