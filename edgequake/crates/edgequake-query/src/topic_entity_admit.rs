//! 038 — Mix topic-entity admission (Exploratory SELECT).
//!
//! First principles (037): topic entities can be AGE-linked yet lose entity-VDB
//! top-k to hub neighbors, so VECTOR never sees their `source_chunk_ids`.
//!
//! When `EDGEQUAKE_TOPIC_ENTITY_ADMIT=1` and intent is Exploratory:
//! 1. Build exact-name candidates from **question content bigrams** and
//!    multi-token low-level keywords (never bare unigrams like CANCER/STAGE).
//! 2. Look up those entities in the graph; prepend if they have source chunks.
//! 3. Record chunk ids in `context.metadata["topic_admit_chunk_ids"]` so VECTOR
//!    take can pin them into Mix C.
//!
//! One confound. No densify-all ingest. No dual-list / LR-budget.

use std::collections::HashSet;

use edgequake_storage::traits::GraphReadView;

use crate::context::QueryContext;
use crate::helpers::{build_entity_from_node, graph_entity_id_for_workspace};
use crate::keywords::{ExtractedKeywords, QueryIntent};

fn bare_entity_norm(name: &str) -> String {
    let bare = name.rsplit("::").next().unwrap_or(name);
    norm_entity_token(bare)
}

pub const META_TOPIC_ADMIT_CHUNK_IDS: &str = "topic_admit_chunk_ids";
pub const META_TOPIC_ADMIT_ENTITIES: &str = "topic_admit_entities";

/// Env gate (default off — Acc ladder `a1fpsel` only).
pub fn topic_entity_admit_enabled() -> bool {
    matches!(
        std::env::var("EDGEQUAKE_TOPIC_ENTITY_ADMIT")
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase()
            .as_str(),
        "1" | "true" | "yes" | "on"
    )
}

/// 039: CE/fuse protect for topic-admitted chunk ids (default off — `a1fpce`).
pub fn topic_ce_protect_enabled() -> bool {
    matches!(
        std::env::var("EDGEQUAKE_TOPIC_CE_PROTECT")
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase()
            .as_str(),
        "1" | "true" | "yes" | "on"
    )
}

/// 040: prefer topic-admitted ids when greedy-packing under token budget (`a1fptrunc`).
pub fn topic_trunc_protect_enabled() -> bool {
    matches!(
        std::env::var("EDGEQUAKE_TOPIC_TRUNC_PROTECT")
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase()
            .as_str(),
        "1" | "true" | "yes" | "on"
    )
}

/// Cap how many topic chunks jump the pack queue (default 4 — Acc tax guard).
pub fn topic_trunc_protect_max() -> usize {
    std::env::var("EDGEQUAKE_TOPIC_TRUNC_PROTECT_MAX")
        .ok()
        .and_then(|s| s.trim().parse::<usize>().ok())
        .filter(|&n| n > 0)
        .unwrap_or(4)
}

/// 042: KV materialize topic CONTENT chunks into Mix before CE (`a1fpmat`).
pub fn topic_materialize_enabled() -> bool {
    matches!(
        std::env::var("EDGEQUAKE_TOPIC_MATERIALIZE")
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase()
            .as_str(),
        "1" | "true" | "yes" | "on"
    )
}

/// Cap KV-materialized topic chunks (default 4).
pub fn topic_materialize_max() -> usize {
    std::env::var("EDGEQUAKE_TOPIC_MATERIALIZE_MAX")
        .ok()
        .and_then(|s| s.trim().parse::<usize>().ok())
        .filter(|&n| n > 0)
        .unwrap_or(4)
}

/// 045: only materialize KV bodies that contain a question content bigram.
///
/// Blind 042 inject of first-N admit ids Acc/Fact-taxed; CONTENT gate keeps
/// Sum-ER CE_GAP fix without off-topic noise (043 leftover).
pub fn topic_materialize_content_gate_enabled() -> bool {
    matches!(
        std::env::var("EDGEQUAKE_TOPIC_MATERIALIZE_CONTENT")
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase()
            .as_str(),
        "1" | "true" | "yes" | "on"
    )
}

