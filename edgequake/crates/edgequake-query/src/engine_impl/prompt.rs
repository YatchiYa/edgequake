use std::collections::HashMap;
use std::sync::Arc;

use futures::stream::StreamExt;

use crate::context::QueryContext;
use crate::conversation_context::{self, DEFAULT_CONVERSATION_TURN_LIMIT};
use crate::error::Result;
use crate::types::ConversationMessage;
use edgequake_llm::traits::{ChatMessage, ImageData};

use super::QueryEngine;
use super::TokenStream;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AnswerPromptStyle {
    Default,
    LightRag,
    /// 046 — name concrete Context entities over category paraphrases.
    Specific,
}

impl QueryEngine {
    /// Check if metadata matches tenant/workspace filter.
    ///
    /// DEPRECATED (SPEC-007): Prefer `query_filtered()` which pushes filtering to SQL.
    /// Retained for backward-compat with custom VectorStorage impls that don't override
    /// `query_filtered()`.
    #[allow(dead_code)]
    pub(super) fn matches_tenant_filter(
        &self,
        metadata: &serde_json::Value,
        tenant_id: &Option<String>,
        workspace_id: &Option<String>,
    ) -> bool {
        edgequake_storage::MetadataFilter::matches_tenant_workspace_value(
            metadata,
            tenant_id,
            workspace_id,
        )
    }

    /// Check if properties match tenant filter.
    ///
    /// DEPRECATED (SPEC-007): Prefer `query_filtered()` which pushes filtering to SQL.
    #[allow(dead_code)]
    pub(super) fn matches_tenant_filter_props(
        &self,
        properties: &HashMap<String, serde_json::Value>,
        tenant_id: &Option<String>,
        workspace_id: &Option<String>,
    ) -> bool {
        edgequake_storage::MetadataFilter::matches_tenant_workspace_properties(
            properties,
            tenant_id,
            workspace_id,
        )
    }

    // ── Private helpers ──────────────────────────────────────────────────────

    /// Build the shared context section (context text + optional extra instructions).
    ///
    /// WHY (DRY): Both `build_prompt` (text-only path) and
    /// `build_vision_system_message` (chat/vision path) need the same context
    /// block.  Centralising it here avoids duplication and ensures a single
    /// point of change.
    fn format_context_section(
        context: &QueryContext,
        system_prompt_extension: Option<&str>,
    ) -> (String, String) {
        let context_text = context.to_context_string();
        // SPEC-004: optional additional instructions injected by callers
        let additional_instructions = match system_prompt_extension {
            Some(ext) if !ext.trim().is_empty() => {
                format!("\n\n---Additional Instructions---\n\n{}\n", ext.trim())
            }
            _ => String::new(),
        };
        (context_text, additional_instructions)
    }

    // ── Public(super) prompt builders ────────────────────────────────────────

