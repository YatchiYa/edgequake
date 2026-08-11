//! SPEC-115 dual-arm EdgeQuake extract with Mistral Small (pipeline-only).
//!
//! Uses `build_ingestion_pipeline` + `process` so we measure chunk N / mention M /
//! unique names U without SPEC-091 typed vector persistence (memory path).
//!
//! ```bash
//! export MISTRAL_API_KEY=...
//! cd edgequake && cargo run --example spec115_mistral_ingest --release
//! ```

use std::collections::HashSet;
use std::env;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use edgequake_llm::{EmbeddingProvider, LLMProvider, MistralProvider};
use edgequake_pipeline::prompts::EntityExtractionSchema;
use edgequake_pipeline::{build_ingestion_pipeline, ExtractionResult, IngestionPipelineOptions};
use serde_json::json;

fn unique_entity_names(extractions: &[ExtractionResult]) -> usize {
    let mut set = HashSet::new();
    for ex in extractions {
        for e in &ex.entities {
            let key = e.name.trim().to_uppercase().replace(' ', "_");
            if !key.is_empty() {
                set.insert(key);
            }
        }
    }
    set.len()
}

fn unique_relation_keys(extractions: &[ExtractionResult]) -> usize {
    let mut set = HashSet::new();
    for ex in extractions {
        for r in &ex.relationships {
            let s = r.source.trim().to_uppercase().replace(' ', "_");
            let t = r.target.trim().to_uppercase().replace(' ', "_");
            let rel = r.relation_type.trim().to_uppercase().replace(' ', "_");
            if !s.is_empty() && !t.is_empty() {
                set.insert(format!("{s}|{rel}|{t}"));
            }
        }
    }
    set.len()
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("info,edgequake_pipeline=info")
        .init();

    let key = env::var("MISTRAL_API_KEY")
        .map_err(|_| anyhow::anyhow!("MISTRAL_API_KEY required for SPEC-115 EdgeQuake live arm"))?;
    if key.is_empty() {
        anyhow::bail!("MISTRAL_API_KEY empty");
    }

    env::set_var(
        "MISTRAL_MODEL",
        env::var("MISTRAL_MODEL").unwrap_or_else(|_| "mistral-small-latest".into()),
    );
    env::set_var(
        "MISTRAL_EMBEDDING_MODEL",
        env::var("MISTRAL_EMBEDDING_MODEL").unwrap_or_else(|_| "mistral-embed".into()),
    );

    let repo = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("..");
    let gold = repo.join("zz_test_docs/academic_papers/lighrag_2410.05779v3.pymupdf.gold.md");
    let text = std::fs::read_to_string(&gold)
        .map_err(|e| anyhow::anyhow!("read {}: {e}", gold.display()))?;

    let out_dir = repo.join("specs/115-extraction-chunk-kg/measurements");
    std::fs::create_dir_all(&out_dir)?;

    let only = env::var("SPEC115_ONLY_ARM").unwrap_or_default();
    let arm_defs: Vec<(&str, bool)> = match only.as_str() {
        "A" | "a" => vec![("A", false)],
        "B" | "b" => vec![("B", true)],
        _ => vec![("B", true), ("A", false)],
    };

    let mut arms = Vec::new();
    // Merge prior JSON if present when running a single arm.
    if let Ok(prev) = std::fs::read_to_string(out_dir.join("edgequake_live.json")) {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&prev) {
            if let Some(arr) = v.get("arms").and_then(|a| a.as_array()) {
                for item in arr {
                    if let Some(a) = item.get("arm").and_then(|x| x.as_str()) {
                        if only.is_empty() || a.eq_ignore_ascii_case(&only) {
                            continue;
                        }
                        arms.push(item.clone());
                    }
                }
            }
        }
    }

    for (arm, adaptive) in arm_defs {
        println!("\n=== SPEC-115 EdgeQuake arm {arm} adaptive={adaptive} ===");
        // SAFETY: sequential arms only.
        unsafe {
            if adaptive {
                env::set_var("EDGEQUAKE_ADAPTIVE_CHUNKING", "1");
            } else {
                env::set_var("EDGEQUAKE_ADAPTIVE_CHUNKING", "0");
                env::set_var("EDGEQUAKE_CHUNK_SIZE", "1200");
                env::set_var("EDGEQUAKE_CHUNK_OVERLAP", "100");
            }
        }

        let llm_model = env::var("MISTRAL_MODEL").unwrap();
        let emb_model = env::var("MISTRAL_EMBEDDING_MODEL").unwrap();
        let provider = Arc::new(
            MistralProvider::from_env()?
                .with_model(&llm_model)
                .with_embedding_model(&emb_model),
        );
        let llm: Arc<dyn LLMProvider> = provider.clone();
        let embedding: Arc<dyn EmbeddingProvider> = provider.clone();

        let opts = IngestionPipelineOptions::from_document_size(text.len())
            .with_gleaning(true, 1)
            .with_llm_provider("mistral");
        let pipeline = build_ingestion_pipeline(
            llm,
            embedding,
            EntityExtractionSchema::server_default(),
            opts,
        );

        let t0 = Instant::now();
        let result = pipeline
            .process(&format!("spec115-s1md-{arm}"), &text)
            .await?;
        let elapsed = t0.elapsed().as_secs_f64();

        let u_nodes = unique_entity_names(&result.extractions);
        let u_edges = unique_relation_keys(&result.extractions);
        let avg_chunk_tokens = if result.chunks.is_empty() {
            0.0
        } else {
            result
                .chunks
                .iter()
                .map(|c| c.token_count as f64)
                .sum::<f64>()
                / result.chunks.len() as f64
        };

        let row = json!({
            "arm": arm,
            "sut": "edgequake",
            "mode": "live-mistral-pipeline",
            "sample_id": "S1-md",
            "adaptive": adaptive,
            "chars": text.len(),
            "chunk_count": result.stats.chunk_count,
            "chunk_count_vec": result.chunks.len(),
            "avg_chunk_tokens": (avg_chunk_tokens * 10.0).round() / 10.0,
            "mention_entities": result.stats.entity_count,
            "mention_relations": result.stats.relationship_count,
            "unique_nodes_name_norm": u_nodes,
            "unique_edges_name_norm": u_edges,
            "successful_chunks": result.stats.successful_chunks,
            "failed_chunks": result.stats.failed_chunks,
            "elapsed_s": (elapsed * 100.0).round() / 100.0,
            "llm_model": llm_model,
            "embed_model": emb_model,
            "note": "U via uppercase+underscore name normalize (approx merger EntityId); not AGE.",
        });
        println!("{}", serde_json::to_string_pretty(&row)?);
        arms.push(row);
    }

    let payload = json!({
        "utc": chrono::Utc::now().to_rfc3339(),
        "arms": arms,
        "note": "Pipeline-only path (no vector persist). Compare unique_nodes_name_norm to LightRAG unique_nodes.",
    });
    let out = out_dir.join("edgequake_live.json");
    std::fs::write(&out, serde_json::to_string_pretty(&payload)? + "\n")?;
    println!("\nWrote {}", out.display());
    Ok(())
}