/// 048: type-scope materialize (comma tokens, e.g. `summarize`).
///
/// Empty → always when `TOPIC_MATERIALIZE` on (042/045). Non-empty → only when
/// `question_type` lowercase contains a token. Missing type → skip (protect Fact).
pub fn topic_materialize_types_allow(question_type: Option<&str>) -> bool {
    let raw = std::env::var("EDGEQUAKE_TOPIC_MATERIALIZE_TYPES").unwrap_or_default();
    let tokens: Vec<String> = raw
        .split(',')
        .map(|s| s.trim().to_ascii_lowercase())
        .filter(|s| !s.is_empty())
        .collect();
    if tokens.is_empty() {
        return true;
    }
    let qt = question_type.unwrap_or("").trim().to_ascii_lowercase();
    if qt.is_empty() {
        return false;
    }
    tokens.iter().any(|t| qt.contains(t.as_str()))
}

/// CE/trunc survival for topic ids when materialize or explicit protect is on.
pub fn topic_survival_enabled() -> bool {
    topic_ce_protect_enabled() || topic_trunc_protect_enabled() || topic_materialize_enabled()
}

/// Move up to `max_protect` in-list topic chunks to the front before truncate.
///
/// 040 First principles: answer-in-context under a token budget — greedy
/// `truncate_chunks` keeps prefix order; CE survivors that are topic-admitted
/// must occupy early pack slots or they never reach C.
/// 042: also runs when `TOPIC_MATERIALIZE` is on.
pub fn prefer_topic_chunks_for_trunc(
    chunks: &mut Vec<crate::context::RetrievedChunk>,
    topic_ids: &[String],
    max_protect: usize,
) {
    if !(topic_trunc_protect_enabled() || topic_materialize_enabled())
        || topic_ids.is_empty()
        || max_protect == 0
        || chunks.is_empty()
    {
        return;
    }
    let want: HashSet<&str> = topic_ids.iter().map(|s| s.as_str()).collect();
    let mut protected = Vec::with_capacity(max_protect.min(chunks.len()));
    let mut rest = Vec::with_capacity(chunks.len());
    for c in chunks.drain(..) {
        if protected.len() < max_protect && want.contains(c.id.as_str()) {
            protected.push(c);
        } else {
            rest.push(c);
        }
    }
    let n = protected.len();
    chunks.append(&mut protected);
    chunks.append(&mut rest);
    if n > 0 {
        tracing::info!(
            topic_pack = n,
            max_protect,
            mix_len = chunks.len(),
            "040 topic_trunc_protect: preferred topic chunks for greedy pack"
        );
    }
}

/// Union `topic_admit_*` metadata from Mix arms onto `merged` (fuse hole fix).
pub fn merge_topic_admit_metadata(merged: &mut QueryContext, arms: &[&QueryContext]) {
    let mut chunk_ids: Vec<String> = topic_chunk_ids_from_context(merged);
    let mut entities: Vec<String> = merged
        .metadata
        .get(META_TOPIC_ADMIT_ENTITIES)
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|x| x.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    for arm in arms {
        for id in topic_chunk_ids_from_context(arm) {
            if !chunk_ids.iter().any(|x| x == &id) {
                chunk_ids.push(id);
            }
        }
        if let Some(arr) = arm
            .metadata
            .get(META_TOPIC_ADMIT_ENTITIES)
            .and_then(|v| v.as_array())
        {
            for x in arr {
                if let Some(s) = x.as_str() {
                    if !entities.iter().any(|e| e == s) {
                        entities.push(s.to_string());
                    }
                }
            }
        }
    }

    if !chunk_ids.is_empty() {
        merged.metadata.insert(
            META_TOPIC_ADMIT_CHUNK_IDS.to_string(),
            serde_json::json!(chunk_ids),
        );
    }
    if !entities.is_empty() {
        merged.metadata.insert(
            META_TOPIC_ADMIT_ENTITIES.to_string(),
            serde_json::json!(entities),
        );
    }
}

