//! LLM-judge faithfulness (SPEC-046 OPS-P3.22).
//!
//! Opt-in via `EDGEQUAKE_FAITHFULNESS_JUDGE=1|true|llm`. When disabled (default),
//! callers keep the heuristic sampler only.
//!
//! Design (SOLID / non-flaky):
//! - Pure parsers for env (`parse_faithfulness_judge_enabled`) — tests never
//!   mutate process env.
//! - Pure response parser (`parse_judge_score`) — unit-testable without LLM.
//! - Async judge takes `&dyn LLMProvider` — mock in CI, Mistral/OpenAI in live.

use edgequake_llm::traits::LLMProvider;

use crate::context::QueryContext;
use crate::eval::faithfulness::significant_tokens;

/// Parse judge enable flag from a raw env string (default off).
pub fn parse_faithfulness_judge_enabled(raw: &str) -> bool {
    matches!(
        raw.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "on" | "llm" | "judge"
    )
}

/// Read `EDGEQUAKE_FAITHFULNESS_JUDGE` (default off).
pub fn faithfulness_judge_enabled_from_env() -> bool {
    parse_faithfulness_judge_enabled(
        &std::env::var("EDGEQUAKE_FAITHFULNESS_JUDGE").unwrap_or_default(),
    )
}

/// Build the judge prompt (deterministic; truncated for cost).
pub fn build_faithfulness_judge_prompt(answer: &str, context: &QueryContext) -> String {
    let mut evidence = String::new();
    for (i, chunk) in context.chunks.iter().take(8).enumerate() {
        let preview = truncate_chars(&chunk.content, 400);
        evidence.push_str(&format!("[chunk {i}] {preview}\n"));
    }
    for entity in context.entities.iter().take(8) {
        evidence.push_str(&format!(
            "[entity] {} — {}\n",
            entity.name,
            truncate_chars(&entity.description, 200)
        ));
    }
    if evidence.trim().is_empty() {
        evidence.push_str("(no retrieved evidence)\n");
    }
    format!(
        "You are a faithfulness judge for RAG answers.\n\
         Score how well the ANSWER is grounded in the EVIDENCE on a scale 0.0–1.0.\n\
         Reply with ONLY a single float in [0,1] (optional brief reason after).\n\n\
         EVIDENCE:\n{evidence}\n\
         ANSWER:\n{}\n\n\
         Score:",
        truncate_chars(answer, 800)
    )
}

fn truncate_chars(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    s.chars().take(max).collect::<String>() + "…"
}

/// Parse a judge model response into a score in `[0,1]`.
///
/// Accepts leading float, optional percent, or JSON `{"score":0.8}`.
/// Returns `None` when no parseable score is found.
pub fn parse_judge_score(raw: &str) -> Option<f32> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    // JSON-ish: "score": 0.85
    if let Some(idx) = trimmed.find("\"score\"") {
        let after = &trimmed[idx + 7..];
        if let Some(num_start) = after.find(|c: char| c.is_ascii_digit() || c == '.') {
            if let Some(score) = parse_leading_float(&after[num_start..]) {
                return Some(score);
            }
        }
    }
    parse_leading_float(trimmed)
}

fn parse_leading_float(s: &str) -> Option<f32> {
    let bytes = s.as_bytes();
    if bytes.first() == Some(&b'-') {
        return None; // negative scores rejected
    }
    let mut end = 0;
    while end < bytes.len()
        && (bytes[end].is_ascii_digit()
            || bytes[end] == b'.'
            || bytes[end] == b'e'
            || bytes[end] == b'E')
    {
        end += 1;
    }
    if end == 0 {
        return None;
    }
    let token = &s[..end];
    let num: f32 = token.parse().ok()?;
    if !num.is_finite() {
        return None;
    }
    // Percent style only for whole numbers in (1, 100] (e.g. "85"), not "1.5".
    let score = if num > 1.0 && num <= 100.0 && !token.contains('.') {
        num / 100.0
    } else {
        num
    };
    Some(score.clamp(0.0, 1.0))
}

