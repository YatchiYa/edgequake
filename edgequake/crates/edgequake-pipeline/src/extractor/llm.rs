//! LLM-based entity extractor using structured JSON prompts.

use async_trait::async_trait;

use super::{
    assign_token_usage, extraction_completion_options_with_effort,
    maybe_lift_extract_effort_from_llm_error, ConfigurableEntitySchema, EntityExtractor,
    ExtractionResult,
};
use crate::chunker::TextChunk;
use crate::error::{PipelineError, Result};
use crate::prompts::{JsonExtractionParser, JsonParseOptions};

/// LLM-based entity extractor using structured prompts.
///
/// # WHY: LLM Extraction Strategy
///
/// LLM extraction is the core of knowledge graph construction:
///
/// 1. **Structured Prompt** - Uses a carefully designed prompt that:
///    - Lists valid entity types to constrain LLM output
///    - Requests JSON format for reliable parsing
///    - Asks for descriptions to enrich entity/relationship context
///    - WHY JSON: Tuples are faster but JSON is more reliable for complex relationships
///
/// 2. **Entity Type Constraints** - Pre-defined types (PERSON, ORG, LOCATION, etc.)
///    - WHY: Constraining types improves extraction consistency
///    - WHY custom types: Domain-specific extraction (e.g., PROTEIN for biomedical)
///
/// 3. **Relationship Extraction** - Source → Relationship → Target triples
///    - WHY tuples: Graph databases need explicit source/target
///    - WHY descriptions: Context for semantic search
///
/// 4. **Error-Tolerant Parsing** - Handles malformed LLM output
///    - WHY: LLMs occasionally produce invalid JSON; we extract what we can
pub struct LLMExtractor<L>
where
    L: edgequake_llm::LLMProvider + ?Sized,
{
    llm_provider: std::sync::Arc<L>,
    entity_schema: crate::prompts::EntityExtractionSchema,
    /// Natural-language output language for entity/relationship string values (SPEC-096).
    language: String,
    /// Desired reasoning effort for extract (SPEC-109 / SPEC-113 think-off).
    ///
    /// When `None`, provider-aware flooring applies (`none` for Ollama/LM Studio).
    reasoning_effort: Option<String>,
    /// SPEC-117: resolved per-response caps (`None` → fleet env at use time).
    extraction_caps: Option<crate::prompts::ExtractionCaps>,
}

impl<L> LLMExtractor<L>
where
    L: edgequake_llm::LLMProvider + ?Sized,
{
    /// Create a new LLM extractor.
    pub fn new(llm_provider: std::sync::Arc<L>) -> Self {
        Self {
            llm_provider,
            entity_schema: crate::prompts::EntityExtractionSchema::server_default(),
            language: crate::prompts::DEFAULT_EXTRACTION_LANGUAGE.to_string(),
            reasoning_effort: None,
            extraction_caps: None,
        }
    }
}

impl<L> ConfigurableEntitySchema for LLMExtractor<L>
where
    L: edgequake_llm::LLMProvider + ?Sized,
{
    fn entity_schema_field(&mut self) -> &mut crate::prompts::EntityExtractionSchema {
        &mut self.entity_schema
    }
}

impl<L> LLMExtractor<L>
where
    L: edgequake_llm::LLMProvider + ?Sized,
{
    /// Create with custom entity types (strict enforcement, backward compatible).
    pub fn with_entity_types(self, types: Vec<String>) -> Self {
        ConfigurableEntitySchema::with_entity_types(self, types)
    }

    /// Create with full schema (types + strict/permissive mode).
    pub fn with_entity_schema(self, schema: crate::prompts::EntityExtractionSchema) -> Self {
        ConfigurableEntitySchema::with_entity_schema(self, schema)
    }

    /// Set natural-language output language (SPEC-096).
    pub fn with_language(mut self, language: impl Into<String>) -> Self {
        self.language = language.into();
        self
    }

    /// Set desired extract reasoning effort (e.g. `"none"` to disable Ollama think).
    pub fn with_reasoning_effort(mut self, effort: Option<String>) -> Self {
        self.reasoning_effort = effort
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        self
    }

    /// Set resolved extract caps (SPEC-117).
    pub fn with_extraction_caps(mut self, caps: crate::prompts::ExtractionCaps) -> Self {
        self.extraction_caps = Some(caps);
        self
    }

    fn resolved_caps(&self) -> crate::prompts::ExtractionCaps {
        self.extraction_caps
            .unwrap_or_else(crate::prompts::ExtractionCaps::from_env)
    }

    /// Current extraction language.
    pub fn language(&self) -> &str {
        &self.language
    }
}

