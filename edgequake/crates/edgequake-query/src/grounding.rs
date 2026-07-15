//! Grounding instructions for RAG answer generation (SPEC-047 Q1.2 / 020 A1).
//!
//! # SOLID
//!
//! - **SRP:** Prompt grounding policy only — no retrieval or truncation.
//! - **DRY:** Shared by text and vision prompt builders via [`grounding_instructions`].
//! - **OCP:** Policy text evolves here; callers stay unchanged.
//!
//! # Policy (020 Q11–Q12)
//!
//! Selective refusal ≠ maximal refusal:
//! - **Answer + cite** when a Document Chunk / KG fact supports the asked claim.
//! - **Refuse** only when no supporting evidence is in context.
//! - Prefer partial grounded answers over "Not answerable".
//! - Never ban honest refusal (019 Q8).

/// Instructions injected into system/text prompts so the LLM uses page/modality
/// headers and citation markers from [`crate::context_format`].
///
/// Calibrated (020 A1): entailment-first — answer when evidence supports;
/// refuse only when it does not.
///
/// W3-arith (026 / 032): when context explicitly states both a percentage and a
/// sample size that determine a count, composing `round(pct/100 * N)` is
/// grounded arithmetic — not external knowledge.
pub fn grounding_instructions() -> &'static str {
    r#"2b. Citations & Page Grounding:
  - Document chunks are labeled `[N] (score: …) page=P modality=…` when available.
  - Prefer facts from chunks whose `page=` matches the question's likely evidence pages.
  - When a Document Chunk or Knowledge Graph fact SUPPORTS the asked claim, answer it and cite the supporting chunk as [N]. Do NOT refuse merely because the wording is imperfect or the answer is partial.
  - Prefer a partial answer that quotes what IS in context (with [N]) over "Not answerable".
  - Refuse with "Not answerable" / insufficient evidence ONLY when no Document Chunk and no Knowledge Graph fact supports the asked claim. Do NOT invent values from general knowledge.
  - When stating a concrete fact (number, name, date), cite the supporting chunk as [N].
  - Grounded arithmetic (W3-arith): if the question asks how many / a headcount / a count of people and the Context explicitly states BOTH (a) a percentage/rate and (b) a sample size N (e.g. "1,503 adults", "n=710") that together determine that count, you MUST compute count = round(percentage/100 × N), answer with that short integer (not the percentage), and cite the chunks that supplied the percentage and N. Worked example: Context has "Not good" = 36% and sample "1,503 adults" → answer 541 (not 36 or 36%). Do NOT invent missing percentages or sample sizes. Do NOT refuse merely because the count is not printed as a literal integer when both operands are present."#
}

/// True if policy text still allows honest "Not answerable" (019 Q8 / 020 reject ban).
pub fn allows_honest_refusal(instructions: &str) -> bool {
    let lower = instructions.to_lowercase();
    (lower.contains("not answerable") || lower.contains("insufficient"))
        && !lower.contains("never say not answerable")
}

/// True if policy is entailment-first (answer when evidence supports) — 020 A1.
pub fn is_entailment_first(instructions: &str) -> bool {
    let lower = instructions.to_lowercase();
    lower.contains("supports the asked claim")
        && lower.contains("prefer a partial answer")
        && lower.contains("only when no")
}

/// True if policy permits grounded %×N composition (026 / 032 W3-arith).
pub fn allows_grounded_arithmetic(instructions: &str) -> bool {
    let lower = instructions.to_lowercase();
    lower.contains("grounded arithmetic")
        && (lower.contains("sample size") || lower.contains("percentage"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grounding_mentions_page_and_refusal() {
        let g = grounding_instructions();
        assert!(g.contains("page="));
        assert!(g.contains("[N]"));
        assert!(allows_honest_refusal(g));
    }

    #[test]
    fn grounding_is_entailment_first_calibrated() {
        let g = grounding_instructions();
        assert!(
            is_entailment_first(g),
            "020 A1 requires answer-when-supported / refuse-only-when-absent; got:\n{g}"
        );
        assert!(allows_honest_refusal(g));
    }

    #[test]
    fn grounding_allows_w3_arith() {
        let g = grounding_instructions();
        assert!(
            allows_grounded_arithmetic(g),
            "032 W3-arith requires grounded %×N composition; got:\n{g}"
        );
        assert!(
            g.contains("541") && g.to_lowercase().contains("must compute"),
            "W3-arith-v2 requires worked example + MUST compute; got:\n{g}"
        );
        assert!(allows_honest_refusal(g));
    }
}
