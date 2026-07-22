//! SOTA Prompt Templates for Entity Extraction
//!
//! This module contains production-quality prompts ported from LightRAG,
//! implementing tuple-based extraction format for robustness.
//!
//! ## Key Features
//!
//! - **Tuple Format**: Uses `<|#|>` delimiter for robust parsing
//! - **Completion Signal**: `<|COMPLETE|>` for reliable extraction detection
//! - **Multi-Language**: Configurable `{language}` parameter
//! - **N-ary Decomposition**: Explicit instructions for complex relationships
//! - **Entity Naming**: Title case with consistent naming rules
//!
//! ## Usage
//!
//! ```rust,ignore
//! use edgequake_pipeline::prompts::{EntityExtractionPrompts, TupleParser};
//!
//! let prompts = EntityExtractionPrompts::default();
//! let system_prompt = prompts.system_prompt(&["PERSON", "ORGANIZATION"], "English");
//! let user_prompt = prompts.user_prompt("Some text...", &["PERSON"], "English");
//!
//! // Parse LLM response
//! let parser = TupleParser::new();
//! let result = parser.parse(&llm_response, "chunk-1")?;
//! ```

mod entity_extraction;
mod entity_type_policy;
mod extract_caps;
mod json_extract;
mod json_prompts;
mod normalizer;
mod parser;
mod section_context;
mod summarization;

pub use entity_extraction::EntityExtractionPrompts;
pub use entity_type_policy::{
    enforce_entity_type, json_entity_types_prompt_section, normalize_type_token,
    sota_entity_type_instruction, EntityExtractionSchema, METADATA_ENTITY_TYPES_STRICT,
};
pub use extract_caps::{
    apply_default_extraction_caps, apply_extraction_caps, ExtractionCaps,
    DEFAULT_MAX_EXTRACTION_ENTITIES, DEFAULT_MAX_EXTRACTION_RECORDS,
};
pub use json_extract::extract_json_from_response;
pub use json_prompts::{json_extraction_prompt, json_gleaning_prompt, JSON_OUTPUT_FORMAT_SECTION};
pub use normalizer::{is_opaque_identifier, normalize_entity_name};
pub use parser::{
    detect_format_markers, ExtractionResultParser, HybridExtractionParser, JsonExtractionParser,
    JsonParseOptions, TupleParser,
};
pub use section_context::{
    format_section_context, text_with_section_context, truncate_section_context,
    DEFAULT_MAX_SECTION_CONTEXT_TOKENS,
};
pub use summarization::SummarizationPrompts;

/// Default tuple delimiter for extraction output.
pub const DEFAULT_TUPLE_DELIMITER: &str = "<|#|>";

/// Completion signal to detect complete extractions.
pub const DEFAULT_COMPLETION_DELIMITER: &str = "<|COMPLETE|>";

/// Supported output languages for extraction.
pub const SUPPORTED_LANGUAGES: &[&str] = &[
    "English",
    "Chinese",
    "Japanese",
    "Korean",
    "Spanish",
    "French",
    "German",
    "Portuguese",
    "Italian",
    "Russian",
];

/// Default entity types for extraction.
///
/// Matches LightRAG `default_entity_types_guidance` / `DEFAULT_ENTITY_TYPES`
/// (Person…NaturalObject) plus `OTHER`. No `DATE` — that type induced
/// duration/measurement noise on Acc medical corpora (053).
///
/// `NATURALOBJECT` is the UPPER fold of LR's `NaturalObject` (no underscore
/// inserted by [`normalize_type_token`]).
pub fn default_entity_types() -> Vec<String> {
    vec![
        "PERSON".to_string(),
        "CREATURE".to_string(),
        "ORGANIZATION".to_string(),
        "LOCATION".to_string(),
        "EVENT".to_string(),
        "CONCEPT".to_string(),
        "METHOD".to_string(),
        "CONTENT".to_string(),
        "DATA".to_string(),
        "ARTIFACT".to_string(),
        "NATURALOBJECT".to_string(),
        "OTHER".to_string(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_entity_types() {
        let types = default_entity_types();
        assert!(types.contains(&"PERSON".to_string()));
        assert!(types.contains(&"ORGANIZATION".to_string()));
        assert!(types.contains(&"NATURALOBJECT".to_string()));
        assert!(types.contains(&"OTHER".to_string()));
        assert!(types.contains(&"METHOD".to_string()));
        assert!(!types.iter().any(|t| t == "DATE"));
        assert!(!types.iter().any(|t| t == "PRODUCT"));
        assert!(!types.iter().any(|t| t == "TECHNOLOGY"));
        assert!(!types.iter().any(|t| t == "DOCUMENT"));
        assert_eq!(types.len(), 12);
    }

    #[test]
    fn test_supported_languages() {
        assert!(SUPPORTED_LANGUAGES.contains(&"English"));
        assert!(SUPPORTED_LANGUAGES.contains(&"Chinese"));
    }
}