impl<L> LLMExtractor<L>
where
    L: edgequake_llm::LLMProvider + ?Sized,
{
    /// Build the extraction prompt.
    #[cfg(test)]
    fn build_prompt(&self, chunk: &TextChunk) -> String {
        let text =
            crate::prompts::text_with_section_context(&chunk.content, chunk.section.as_ref());
        crate::prompts::json_extraction_prompt_with_caps(
            &text,
            &self.entity_schema,
            &crate::prompts::effective_extraction_language(&self.language),
            self.resolved_caps(),
        )
    }

    /// Parse the LLM response via shared [`JsonExtractionParser`] (normalization + recovery).
    fn parse_response(&self, response: &str, chunk_id: &str) -> Result<ExtractionResult> {
        JsonExtractionParser::new().parse_with_options(
            response,
            chunk_id,
            JsonParseOptions {
                entity_schema: Some(&self.entity_schema),
                recover_truncated: true,
                // X-16: fail-closed — silent empty on missing JSON marked docs processed.
                empty_on_missing_json: false,
                extraction_caps: Some(self.resolved_caps()),
            },
        )
    }
}

#[async_trait]
impl<L> EntityExtractor for LLMExtractor<L>
where
    L: edgequake_llm::LLMProvider + Send + Sync + ?Sized,
{
    async fn extract(&self, chunk: &TextChunk) -> Result<ExtractionResult> {
        let text =
            crate::prompts::text_with_section_context(&chunk.content, chunk.section.as_ref());
        let language = crate::prompts::effective_extraction_language(&self.language);
        let system = crate::prompts::json_extraction_system_prompt_with_caps(
            &self.entity_schema,
            &language,
            self.resolved_caps(),
        );
        let user = crate::prompts::json_extraction_user_prompt(&text);
        let messages = vec![
            edgequake_llm::traits::ChatMessage::system(&system),
            edgequake_llm::traits::ChatMessage::user(&user),
        ];

        // WHY provider-aware effort + max_tokens:
        // - Cloud reasoning models exhaust completion_tokens on CoT without a floor.
        // - Ollama thinking models default think:true when effort is unset (SPEC-113 Auto).
        // - Local extract floors to "none" → wire think:false (edgequake-llm ≥0.10.7).
        // - Cloud `none` is coerced off disable; one retry if the endpoint still 400s.
        let mut desired_effort = self.reasoning_effort.clone();
        let mut options = extraction_completion_options_with_effort(
            self.llm_provider.model(),
            16384,
            desired_effort.as_deref(),
            self.llm_provider.name(),
        );

        let response = {
            let mut last_err: Option<edgequake_llm::error::LlmError> = None;
            let mut response = None;
            for _attempt in 0..2 {
                match self.llm_provider.chat(&messages, Some(&options)).await {
                    Ok(resp) => {
                        response = Some(resp);
                        break;
                    }
                    Err(e) => {
                        if let Some(lifted) = maybe_lift_extract_effort_from_llm_error(
                            self.llm_provider.name(),
                            self.llm_provider.model(),
                            options
                                .reasoning_effort
                                .as_deref()
                                .or(desired_effort.as_deref()),
                            &e.to_string(),
                        ) {
                            tracing::warn!(
                                from = desired_effort.as_deref().unwrap_or("none"),
                                to = %lifted,
                                "Extract reasoning-off rejected; lifting effort and retrying"
                            );
                            desired_effort = Some(lifted);
                            options = extraction_completion_options_with_effort(
                                self.llm_provider.model(),
                                16384,
                                desired_effort.as_deref(),
                                self.llm_provider.name(),
                            );
                            last_err = Some(e);
                            continue;
                        }
                        return Err(PipelineError::ExtractionError(format!("LLM error: {}", e)));
                    }
                }
            }
            response.ok_or_else(|| {
                PipelineError::ExtractionError(format!(
                    "LLM error: {}",
                    last_err
                        .map(|e| e.to_string())
                        .unwrap_or_else(|| "unknown".into())
                ))
            })?
        };

        let mut result = match self.parse_response(&response.content, &chunk.id) {
            Ok(r) => r,
            Err(parse_err) => {
                // Instructor-style repair: one corrective turn with validator errors.
                tracing::warn!(
                    chunk_id = %chunk.id,
                    error = %parse_err,
                    "Extract JSON parse failed — attempting repair turn"
                );
                let repair_user = format!(
                    "{user}\n\nYour previous response was not valid extraction JSON.\n\
                     Validator error: {parse_err}\n\n\
                     Return ONLY a JSON object with keys \"entities\" and \"relationships\".\n\
                     No markdown fences, no commentary."
                );
                let repair_messages = vec![
                    edgequake_llm::traits::ChatMessage::system(&system),
                    edgequake_llm::traits::ChatMessage::user(&repair_user),
                ];
                let repair = self
                    .llm_provider
                    .chat(&repair_messages, Some(&options))
                    .await
                    .map_err(|e| {
                        PipelineError::ExtractionError(format!(
                            "LLM repair error: {e}; prior parse: {parse_err}"
                        ))
                    })?;
                self.parse_response(&repair.content, &chunk.id)
                    .map_err(|e| {
                        PipelineError::ExtractionError(format!(
                            "Invalid JSON after repair: {e}; first: {parse_err}"
                        ))
                    })?
            }
        };

        assign_token_usage(
            &mut result,
            response.prompt_tokens,
            response.completion_tokens,
        );

        Ok(result)
    }

    fn name(&self) -> &str {
        "llm"
    }

    fn model_name(&self) -> &str {
        self.llm_provider.model()
    }

    /// @implements SPEC-032/OODA-226: Provider tracking in ProcessingStats
    fn provider_name(&self) -> &str {
        self.llm_provider.name()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_parse_response_recovers_partial_json() {
        let provider = Arc::new(edgequake_llm::MockProvider::default());
        let extractor = LLMExtractor::new(provider);

        let truncated_response = r#"{"entities":[{"name":"ALICE","type":"PERSON","description":"A scientist"},{"name":"BOB","type":"PERSON","description":"A colleague"}],"relationships":["#;

        let result = extractor.parse_response(truncated_response, "chunk_001");
        assert!(
            result.is_ok(),
            "parse_response should recover, got: {:?}",
            result
        );
        let extraction = result.unwrap();
        assert_eq!(extraction.entities.len(), 2);
        assert_eq!(extraction.entities[0].name, "ALICE");
        assert_eq!(extraction.entities[1].name, "BOB");
    }

    #[test]
    fn test_parse_response_errors_on_non_json() {
        // X-16: fail-closed — missing JSON must not silently mark chunks processed.
        let provider = Arc::new(edgequake_llm::MockProvider::default());
        let extractor = LLMExtractor::new(provider);

        let result = extractor.parse_response("this is not json", "chunk_bad");
        assert!(
            result.is_err(),
            "non-JSON must error when empty_on_missing_json=false"
        );
    }

    #[test]
    fn test_parse_response_normalizes_entity_names() {
        let provider = Arc::new(edgequake_llm::MockProvider::default());
        let extractor = LLMExtractor::new(provider);
        let response = r#"{"entities":[{"name":"The Company","type":"ORG","description":"x"}],"relationships":[]}"#;
        let extraction = extractor.parse_response(response, "c1").unwrap();
        assert_eq!(extraction.entities[0].name, "COMPANY");
    }

    #[test]
    fn spec096_llm_extractor_language_builder() {
        use crate::chunker::TextChunk;

        let provider = Arc::new(edgequake_llm::MockProvider::default());
        let extractor = LLMExtractor::new(provider).with_language("Japanese");
        assert_eq!(extractor.language(), "Japanese");
        let chunk = TextChunk::new("c1", "Tokyo is the capital.", 0, 0, 21);
        let prompt = extractor.build_prompt(&chunk);
        assert!(prompt.contains("Japanese"));
        assert!(prompt.contains("Output Language"));
    }
}