/// 042/045: fetch missing topic chunk bodies from KV and prepend into Mix before CE.
///
/// When [`topic_materialize_content_gate_enabled`], scan a wider admit pool and
/// keep only bodies that contain a question content bigram (043/045).
///
/// Returns topic ids (capped) that now have non-empty content in `chunks`.
pub async fn materialize_topic_chunks_into_mix(
    kv: Option<&dyn edgequake_storage::traits::KVStorage>,
    chunks: &mut Vec<crate::context::RetrievedChunk>,
    topic_ids: &[String],
    max_materialize: usize,
    max_mix: usize,
    query: &str,
) -> Vec<String> {
    if !topic_materialize_enabled() || topic_ids.is_empty() || max_materialize == 0 {
        return Vec::new();
    }
    let Some(kv) = kv else {
        tracing::warn!("042 topic_materialize: no KV storage — skip");
        return Vec::new();
    };

    let content_gate = topic_materialize_content_gate_enabled();
    let phrases = if content_gate {
        question_content_phrases(query)
    } else {
        Vec::new()
    };
    if content_gate && phrases.is_empty() {
        tracing::info!("045 topic_materialize_content: no content phrases — skip");
        return Vec::new();
    }

    // Blind 042: first N. Content gate: scan wider, keep only phrase hits ≤ N.
    let scan_n = if content_gate {
        (max_materialize.saturating_mul(8).max(24)).min(topic_ids.len())
    } else {
        max_materialize.min(topic_ids.len())
    };
    let scan: Vec<String> = topic_ids.iter().take(scan_n).cloned().collect();

    let have: HashSet<&str> = chunks
        .iter()
        .filter(|c| !c.content.is_empty())
        .map(|c| c.id.as_str())
        .collect();

    let need: Vec<String> = scan
        .iter()
        .filter(|id| !have.contains(id.as_str()))
        .cloned()
        .collect();

    let fetched = if need.is_empty() {
        std::collections::HashMap::new()
    } else {
        edgequake_storage::chunk_content::batch_fetch_chunk_contents(kv, &need)
            .await
            .unwrap_or_default()
    };

    let mut ready: Vec<crate::context::RetrievedChunk> = Vec::new();
    for id in &scan {
        if ready.len() >= max_materialize {
            break;
        }
        let body_owned: Option<String> = if let Some(existing) = chunks.iter().find(|c| c.id == *id)
        {
            if !existing.content.is_empty() {
                Some(existing.content.clone())
            } else {
                fetched.get(id).cloned()
            }
        } else {
            fetched.get(id).cloned()
        };
        let Some(body) = body_owned else {
            continue;
        };
        if content_gate && !body_hits_content_phrases(&body, &phrases) {
            continue;
        }
        if let Some(existing) = chunks.iter().find(|c| c.id == *id) {
            let mut c = existing.clone();
            c.content = body;
            ready.push(c);
        } else {
            ready.push(crate::context::RetrievedChunk::new(id.clone(), body, 1.0));
        }
    }

    if ready.is_empty() {
        if content_gate {
            tracing::info!(
                scanned = scan.len(),
                phrases = phrases.len(),
                "045 topic_materialize_content: 0 phrase-hitting bodies"
            );
        }
        return Vec::new();
    }

    let injected: Vec<String> = ready.iter().map(|c| c.id.clone()).collect();
    let want: HashSet<&str> = injected.iter().map(|s| s.as_str()).collect();
    let rest: Vec<_> = chunks
        .drain(..)
        .filter(|c| !want.contains(c.id.as_str()))
        .collect();
    chunks.extend(ready);
    chunks.extend(rest);
    if max_mix > 0 && chunks.len() > max_mix {
        chunks.truncate(max_mix);
    }
    tracing::info!(
        materialized = injected.len(),
        fetched = fetched.len(),
        mix_len = chunks.len(),
        max_materialize,
        content_gate,
        scanned = scan.len(),
        "042/045 topic_materialize: KV bodies injected into Mix before CE"
    );
    injected
}

/// Force topic-admitted chunks into fused Mix[:max_chunks] when present in lookup.
pub fn fuse_protect_topic_chunks(
    chunks: &mut Vec<crate::context::RetrievedChunk>,
    lookup: &std::collections::HashMap<String, crate::context::RetrievedChunk>,
    topic_ids: &[String],
    max_chunks: usize,
) {
    if !topic_survival_enabled() || topic_ids.is_empty() || max_chunks == 0 {
        return;
    }
    let have: HashSet<&str> = chunks.iter().map(|c| c.id.as_str()).collect();
    let mut missing: Vec<crate::context::RetrievedChunk> = Vec::new();
    for id in topic_ids {
        if have.contains(id.as_str()) {
            continue;
        }
        if let Some(c) = lookup.get(id) {
            missing.push(c.clone());
        }
    }
    if missing.is_empty() {
        return;
    }
    // Prepend missing topic chunks, then keep max_chunks (dedupe).
    let mut out = missing;
    let mut seen: HashSet<String> = out.iter().map(|c| c.id.clone()).collect();
    for c in chunks.drain(..) {
        if seen.insert(c.id.clone()) {
            out.push(c);
        }
    }
    out.truncate(max_chunks);
    *chunks = out;
    tracing::info!(
        topic_forced = topic_ids.len(),
        mix_len = chunks.len(),
        "039/042 topic_survival: forced topic chunks into fused Mix"
    );
}