    /// Build an all-in-one text prompt for `provider.complete()` (text-only path).
    ///
    /// WHY: The prompt is designed to maximise information extraction from available
    /// context.  When comparing products where one term doesn't exist in the knowledge
    /// base, we still want to provide useful information about what IS available,
    /// rather than just saying "no information found."
    ///
    /// `system_prompt_extension`: Optional additional instructions injected between
    /// the base instructions and the context section (SPEC-004).
    ///
    /// `question_type`: Optional GraphRAG-Bench / product type label (047). Used when
    /// `EDGEQUAKE_ANSWER_SPECIFIC_TYPES` scopes `ANSWER_PROMPT=specific`.
    pub(super) fn build_prompt(
        &self,
        query: &str,
        context: &QueryContext,
        system_prompt_extension: Option<&str>,
        conversation_history: &[ConversationMessage],
        question_type: Option<&str>,
    ) -> String {
        if context.is_empty() {
            return "I'm sorry, but I couldn't find any relevant information in my knowledge base to answer your question.".to_string();
        }

        let (context_text, additional_instructions) =
            Self::format_context_section(context, system_prompt_extension);
        let conversation_section = conversation_context::format_conversation_history(
            conversation_history,
            DEFAULT_CONVERSATION_TURN_LIMIT,
        )
        .map(|section| format!("\n{section}\n"))
        .unwrap_or_default();

        match Self::answer_prompt_style(question_type) {
            AnswerPromptStyle::LightRag => {
                return Self::build_prompt_lightrag(
                    query,
                    &context_text,
                    &additional_instructions,
                    &conversation_section,
                );
            }
            AnswerPromptStyle::Specific => {
                return Self::build_prompt_specific(
                    query,
                    &context_text,
                    &additional_instructions,
                    &conversation_section,
                );
            }
            AnswerPromptStyle::Default => {}
        }

        let grounding = crate::grounding::grounding_instructions();

        format!(
            r#"---Role---

You are an expert AI assistant specializing in synthesizing information from a provided knowledge base. Your primary function is to answer user queries accurately by ONLY using the information within the provided **Context**.

---Goal---

Generate a comprehensive, well-structured answer to the user query.
The answer must integrate relevant facts from the Knowledge Graph and Document Chunks found in the **Context**.

---Instructions---

1. Step-by-Step Reasoning:
  - Carefully determine the user's query intent to fully understand the information need.
  - Scrutinize the **Entities**, **Relations**, and **Chunks** sections in the **Context**. Identify and extract all pieces of information that are directly relevant to answering the user query.
  - Weave the extracted facts into a coherent and logical response. Your own knowledge must ONLY be used to formulate fluent sentences and connect ideas, NOT to introduce any external information.

2. Content & Grounding:
  - Strictly adhere to the provided context; DO NOT invent facts from general knowledge or assume missing numbers.
  - Grounded arithmetic is allowed when BOTH operands (e.g. percentage and sample size N) are explicit in Context — compute the count (not the bare percentage) and cite both sources (see Citations & Page Grounding).
  - If the answer cannot be fully determined from the **Context**, state what information IS available and note what is missing. A partial answer with specific data is better than a generic "insufficient information" response.

{grounding}

3. Formatting & Language:
  - The response MUST be in the same language as the user query.
  - Use Markdown formatting for clarity (headings, bold text, bullet points).
{additional_instructions}
---Context---

{context_text}
{conversation_section}
---User Query---

{query}"#
        )
    }

    /// `EDGEQUAKE_ANSWER_PROMPT`: `default` | `lightrag` | `specific` (046/047).
    ///
    /// When style is `specific` and `EDGEQUAKE_ANSWER_SPECIFIC_TYPES` is non-empty
    /// (comma-separated tokens, e.g. `complex`), apply specificity only if
    /// `question_type` lowercase contains a token. Empty types → always specific (046).
    /// Scoped + missing/empty `question_type` → default (protect Fact Acc).
    fn answer_prompt_style(question_type: Option<&str>) -> AnswerPromptStyle {
        let base = match std::env::var("EDGEQUAKE_ANSWER_PROMPT")
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase()
            .as_str()
        {
            "lightrag" | "lr" | "rag_response" => AnswerPromptStyle::LightRag,
            "specific" | "entity_first" | "specificity" => AnswerPromptStyle::Specific,
            _ => AnswerPromptStyle::Default,
        };
        if base == AnswerPromptStyle::Specific
            && !Self::specific_types_allow(question_type)
        {
            return AnswerPromptStyle::Default;
        }
        base
    }

