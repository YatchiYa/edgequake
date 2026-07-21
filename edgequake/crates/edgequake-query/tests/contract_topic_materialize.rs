//! 042/045 — KV materialize topic CONTENT chunks into Mix before CE.

use std::sync::Arc;

use edgequake_query::context::RetrievedChunk;
use edgequake_query::topic_entity_admit::{
    materialize_topic_chunks_into_mix, question_content_phrases, META_TOPIC_ADMIT_CHUNK_IDS,
};
use edgequake_storage::adapters::memory::MemoryKVStorage;
use edgequake_storage::traits::KVStorage;

#[tokio::test]
async fn materialize_injects_missing_topic_body_from_kv() {
    std::env::set_var("EDGEQUAKE_TOPIC_MATERIALIZE", "1");
    std::env::remove_var("EDGEQUAKE_TOPIC_MATERIALIZE_CONTENT");
    let kv: Arc<dyn KVStorage> = Arc::new(MemoryKVStorage::new("mat"));
    kv.upsert(&[(
        "topic-chunk".to_string(),
        serde_json::json!({ "content": "About bone cancers staging and TNM." }),
    )])
    .await
    .unwrap();

    let mut chunks = vec![
        RetrievedChunk::new("noise-a", "cervical cancer staging", 0.9),
        RetrievedChunk::new("noise-b", "anal cancer basics", 0.8),
    ];
    let topic_ids = vec!["topic-chunk".into(), "missing".into()];

    let injected = materialize_topic_chunks_into_mix(
        Some(kv.as_ref()),
        &mut chunks,
        &topic_ids,
        4,
        30,
        "How are bone cancers staged?",
    )
    .await;

    assert_eq!(injected, vec!["topic-chunk".to_string()]);
    assert_eq!(chunks[0].id, "topic-chunk");
    assert!(chunks[0].content.contains("bone cancers"));
    assert_eq!(chunks.len(), 3);

    assert_eq!(META_TOPIC_ADMIT_CHUNK_IDS, "topic_admit_chunk_ids");

    std::env::remove_var("EDGEQUAKE_TOPIC_MATERIALIZE");
}

#[tokio::test]
async fn content_gate_skips_offtopic_and_keeps_phrase_hit() {
    std::env::set_var("EDGEQUAKE_TOPIC_MATERIALIZE", "1");
    std::env::set_var("EDGEQUAKE_TOPIC_MATERIALIZE_CONTENT", "1");
    let kv: Arc<dyn KVStorage> = Arc::new(MemoryKVStorage::new("mat-c"));
    kv.upsert(&[
        (
            "offtopic".to_string(),
            serde_json::json!({ "content": "Cervical cancer staging uses FIGO." }),
        ),
        (
            "ontopic".to_string(),
            serde_json::json!({ "content": "Bone cancer staging uses TNM and grade." }),
        ),
    ])
    .await
    .unwrap();

    let mut chunks = vec![RetrievedChunk::new("noise-a", "unrelated", 0.9)];
    // Off-topic first in admit order — blind 042 would inject it; content gate skips.
    let topic_ids = vec!["offtopic".into(), "ontopic".into()];

    let injected = materialize_topic_chunks_into_mix(
        Some(kv.as_ref()),
        &mut chunks,
        &topic_ids,
        4,
        30,
        "How are bone cancers staged and what factors are considered?",
    )
    .await;

    assert_eq!(injected, vec!["ontopic".to_string()]);
    assert_eq!(chunks[0].id, "ontopic");
    assert!(chunks[0]
        .content
        .to_ascii_lowercase()
        .contains("bone cancer"));

    std::env::remove_var("EDGEQUAKE_TOPIC_MATERIALIZE");
    std::env::remove_var("EDGEQUAKE_TOPIC_MATERIALIZE_CONTENT");
}

#[test]
fn content_phrases_include_bone_cancers() {
    let phrases = question_content_phrases(
        "How are bone cancers staged and what factors are considered in determining the stage?",
    );
    assert!(
        phrases.iter().any(|p| p.contains("bone cancer")),
        "phrases={phrases:?}"
    );
}