fn intent_allows(intent: QueryIntent) -> bool {
    matches!(intent, QueryIntent::Exploratory)
}

/// Closed-class WH stopwords for content-word extraction (not domain needles).
const Q_STOP: &[&str] = &[
    "how",
    "are",
    "what",
    "which",
    "the",
    "and",
    "for",
    "with",
    "in",
    "of",
    "to",
    "a",
    "an",
    "is",
    "their",
    "this",
    "that",
    "from",
    "into",
    "used",
    "most",
    "main",
    "does",
    "do",
    "can",
    "when",
    "where",
    "who",
    "why",
    "was",
    "were",
    "been",
    "being",
    "have",
    "has",
    "had",
    "will",
    "would",
    "should",
    "could",
    "may",
    "might",
    "must",
    "about",
    "into",
    "onto",
    "over",
    "under",
    "than",
    "then",
    "also",
    "just",
    "only",
    "such",
    "considered",
    "determining",
    "factors",
];

fn norm_entity_token(s: &str) -> String {
    s.trim()
        .to_ascii_uppercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect::<String>()
        .split('_')
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .join("_")
}

fn singularize_norm(n: &str) -> Option<String> {
    if n.len() > 4 && n.ends_with('S') && !n.ends_with("SS") {
        Some(n[..n.len() - 1].to_string())
    } else {
        None
    }
}

/// Content-word tokens from the question (≥3 chars, WH stopwords stripped).
fn question_content_words(query: &str) -> Vec<String> {
    query
        .split(|c: char| !c.is_ascii_alphanumeric() && c != '-' && c != '_')
        .filter(|w| w.len() >= 3)
        .map(|w| w.to_string())
        .filter(|w| !Q_STOP.contains(&w.to_ascii_lowercase().as_str()))
        .collect()
}

/// Space-form content bigrams for body matching (045), e.g. `"bone cancers"`.
///
/// Includes a singularized last-token variant (`"bone cancer"`) when applicable.
pub fn question_content_phrases(query: &str) -> Vec<String> {
    let words = question_content_words(query);
    let mut out: Vec<String> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    let mut push = |p: String| {
        if p.len() < 5 || !seen.insert(p.clone()) {
            return;
        }
        out.push(p);
    };
    for i in 0..words.len().saturating_sub(1) {
        let a = words[i].to_ascii_lowercase();
        let b = words[i + 1].to_ascii_lowercase();
        push(format!("{a} {b}"));
        if b.len() > 4 && b.ends_with('s') && !b.ends_with("ss") {
            push(format!("{a} {}", &b[..b.len() - 1]));
        }
    }
    out
}

fn body_hits_content_phrases(body: &str, phrases: &[String]) -> bool {
    if phrases.is_empty() {
        return false;
    }
    let b = body.to_ascii_lowercase();
    phrases.iter().any(|p| b.contains(p.as_str()))
}

/// Exact-name candidates from the question + multi-token LL keywords.
///
/// Bigrams only from the question (avoids hub unigrams). Low-level keywords
/// contribute only when they contain ≥2 tokens.
pub fn candidate_entity_norms(query: &str, keywords: &ExtractedKeywords) -> Vec<String> {
    let words = question_content_words(query);

    let mut out: Vec<String> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    let mut push = |n: String| {
        if n.len() < 4 || !seen.insert(n.clone()) {
            return;
        }
        out.push(n);
    };

    for i in 0..words.len().saturating_sub(1) {
        let gram = format!("{}_{}", words[i], words[i + 1]);
        let n = norm_entity_token(&gram);
        if let Some(s) = singularize_norm(&n) {
            push(s);
        }
        push(n);
    }

    for kw in &keywords.low_level {
        let parts: Vec<&str> = kw
            .split(|c: char| !c.is_ascii_alphanumeric() && c != '-' && c != '_')
            .filter(|w| !w.is_empty())
            .collect();
        if parts.len() < 2 {
            continue;
        }
        let n = norm_entity_token(&parts.join("_"));
        if let Some(s) = singularize_norm(&n) {
            push(s);
        }
        push(n);
    }

    out
}