    /// 047: token match against `EDGEQUAKE_ANSWER_SPECIFIC_TYPES`.
    fn specific_types_allow(question_type: Option<&str>) -> bool {
        let raw = std::env::var("EDGEQUAKE_ANSWER_SPECIFIC_TYPES").unwrap_or_default();
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

    /// 028 A3: `EDGEQUAKE_ANSWER_PROMPT=lightrag` → closer to LR `rag_response`.
    #[allow(dead_code)]
    fn answer_prompt_style_lightrag() -> bool {
        matches!(Self::answer_prompt_style(None), AnswerPromptStyle::LightRag)
    }

    /// 046: prefer concrete Context names over category paraphrases (Complex Acc).
    ///
    /// Keeps EQ grounded-arithmetic / partial-answer rules (unlike LR abstain).
    fn build_prompt_specific(
        query: &str,
        context_text: &str,
        additional_instructions: &str,
        conversation_section: &str,
    ) -> String {
        let grounding = crate::grounding::grounding_instructions();
        format!(
            r#"---Role---

You are an expert AI assistant specializing in synthesizing information from a provided knowledge base. Your primary function is to answer user queries accurately by ONLY using the information within the provided **Context**.

---Goal---

Generate a comprehensive, well-structured answer to the user query.
The answer must integrate relevant facts from the Knowledge Graph and Document Chunks found in the **Context**.
Prefer **specific named items from Context** (drug names, test names, staging systems, entity labels) over generic category paraphrases.

---Instructions---

1. Step-by-Step Reasoning:
  - Carefully determine the user's query intent to fully understand the information need.
  - Scrutinize the **Entities**, **Relations**, and **Chunks** sections in the **Context**. Identify and extract all pieces of information that are directly relevant to answering the user query.
  - When Context lists concrete members of a class (e.g. named PARP inhibitors, named imaging/exam modalities), **name those members** rather than only the class label.
  - For multi-part questions, address each part explicitly (what / why / when / which factors).
  - Weave the extracted facts into a coherent and logical response. Your own knowledge must ONLY be used to formulate fluent sentences and connect ideas, NOT to introduce any external information.

2. Content & Grounding:
  - Strictly adhere to the provided context; DO NOT invent facts from general knowledge or assume missing numbers.
  - Grounded arithmetic is allowed when BOTH operands (e.g. percentage and sample size N) are explicit in Context — compute the count (not the bare percentage) and cite both sources (see Citations & Page Grounding).
  - If the answer cannot be fully determined from the **Context**, state what information IS available and note what is missing. A partial answer with specific data is better than a generic "insufficient information" response.

{grounding}

3. Formatting & Language:
  - The response MUST be in the same language as the user query.
  - Use Markdown formatting for clarity (headings, bold text, bullet points).
{additional_instructions}
---Context---

{context_text}
{conversation_section}
---User Query---

{query}"#
        )
    }

    /// LightRAG-aligned answer prompt (028 A3 Acc ablation).
    ///
    /// Diff vs EQ default: stricter "do not guess", explicit References section,
    /// Knowledge Graph Data + Document Chunks wording, no grounded-arithmetic block.
    fn build_prompt_lightrag(
        query: &str,
        context_text: &str,
        additional_instructions: &str,
        conversation_section: &str,
    ) -> String {
        format!(
            r#"---Role---

You are an expert AI assistant specializing in synthesizing information from a provided knowledge base. Your primary function is to answer user queries accurately by ONLY using the information within the provided **Context**.

---Goal---

Generate a comprehensive, well-structured answer to the user query.
The answer must integrate relevant facts from the Knowledge Graph and Document Chunks found in the **Context**.
Consider the conversation history if provided to maintain conversational flow and avoid repeating information.

---Instructions---

1. Step-by-Step Instruction:
  - Carefully determine the user's query intent in the context of the conversation history to fully understand the user's information need.
  - Scrutinize both Knowledge Graph Data (Entities / Relations) and Document Chunks in the **Context**. Identify and extract all pieces of information that are directly relevant to answering the user query.
  - Weave the extracted facts into a coherent and logical response. Your own knowledge must ONLY be used to formulate fluent sentences and connect ideas, NOT to introduce any external information.
  - Track chunk ids that directly support the facts presented. Prefer citing those chunks when available.
  - When useful, end with a short `### References` section listing at most 5 supporting document/chunk titles or ids. Do not add commentary after References.

2. Content & Grounding:
  - Strictly adhere to the provided context from the **Context**; DO NOT invent, assume, or infer any information not explicitly stated.
  - If the answer cannot be found in the **Context**, state that you do not have enough information to answer. Do not attempt to guess.

3. Formatting & Language:
  - The response MUST be in the same language as the user query.
  - The response MUST utilize Markdown formatting for enhanced clarity and structure (e.g., headings, bold text, bullet points).
  - Prefer a Multiple Paragraphs style unless the query clearly asks for a short fact.
{additional_instructions}
---Context---

{context_text}
{conversation_section}
---User Query---

{query}"#
        )
    }

    /// Build the **system message** for a vision-enabled `provider.chat()` call.
    ///
    /// WHY (First Principles): The chat API separates concerns cleanly —
    /// role/instructions/context belong in the *system* message; the user's
    /// actual query (+ images) belong in the *user* message.  Putting the role
    /// text ("ONLY use the knowledge graph") inside the *user* message (as the
    /// previous code did) caused the LLM to refuse image queries because the
    /// role text explicitly said to ignore non-textual input.
    ///
    /// This method returns only the system half.  The caller is responsible for
    /// constructing `ChatMessage::user_with_images(query, images)`.
    pub(super) fn build_vision_system_message(
        &self,
        context: &QueryContext,
        system_prompt_extension: Option<&str>,
    ) -> String {
        let (context_text, additional_instructions) =
            Self::format_context_section(context, system_prompt_extension);
        let grounding = crate::grounding::grounding_instructions();

        format!(
            r#"---Role---

You are an expert AI assistant that can analyse images and synthesise information from a provided knowledge base. Your primary function is to answer user queries by using:
1. The visual content of any attached images.
2. The information within the provided **Context** (knowledge graph entities, relationships, and document chunks).

---Goal---

Generate a comprehensive, well-structured answer that integrates observations from the attached images with relevant facts from the Knowledge Graph and Document Chunks.

---Instructions---

1. Visual Analysis:
  - Examine every attached image carefully before answering.
  - Describe, identify, or interpret visual content as requested by the user.
  - Cross-reference visual observations with knowledge graph entities when relevant.

2. Step-by-Step Reasoning:
  - Carefully determine the user's query intent.
  - Extract facts from both the images and the **Context** that are relevant to the query.
  - Weave observations and facts into a coherent, logical response.

3. Content & Grounding:
  - Prefer explicit visual evidence from images and stated facts from the context.
  - Grounded arithmetic is allowed when BOTH operands (e.g. percentage and sample size N) are explicit in Context — compute the count (not the bare percentage) and cite both sources (see Citations & Page Grounding).
  - If the answer cannot be fully determined, state what IS available and note what is missing.

{grounding}

4. Formatting & Language:
  - The response MUST be in the same language as the user query.
  - Use Markdown formatting for clarity (headings, bold text, bullet points).
{additional_instructions}
---Context---

{context_text}"#
        )
    }

    /// Generate answer using LLM.
    ///
    /// If `llm_override` is provided, uses that provider instead of the default.
    /// This enables per-request provider selection (SPEC-032).
    ///
    /// If `images` is Some and non-empty, uses `provider.chat()` with image
    /// attachments instead of `provider.complete()` (FEAT0203: vision queries).
    pub(super) async fn generate_answer_with_provider(
        &self,
        query: &str,
        context: &QueryContext,
        llm_override: Option<&Arc<dyn crate::LLMProvider>>,
        system_prompt_extension: Option<&str>,
        images: Option<&[ImageData]>,
        conversation_history: &[ConversationMessage],
        question_type: Option<&str>,
    ) -> Result<(String, usize)> {
        if context.is_empty() {
            return Ok((
                "I'm sorry, but I couldn't find any relevant information in my knowledge base to answer your question.".to_string(),
                0,
            ));
        }

        let provider = llm_override.unwrap_or(&self.llm_provider);

        // FEAT0203: Two distinct call paths based on whether images are attached.
        //
        // WHY (First Principles): chat() separates system instructions from the user
        // turn.  Putting role text ("ONLY use text context") into the *user* message
        // alongside images caused the LLM to refuse image queries.  The fix is:
        //   • system message  → role + instructions + RAG context (no images, no query)
        //   • user message    → raw query + images
        // This gives the LLM the full context AND the visual content in the correct
        // roles, so it can use both freely.
        //
        // Text-only path keeps using provider.complete() to avoid an unnecessary
        // chat-API round-trip for providers that support both.
        let response = if let Some(imgs) = images.filter(|i| !i.is_empty()) {
            let system_text = self.build_vision_system_message(context, system_prompt_extension);
            let user_text = conversation_context::query_with_conversation_context(
                query,
                conversation_history,
                DEFAULT_CONVERSATION_TURN_LIMIT,
            );
            let messages = vec![
                ChatMessage::system(&system_text),
                ChatMessage::user_with_images(&user_text, imgs.to_vec()),
            ];
            match provider.chat(&messages, None).await {
                Ok(r) => r,
                Err(e) => {
                    tracing::warn!(error = %e, "Vision chat failed; retrying as text-only query");
                    provider
                        .complete(&self.build_prompt(
                            query,
                            context,
                            system_prompt_extension,
                            conversation_history,
                            question_type,
                        ))
                        .await?
                }
            }
        } else {
            provider
                .complete(&self.build_prompt(
                    query,
                    context,
                    system_prompt_extension,
                    conversation_history,
                    question_type,
                ))
                .await?
        };

        Ok((response.content, response.completion_tokens))
    }

    /// Generate answer using the default LLM.
    pub(super) async fn generate_answer(
        &self,
        query: &str,
        context: &QueryContext,
        system_prompt_extension: Option<&str>,
        conversation_history: &[ConversationMessage],
        question_type: Option<&str>,
    ) -> Result<(String, usize)> {
        self.generate_answer_with_provider(
            query,
            context,
            None,
            system_prompt_extension,
            None,
            conversation_history,
            question_type,
        )
        .await
    }

    /// Generate a *direct* LLM answer with no retrieval context (P-G8 / RC-13).
    ///
    /// WHY (First Principles): Bypass mode means "skip retrieval, ask the LLM
    /// directly" — the opposite of RAG. The RAG `generate_answer_with_provider`
    /// guards on `context.is_empty()` and returns the *apology* string for a
    /// real retrieval miss, which is correct for Local/Global/Hybrid/Naive but
    /// wrong for Bypass, where an empty context is *intentional*. This method
    /// bypasses that guard and calls the LLM with a direct prompt, mirroring
    /// `sota_bridge::query_bypass` so both entry paths (HTTP `/query` and the
    /// orchestrator) produce identical Bypass answers (DRY).
    ///
    /// E23: an empty/whitespace query still reaches the LLM; the provider is
    /// responsible for its own handling. `system_prompt_extension` is honored
    /// as a system message when present.
    pub(super) async fn generate_bypass_answer(
        &self,
        query: &str,
        llm_override: Option<&Arc<dyn crate::LLMProvider>>,
        system_prompt_extension: Option<&str>,
        images: Option<&[ImageData]>,
    ) -> Result<(String, usize)> {
        let provider = llm_override.unwrap_or(&self.llm_provider);
        let user_prompt = format!(
            "Answer the following question to the best of your ability.\n\nQuestion: {}\n\nAnswer:",
            query
        );

        let response = if let Some(imgs) = images.filter(|i| !i.is_empty()) {
            // Vision-capable bypass: system = optional extension, user = query + images.
            let system_text = system_prompt_extension
                .unwrap_or("You are a helpful assistant. Answer the user's question directly.");
            let messages = vec![
                ChatMessage::system(system_text),
                ChatMessage::user_with_images(&user_prompt, imgs.to_vec()),
            ];
            match provider.chat(&messages, None).await {
                Ok(r) => r,
                Err(e) => {
                    tracing::warn!(error = %e, "Bypass vision chat failed; retrying as text-only");
                    provider.complete(&user_prompt).await?
                }
            }
        } else if let Some(ext) = system_prompt_extension.filter(|s| !s.trim().is_empty()) {
            let messages = vec![ChatMessage::system(ext), ChatMessage::user(&user_prompt)];
            provider.chat(&messages, None).await?
        } else {
            provider.complete(&user_prompt).await?
        };

        Ok((response.content, response.completion_tokens))
    }

    /// Stream a vision (image-attached) answer (P-G11 / RC-16).
    ///
    /// WHY (First Principles): the `LLMProvider::stream` trait method takes only
    /// a text prompt — it cannot carry images. The vision-capable path is
    /// `provider.chat()` with image attachments (FEAT0203). So streaming vision
    /// parity means: when images are attached, run the vision `chat` call and
    /// emit its result as a one-shot token stream. This keeps the streaming
    /// entry's contract (a `Stream<Item = Result<String>>`) while using the
    /// vision path — the same trade-off the sync path already makes.
    ///
    /// E30: if the vision chat fails (e.g., vision LLM unavailable), fall back
    /// to the text-only `stream`/`complete` path — identical to
    /// `generate_answer_with_provider`'s image fallback.
    pub(super) async fn stream_vision_answer(
        &self,
        query: &str,
        context: &QueryContext,
        llm_override: Option<Arc<dyn crate::LLMProvider>>,
        system_prompt_extension: Option<&str>,
        images: &[ImageData],
    ) -> Result<TokenStream> {
        let provider = llm_override.unwrap_or_else(|| self.llm_provider.clone());
        let system_text = self.build_vision_system_message(context, system_prompt_extension);
        let messages = vec![
            ChatMessage::system(&system_text),
            ChatMessage::user_with_images(query, images.to_vec()),
        ];

        match provider.chat(&messages, None).await {
            Ok(r) => Ok(futures::stream::once(async move { Ok(r.content) }).boxed()),
            Err(e) => {
                tracing::warn!(
                    error = %e,
                    "Streaming vision chat failed; falling back to text-only stream"
                );
                // Text-only fallback: prefer streaming if supported, else one-shot.
                let prompt =
                    self.build_prompt(query, context, system_prompt_extension, &[], None);
                if provider.supports_streaming() {
                    provider
                        .stream(&prompt)
                        .await
                        .map(|s| {
                            s.map(|res| res.map_err(crate::error::QueryError::from))
                                .boxed()
                        })
                        .map_err(crate::error::QueryError::from)
                } else {
                    let resp = provider
                        .complete(&prompt)
                        .await
                        .map_err(crate::error::QueryError::from)?;
                    Ok(futures::stream::once(async move { Ok(resp.content) }).boxed())
                }
            }
        }
    }
}