/// Ask an LLM to score faithfulness; falls back to `None` on parse/provider error.
pub async fn score_faithfulness_llm(
    llm: &dyn LLMProvider,
    answer: &str,
    context: &QueryContext,
) -> Option<f32> {
    // Nothing to judge when answer has no significant tokens.
    if significant_tokens(answer).is_empty() {
        return Some(1.0);
    }
    let prompt = build_faithfulness_judge_prompt(answer, context);
    match llm.complete(&prompt).await {
        Ok(resp) => {
            let score = parse_judge_score(&resp.content);
            if score.is_none() {
                tracing::warn!(
                    preview = %truncate_chars(&resp.content, 80),
                    "faithfulness judge: unparseable score"
                );
            }
            score
        }
        Err(e) => {
            tracing::warn!(error = %e, "faithfulness judge LLM call failed");
            None
        }
    }
}

/// Maybe run LLM judge when enabled; otherwise `None` (caller keeps heuristic).
pub async fn maybe_score_faithfulness_llm(
    judge_enabled: bool,
    llm: Option<&dyn LLMProvider>,
    answer: &str,
    context: &QueryContext,
) -> Option<f32> {
    if !judge_enabled {
        return None;
    }
    let llm = llm?;
    score_faithfulness_llm(llm, answer, context).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::{QueryContext, RetrievedChunk};
    use edgequake_llm::MockProvider;

    #[test]
    fn parse_judge_enabled_edge_cases() {
        assert!(!parse_faithfulness_judge_enabled(""));
        assert!(!parse_faithfulness_judge_enabled("0"));
        assert!(!parse_faithfulness_judge_enabled("false"));
        assert!(parse_faithfulness_judge_enabled("1"));
        assert!(parse_faithfulness_judge_enabled("TRUE"));
        assert!(parse_faithfulness_judge_enabled("llm"));
        assert!(parse_faithfulness_judge_enabled("judge"));
    }

    #[test]
    fn parse_judge_score_variants() {
        assert_eq!(parse_judge_score("0.85"), Some(0.85));
        assert_eq!(parse_judge_score("0.85 grounded"), Some(0.85));
        assert_eq!(parse_judge_score("85"), Some(0.85));
        assert_eq!(parse_judge_score(r#"{"score": 0.7}"#), Some(0.7));
        assert_eq!(parse_judge_score(""), None);
        assert_eq!(parse_judge_score("no number"), None);
        assert_eq!(parse_judge_score("-0.5"), None);
        assert_eq!(parse_judge_score("1.5"), Some(1.0)); // clamp
    }

    #[test]
    fn prompt_includes_evidence_and_answer() {
        let mut ctx = QueryContext::new();
        ctx.add_chunk(RetrievedChunk::new(
            "c1",
            "Apache AGE is a graph extension",
            1.0,
        ));
        let p = build_faithfulness_judge_prompt("AGE is a graph extension", &ctx);
        assert!(p.contains("Apache AGE"));
        assert!(p.contains("AGE is a graph extension"));
        assert!(p.contains("Score:"));
    }

    #[tokio::test]
    async fn llm_judge_parses_scripted_score() {
        let llm = MockProvider::new();
        llm.add_response("0.91 looks grounded").await;
        let mut ctx = QueryContext::new();
        ctx.add_chunk(RetrievedChunk::new("c1", "Sarah Chen leads research", 1.0));
        let score = score_faithfulness_llm(&llm, "Sarah Chen leads research", &ctx)
            .await
            .expect("score");
        assert!((score - 0.91).abs() < 1e-5);
    }

    #[tokio::test]
    async fn maybe_score_off_returns_none() {
        let llm = MockProvider::new();
        llm.add_response("1.0").await;
        let ctx = QueryContext::new();
        assert!(
            maybe_score_faithfulness_llm(false, Some(&llm), "answer tokens here", &ctx)
                .await
                .is_none()
        );
    }
}