/// Admit topic entities into `context` and record chunk ids for VECTOR pin.
pub async fn admit_topic_entities(
    graph: GraphReadView<'_>,
    context: &mut QueryContext,
    query_text: &str,
    keywords: &ExtractedKeywords,
    workspace_id: Option<&str>,
) -> crate::error::Result<usize> {
    if !topic_entity_admit_enabled() || !intent_allows(keywords.query_intent) {
        return Ok(0);
    }

    let candidates = candidate_entity_norms(query_text, keywords);
    if candidates.is_empty() {
        return Ok(0);
    }

    let graph_ids: Vec<String> = candidates
        .iter()
        .map(|n| graph_entity_id_for_workspace(n, workspace_id))
        .filter(|id| !id.is_empty())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

    if graph_ids.is_empty() {
        return Ok(0);
    }

    let nodes = graph.get_nodes_batch(&graph_ids).await?;
    let existing: HashSet<String> = context
        .entities
        .iter()
        .map(|e| bare_entity_norm(&e.name))
        .collect();

    let mut admitted_names: Vec<String> = Vec::new();
    let mut admit_chunk_ids: Vec<String> = Vec::new();
    let mut prepend: Vec<crate::context::RetrievedEntity> = Vec::new();

    for id in &graph_ids {
        let Some(node) = nodes.get(id) else {
            continue;
        };
        let bare_norm = bare_entity_norm(id);
        let entity = build_entity_from_node(id, &node.properties, 0, 1.0);
        if entity.source_chunk_ids.is_empty() {
            continue;
        }
        admit_chunk_ids.extend(entity.source_chunk_ids.iter().cloned());
        if existing.contains(&bare_norm) {
            // Already retrieved — still pin its chunks; bump score for ranking.
            if let Some(e) = context
                .entities
                .iter_mut()
                .find(|e| bare_entity_norm(&e.name) == bare_norm)
            {
                e.score = e.score.max(1.0);
            }
            admitted_names.push(bare_norm);
            continue;
        }
        admitted_names.push(bare_norm);
        prepend.push(entity);
    }

    if prepend.is_empty() && admit_chunk_ids.is_empty() {
        return Ok(0);
    }

    if !prepend.is_empty() {
        let mut rest = std::mem::take(&mut context.entities);
        let mut merged = prepend;
        merged.append(&mut rest);
        context.entities = merged;
    }

    // de-dupe chunk ids preserve order
    let mut seen_c = HashSet::new();
    admit_chunk_ids.retain(|c| seen_c.insert(c.clone()));

    context.metadata.insert(
        META_TOPIC_ADMIT_CHUNK_IDS.to_string(),
        serde_json::json!(admit_chunk_ids),
    );
    context.metadata.insert(
        META_TOPIC_ADMIT_ENTITIES.to_string(),
        serde_json::json!(admitted_names),
    );

    tracing::info!(
        admitted = admitted_names.len(),
        topic_chunks = admit_chunk_ids.len(),
        candidates = candidates.len(),
        "038 topic_entity_admit: Exploratory exact-name entities pinned for Mix"
    );

    Ok(admitted_names.len())
}

/// Prefer topic-admitted chunk ids in a VECTOR result list (stable pin).
pub fn pin_topic_chunks_in_results<T, F>(results: &mut Vec<T>, topic_chunk_ids: &[String], id_of: F)
where
    F: Fn(&T) -> &str,
{
    if topic_chunk_ids.is_empty() || results.is_empty() {
        return;
    }
    let want: HashSet<&str> = topic_chunk_ids.iter().map(|s| s.as_str()).collect();
    let mut pinned = Vec::new();
    let mut rest = Vec::new();
    for r in results.drain(..) {
        if want.contains(id_of(&r)) {
            pinned.push(r);
        } else {
            rest.push(r);
        }
    }
    results.append(&mut pinned);
    results.append(&mut rest);
}

