//! Tuple-delimited extraction result parser (SOTA format).
//!
//! Parses extraction output in the format used by LightRAG:
//! ```text
//! entity<|#|>Name<|#|>TYPE<|#|>Description
//! relation<|#|>Source<|#|>Target<|#|>keywords<|#|>Description
//! <|COMPLETE|>
//! ```
//!
//! # WHY Tuple Format Over JSON
//!
//! The tuple-delimited format is significantly more robust for LLM outputs:
//!
//! 1. **Partial output recovery**: Valid lines parse independently even if
//!    the response is truncated.
//! 2. **No escaping issues**: No quotes, backslashes, or unicode escaping.
//! 3. **Line-by-line processing**: Enables streaming extraction.
//! 4. **Battle-tested**: Proven in the LightRAG paper with millions of extractions.

use super::super::extract_caps::{apply_extraction_caps, ExtractionCaps};
use super::super::normalizer::normalize_entity_name;
use super::super::{DEFAULT_COMPLETION_DELIMITER, DEFAULT_TUPLE_DELIMITER};
use crate::error::Result;
use crate::extractor::{ExtractedEntity, ExtractedRelationship, ExtractionResult};

/// Parser for tuple-delimited extraction results (SOTA format).
///
/// Parses extraction output in the format:
/// ```text
/// entity<|#|>Name<|#|>TYPE<|#|>Description
/// relation<|#|>Source<|#|>Target<|#|>keywords<|#|>Description
/// <|COMPLETE|>
/// ```
#[derive(Debug, Clone)]
pub struct TupleParser {
    tuple_delimiter: String,
    completion_delimiter: String,
    /// SPEC-117: resolved caps (`None` → fleet env at parse time).
    extraction_caps: Option<ExtractionCaps>,
}

impl Default for TupleParser {
    fn default() -> Self {
        Self::new()
    }
}

impl TupleParser {
    /// Create a new tuple parser with default delimiters.
    pub fn new() -> Self {
        Self {
            tuple_delimiter: DEFAULT_TUPLE_DELIMITER.to_string(),
            completion_delimiter: DEFAULT_COMPLETION_DELIMITER.to_string(),
            extraction_caps: None,
        }
    }

    /// Create a parser with custom delimiters.
    pub fn with_delimiters(tuple: &str, completion: &str) -> Self {
        Self {
            tuple_delimiter: tuple.to_string(),
            completion_delimiter: completion.to_string(),
            extraction_caps: None,
        }
    }

    /// Set resolved extract caps (SPEC-117).
    pub fn with_extraction_caps(mut self, caps: ExtractionCaps) -> Self {
        self.extraction_caps = Some(caps);
        self
    }

    /// Parse a response into an extraction result.
    pub fn parse(&self, response: &str, chunk_id: &str) -> Result<ExtractionResult> {
        let mut result = ExtractionResult::new(chunk_id);
        let mut parse_errors = 0u64;

        // Check if the response is complete
        let is_complete = response.contains(&self.completion_delimiter);
        result
            .metadata
            .insert("is_complete".to_string(), serde_json::json!(is_complete));

        for line in response.lines() {
            let line = line.trim();
            if line.is_empty() || line == self.completion_delimiter {
                continue;
            }

            let parts: Vec<&str> = line.split(&self.tuple_delimiter).collect();

            if parts.is_empty() {
                continue;
            }

            match parts[0].trim().to_lowercase().as_str() {
                "entity" if parts.len() >= 4 => {
                    let raw_name = parts[1].trim();
                    let entity_type = parts[2].trim().to_uppercase();
                    let description = parts[3].trim();

                    // Skip entities with empty or whitespace-only names
                    if raw_name.is_empty() {
                        parse_errors += 1;
                        continue;
                    }

                    let normalized_name = normalize_entity_name(raw_name);

                    // BR0006 / 067: Skip entities that normalize to empty (numeric or opaque ID)
                    if normalized_name.is_empty() {
                        if edgequake_storage::is_opaque_identifier(raw_name) {
                            tracing::debug!(
                                raw_name = %raw_name,
                                metric = "opaque_entity_name_rejected",
                                "Skipping opaque identifier entity name (067)"
                            );
                        } else {
                            tracing::debug!(
                                raw_name = %raw_name,
                                "Skipping entity with empty normalized name"
                            );
                        }
                        continue;
                    }

                    result.add_entity(ExtractedEntity::new(
                        normalized_name,
                        entity_type,
                        description,
                    ));
                }
                "relation" | "relationship" if parts.len() >= 5 => {
                    let source = parts[1].trim();
                    let target = parts[2].trim();
                    let keywords_str = parts[3].trim();
                    let description = parts[4].trim();

                    let normalized_source = normalize_entity_name(source);
                    let normalized_target = normalize_entity_name(target);

                    // BR0006: Self-referencing relationships forbidden
                    if normalized_source == normalized_target {
                        tracing::debug!(
                            source = %normalized_source,
                            "Skipping self-referencing relationship (BR0006)"
                        );
                        continue;
                    }

                    // Skip relationships with empty normalized endpoints
                    if normalized_source.is_empty() || normalized_target.is_empty() {
                        tracing::debug!(
                            raw_source = %source,
                            raw_target = %target,
                            "Skipping relationship with empty normalized endpoint"
                        );
                        continue;
                    }

                    // BR0004: Parse and limit keywords (max 5 per edge)
                    let keywords: Vec<String> = keywords_str
                        .split(',')
                        .map(|k| k.trim().to_string())
                        .filter(|k| !k.is_empty())
                        .take(5)
                        .collect();

                    let mut rel = ExtractedRelationship::new(
                        normalized_source,
                        normalized_target,
                        keywords_str,
                    )
                    .with_description(description);

                    if !keywords.is_empty() {
                        rel = rel.with_keywords(keywords);
                    }

                    result.add_relationship(rel);
                }
                _ => {
                    // Unknown line type, count as parse error
                    if line.contains(&self.tuple_delimiter) {
                        parse_errors += 1;
                    }
                }
            }
        }

        result
            .metadata
            .insert("parser".to_string(), serde_json::json!("tuple"));
        result
            .metadata
            .insert("parse_errors".to_string(), serde_json::json!(parse_errors));

        // 054 / SPEC-117: LightRAG per-response quantity caps.
        let caps = self
            .extraction_caps
            .unwrap_or_else(ExtractionCaps::from_env);
        apply_extraction_caps(&mut result, caps);

        Ok(result)
    }

    /// Check if the response appears complete.
    pub fn is_complete(&self, response: &str) -> bool {
        response.contains(&self.completion_delimiter)
    }
}