pub fn topic_chunk_ids_from_context(context: &QueryContext) -> Vec<String> {
    context
        .metadata
        .get(META_TOPIC_ADMIT_CHUNK_IDS)
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|x| x.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keywords::ExtractedKeywords;

    #[test]
    fn bigrams_not_unigrams() {
        let kw = ExtractedKeywords::new(vec![], vec![], QueryIntent::Exploratory);
        let c = candidate_entity_norms(
            "How are bone cancers staged and what factors are considered?",
            &kw,
        );
        assert!(c.iter().any(|n| n == "BONE_CANCER"), "{c:?}");
        assert!(
            !c.iter()
                .any(|n| n == "CANCER" || n == "STAGE" || n == "BONE"),
            "hub unigrams must not be candidates: {c:?}"
        );
    }

    #[test]
    fn multi_token_ll_keyword() {
        let kw = ExtractedKeywords::new(
            vec![],
            vec!["bone cancer".into(), "staging".into()],
            QueryIntent::Exploratory,
        );
        let c = candidate_entity_norms("tell me more", &kw);
        assert!(c.iter().any(|n| n == "BONE_CANCER"), "{c:?}");
        assert!(!c.iter().any(|n| n == "STAGING"), "{c:?}");
    }

    #[test]
    fn pin_reorders_topic_first() {
        let mut results = vec!["a", "topic1", "b", "topic2"];
        pin_topic_chunks_in_results(&mut results, &["topic2".into(), "topic1".into()], |s| s);
        assert_eq!(results, vec!["topic1", "topic2", "a", "b"]);
    }

    #[test]
    fn prefer_topic_for_trunc_moves_capped_ids_front() {
        std::env::set_var("EDGEQUAKE_TOPIC_TRUNC_PROTECT", "1");
        let mut chunks = vec![
            crate::context::RetrievedChunk::new("a", "A", 1.0),
            crate::context::RetrievedChunk::new("t1", "T1", 0.9),
            crate::context::RetrievedChunk::new("b", "B", 0.8),
            crate::context::RetrievedChunk::new("t2", "T2", 0.7),
            crate::context::RetrievedChunk::new("t3", "T3", 0.6),
        ];
        prefer_topic_chunks_for_trunc(&mut chunks, &["t2".into(), "t1".into(), "t3".into()], 2);
        assert_eq!(
            chunks.iter().map(|c| c.id.as_str()).collect::<Vec<_>>(),
            vec!["t1", "t2", "a", "b", "t3"]
        );
        std::env::remove_var("EDGEQUAKE_TOPIC_TRUNC_PROTECT");
    }

    #[test]
    fn merge_topic_admit_unions_arm_metadata() {
        let mut local = QueryContext::new();
        local.metadata.insert(
            META_TOPIC_ADMIT_CHUNK_IDS.to_string(),
            serde_json::json!(["c1", "c2"]),
        );
        local.metadata.insert(
            META_TOPIC_ADMIT_ENTITIES.to_string(),
            serde_json::json!(["BONE_CANCER"]),
        );
        let mut global = QueryContext::new();
        global.metadata.insert(
            META_TOPIC_ADMIT_CHUNK_IDS.to_string(),
            serde_json::json!(["c2", "c3"]),
        );
        let mut merged = QueryContext::new();
        merge_topic_admit_metadata(&mut merged, &[&local, &global]);
        let ids = topic_chunk_ids_from_context(&merged);
        assert_eq!(ids, vec!["c1", "c2", "c3"]);
        let ents = merged
            .metadata
            .get(META_TOPIC_ADMIT_ENTITIES)
            .and_then(|v| v.as_array())
            .unwrap();
        assert_eq!(ents[0].as_str(), Some("BONE_CANCER"));
    }

    #[test]
    fn materialize_types_scopes_to_summarize() {
        std::env::remove_var("EDGEQUAKE_TOPIC_MATERIALIZE_TYPES");
        assert!(topic_materialize_types_allow(None));
        assert!(topic_materialize_types_allow(Some("Fact Retrieval")));

        std::env::set_var("EDGEQUAKE_TOPIC_MATERIALIZE_TYPES", "summarize");
        assert!(topic_materialize_types_allow(Some("Contextual Summarize")));
        assert!(!topic_materialize_types_allow(Some("Fact Retrieval")));
        assert!(!topic_materialize_types_allow(None));
        std::env::remove_var("EDGEQUAKE_TOPIC_MATERIALIZE_TYPES");
    }
}
