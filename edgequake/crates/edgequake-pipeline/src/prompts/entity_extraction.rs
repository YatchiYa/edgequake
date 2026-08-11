//! SOTA Entity Extraction Prompts ported from LightRAG.
//!
//! Implements tuple-based extraction format with completion signals
//! and comprehensive extraction instructions.

use super::extract_caps::ExtractionCaps;
use super::{DEFAULT_COMPLETION_DELIMITER, DEFAULT_TUPLE_DELIMITER};

/// SOTA Entity Extraction Prompts configuration.
#[derive(Debug, Clone)]
pub struct EntityExtractionPrompts {
    /// Tuple delimiter for parsing.
    pub tuple_delimiter: String,
    /// Completion signal for detection.
    pub completion_delimiter: String,
}

impl Default for EntityExtractionPrompts {
    fn default() -> Self {
        Self {
            tuple_delimiter: DEFAULT_TUPLE_DELIMITER.to_string(),
            completion_delimiter: DEFAULT_COMPLETION_DELIMITER.to_string(),
        }
    }
}

impl EntityExtractionPrompts {
    /// Create with custom delimiters.
    pub fn new(tuple_delimiter: &str, completion_delimiter: &str) -> Self {
        Self {
            tuple_delimiter: tuple_delimiter.to_string(),
            completion_delimiter: completion_delimiter.to_string(),
        }
    }

    /// Build the system prompt for entity extraction (fleet env caps).
    pub fn system_prompt(&self, schema: &super::EntityExtractionSchema, language: &str) -> String {
        self.system_prompt_with_caps(schema, language, ExtractionCaps::from_env())
    }

    /// Build the system prompt with explicit SPEC-117 caps.
    pub fn system_prompt_with_caps(
        &self,
        schema: &super::EntityExtractionSchema,
        language: &str,
        caps: ExtractionCaps,
    ) -> String {
        let entity_types_str = schema.types.join(", ");
        let entity_type_instruction =
            super::sota_entity_type_instruction(schema, &entity_types_str);

        format!(
            r#"---Role---
You are a Knowledge Graph Specialist responsible for extracting entities and relationships from the input text.

---Instructions---
1.  **Entity Extraction & Output:**
    *   **Identification:** Identify clearly defined and meaningful entities in the input text.
    *   **Entity Details:** For each identified entity, extract the following information:
        *   `entity_name`: The **human-readable semantic name** of the entity. If the entity name is case-insensitive, capitalize the first letter of each significant word (title case). Ensure **consistent naming** across the entire extraction process.
        *   **Opaque identifiers (FORBIDDEN as entity_name):** Do **not** use UUIDs, GUIDs, ULIDs, MongoDB ObjectIds, hex hashes, AWS ARNs, or other opaque machine/resource IDs as `entity_name`. Prefer the human referent (e.g. organization, product, project, API resource type). If an opaque ID appears in the text, you may mention it inside `entity_description`, but never as the name.
        *   `entity_type`: {entity_type_instruction}
        *   `entity_description`: Provide a concise yet comprehensive description of the entity's attributes and activities, based *solely* on the information present in the input text.
    *   **Output Format - Entities:** Output a total of 4 fields for each entity, delimited by `{tuple_delimiter}`, on a single line. The first field *must* be the literal string `entity`.
        *   Format: `entity{tuple_delimiter}entity_name{tuple_delimiter}entity_type{tuple_delimiter}entity_description`

2.  **Relationship Extraction & Output:**
    *   **Identification:** Identify direct, clearly stated, and meaningful relationships between previously extracted entities.
    *   **N-ary Relationship Decomposition:** If a single statement describes a relationship involving more than two entities (an N-ary relationship), decompose it into multiple binary (two-entity) relationship pairs for separate description.
        *   **Example:** For "Alice, Bob, and Carol collaborated on Project X," extract binary relationships such as "Alice collaborated with Project X," "Bob collaborated with Project X," and "Carol collaborated with Project X."
    *   **Relationship Details:** For each binary relationship, extract the following fields:
        *   `source_entity`: The name of the source entity. Ensure **consistent naming** with entity extraction.
        *   `target_entity`: The name of the target entity. Ensure **consistent naming** with entity extraction.
        *   `relationship_keywords`: One or more high-level keywords summarizing the overarching nature of the relationship. Multiple keywords separated by comma.
        *   `relationship_description`: A concise explanation of the nature of the relationship between the source and target entities.
    *   **Output Format - Relationships:** Output a total of 5 fields for each relationship, delimited by `{tuple_delimiter}`, on a single line. The first field *must* be the literal string `relation`.
        *   Format: `relation{tuple_delimiter}source_entity{tuple_delimiter}target_entity{tuple_delimiter}relationship_keywords{tuple_delimiter}relationship_description`

3.  **Delimiter Usage Protocol:**
    *   The `{tuple_delimiter}` is a complete, atomic marker and **must not be filled with content**. It serves strictly as a field separator.
    *   **Correct Example:** `entity{tuple_delimiter}Tokyo{tuple_delimiter}location{tuple_delimiter}Tokyo is the capital of Japan.`

4.  **Relationship Direction & Duplication:**
    *   Treat all relationships as **undirected** unless explicitly stated otherwise.
    *   Avoid outputting duplicate relationships.

5.  **Output Order & Prioritization:**
    *   Output all extracted entities first, followed by all extracted relationships.
    *   Within the list of relationships, prioritize those that are **most significant** to the core meaning of the input text.

6.  **Quantity Limits (STRICT):**
    *   Output at most {max_entity_records} entity rows in this response.
    *   Output at most {max_total_records} total rows across entities and relationships.
    *   Prioritize highest-value, relation-bearing entities first (core actors, organizations, methods, and concepts that participate in relationships). Emit them before lower-value or isolated mentions.
    *   Output fewer rows if fewer high-value items are present. Do not try to fill the limit.
    *   Only output relationship rows whose source and target entities are both included in the selected entity rows for this response.
    *   If the limit is reached, stop adding new rows immediately and output `{completion_delimiter}`.

7.  **Context & Objectivity:**
    *   Ensure all entity names and descriptions are written in the **third person**.
    *   Explicitly name the subject or object; **avoid using pronouns** such as `this article`, `our company`, `I`, `you`.

8.  **Language & Proper Nouns:**
    *   The entire output (entity names, keywords, and descriptions) must be written in `{language}`.
    *   Proper nouns should be retained in their original language if translation would cause ambiguity.

9.  **Completion Signal:** Output the literal string `{completion_delimiter}` only after all entities and relationships have been completely extracted.

---Examples---
{examples}"#,
            entity_type_instruction = entity_type_instruction,
            tuple_delimiter = self.tuple_delimiter,
            language = language,
            completion_delimiter = self.completion_delimiter,
            max_entity_records = caps.max_entities,
            max_total_records = caps.max_total_records,
            examples = self.get_examples(language)
        )
    }

    /// Build the user prompt for extraction (fleet env caps).
    pub fn user_prompt(
        &self,
        input_text: &str,
        entity_types: &[impl AsRef<str>],
        language: &str,
    ) -> String {
        self.user_prompt_with_caps(
            input_text,
            entity_types,
            language,
            ExtractionCaps::from_env(),
        )
    }

    /// Build the user prompt with explicit SPEC-117 caps.
    pub fn user_prompt_with_caps(
        &self,
        input_text: &str,
        entity_types: &[impl AsRef<str>],
        language: &str,
        caps: ExtractionCaps,
    ) -> String {
        let entity_types_str = entity_types
            .iter()
            .map(|s| s.as_ref())
            .collect::<Vec<_>>()
            .join(", ");

        format!(
            r#"---Task---
Extract entities and relationships from the input text below.

---Instructions---
1. Strictly adhere to all format requirements for entity and relationship lists.
2. Output *only* the extracted list of entities and relationships. No introductory or concluding remarks.
3. In this response, output at most {max_entity_records} entity rows and at most {max_total_records} total rows. Output fewer if fewer high-value items are present.
4. Output `{completion_delimiter}` as the final line after all extractions (or immediately when the row limit is reached).
5. Ensure the output language is {language}.

---Data to be Processed---
<Entity_types>
[{entity_types}]

<InputText>
```
{input_text}
```

<Output>"#,
            completion_delimiter = self.completion_delimiter,
            language = language,
            entity_types = entity_types_str,
            input_text = input_text,
            max_entity_records = caps.max_entities,
            max_total_records = caps.max_total_records,
        )
    }

    /// Build the continue extraction (gleaning) prompt (fleet env caps).
    pub fn continue_extraction_prompt(&self, language: &str) -> String {
        self.continue_extraction_prompt_with_caps(language, ExtractionCaps::from_env())
    }

    /// Continue prompt with explicit SPEC-117 caps.
    pub fn continue_extraction_prompt_with_caps(
        &self,
        language: &str,
        caps: ExtractionCaps,
    ) -> String {
        format!(
            r#"---Task---
Based on the last extraction task, identify and extract any **missed or incorrectly formatted** entities and relationships from the input text.

---Instructions---
1.  **Strict Adherence to System Format:** Follow all format requirements from the system instructions.
2.  **Focus on Corrections/Additions:**
    *   **Do NOT** re-output entities and relationships that were **correctly and fully** extracted.
    *   If an entity or relationship was **missed**, extract and output it now.
    *   If an entity or relationship was **truncated or malformed**, re-output the *corrected and complete* version.
3.  **Output Format - Entities:** 4 fields per entity, delimited by `{tuple_delimiter}`.
4.  **Output Format - Relationships:** 5 fields per relationship, delimited by `{tuple_delimiter}`.
5.  **Output Content Only:** No introductory or concluding remarks.
6.  **Quantity Limits:** At most {max_entity_records} entity rows and {max_total_records} total rows in this response.
7.  **Completion Signal:** Output `{completion_delimiter}` as the final line.
8.  **Output Language:** Ensure the output language is {language}.

<Output>"#,
            tuple_delimiter = self.tuple_delimiter,
            completion_delimiter = self.completion_delimiter,
            language = language,
            max_entity_records = caps.max_entities,
            max_total_records = caps.max_total_records,
        )
    }

    /// Get the few-shot examples for the prompt.
    ///
    /// SPEC-096: omit English few-shots when language ≠ English so models
    /// follow the target language instead of copying English exemplars.
    fn get_examples(&self, language: &str) -> String {
        if !language.eq_ignore_ascii_case("English") {
            return String::new();
        }
        format!(
            r#"
Example 1:
<Input Text>
while Alex clenched his jaw, the buzz of frustration dull against the backdrop of Taylor's authoritarian certainty. It was this competitive undercurrent that kept him alert, the sense that his and Jordan's shared commitment to discovery was an unspoken rebellion against Cruz's narrowing vision of control and order.

<Output>
entity{td}Alex{td}PERSON{td}Alex is a character who experiences frustration and is observant of the dynamics among other characters.
entity{td}Taylor{td}PERSON{td}Taylor is portrayed with authoritarian certainty and shows a moment of reverence towards a device.
entity{td}Jordan{td}PERSON{td}Jordan shares a commitment to discovery with Alex.
entity{td}Cruz{td}PERSON{td}Cruz is associated with a vision of control and order.
relation{td}Alex{td}Taylor{td}power dynamics, observation{td}Alex observes Taylor's authoritarian behavior.
relation{td}Alex{td}Jordan{td}shared goals, rebellion{td}Alex and Jordan share a commitment to discovery.
relation{td}Jordan{td}Cruz{td}ideological conflict{td}Jordan's discovery commitment rebels against Cruz's control vision.
{cd}

Example 2:
<Input Text>
Stock markets faced a sharp downturn today as tech giants saw significant declines, with the global tech index dropping by 3.4%.

<Output>
entity{td}Global Tech Index{td}CONCEPT{td}The Global Tech Index tracks major technology stocks and dropped 3.4%.
entity{td}Market Selloff{td}EVENT{td}Market selloff refers to the significant decline in stock values.
relation{td}Global Tech Index{td}Market Selloff{td}market performance{td}The tech index decline is part of the broader selloff.
{cd}

Example 3:
<Input Text>
Dr. Sarah Chen, lead researcher at Quantum Dynamics Lab in Boston, published a groundbreaking paper on quantum entanglement in Nature Physics journal. The study was funded by the National Science Foundation.

<Output>
entity{td}Sarah Chen{td}PERSON{td}Dr. Sarah Chen is the lead researcher at Quantum Dynamics Lab who published a paper on quantum entanglement.
entity{td}Quantum Dynamics Lab{td}ORGANIZATION{td}Quantum Dynamics Lab is a research institution located in Boston.
entity{td}Boston{td}LOCATION{td}Boston is a city where Quantum Dynamics Lab is located.
entity{td}Nature Physics{td}ORGANIZATION{td}Nature Physics is a scientific journal that published Sarah Chen's paper.
entity{td}Quantum Entanglement{td}CONCEPT{td}Quantum entanglement is a physics phenomenon studied in Sarah Chen's groundbreaking paper.
entity{td}National Science Foundation{td}ORGANIZATION{td}The National Science Foundation funded Sarah Chen's quantum entanglement research.
relation{td}Sarah Chen{td}Quantum Dynamics Lab{td}employment, research{td}Sarah Chen works as lead researcher at Quantum Dynamics Lab.
relation{td}Sarah Chen{td}Nature Physics{td}publication{td}Sarah Chen published her research in Nature Physics journal.
relation{td}Sarah Chen{td}Quantum Entanglement{td}research{td}Sarah Chen researches quantum entanglement.
relation{td}Quantum Dynamics Lab{td}Boston{td}location{td}Quantum Dynamics Lab is located in Boston.
relation{td}National Science Foundation{td}Sarah Chen{td}funding{td}The National Science Foundation funded Sarah Chen's research.
{cd}
"#,
            td = self.tuple_delimiter,
            cd = self.completion_delimiter
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prompts::EntityExtractionSchema;

    fn schema(types: Vec<&str>) -> EntityExtractionSchema {
        EntityExtractionSchema {
            types: types.into_iter().map(str::to_string).collect(),
            strict: true,
            relation_types: Vec::new(),
            relation_strict: true,
            relation_edges: Vec::new(),
        }
    }

    #[test]
    fn test_system_prompt_generation() {
        let prompts = EntityExtractionPrompts::default();
        let system = prompts.system_prompt(&schema(vec!["PERSON", "ORGANIZATION"]), "English");

        assert!(system.contains("Knowledge Graph Specialist"));
        assert!(system.contains("PERSON, ORGANIZATION"));
        assert!(system.contains("<|#|>"));
        assert!(system.contains("<|COMPLETE|>"));
        // 067 — opaque ID hygiene (intentional LightRAG divergence)
        assert!(system.contains("Opaque identifiers"));
        assert!(system.contains("UUID"));
    }

    #[test]
    fn test_user_prompt_generation() {
        let prompts = EntityExtractionPrompts::default();
        let user = prompts.user_prompt("Test text here", &["PERSON"], "English");

        assert!(user.contains("Test text here"));
        assert!(user.contains("<|COMPLETE|>"));
        assert!(user.contains("PERSON"));
    }

    #[test]
    fn test_continue_extraction_prompt() {
        let prompts = EntityExtractionPrompts::default();
        let continue_prompt = prompts.continue_extraction_prompt("English");

        assert!(continue_prompt.contains("missed or incorrectly formatted"));
        assert!(continue_prompt.contains("<|#|>"));
        assert!(continue_prompt.contains("<|COMPLETE|>"));
    }

    #[test]
    fn test_examples_in_prompt() {
        let prompts = EntityExtractionPrompts::default();
        let system = prompts.system_prompt(&schema(vec!["PERSON"]), "English");

        assert!(system.contains("Example 1:"));
        assert!(system.contains("Example 2:"));
        assert!(system.contains("Example 3:"));
        assert!(system.contains("Alex"));
        assert!(system.contains("Sarah Chen"));
    }

    #[test]
    fn spec096_sota_omits_examples_non_english() {
        let prompts = EntityExtractionPrompts::default();
        let system = prompts.system_prompt(&schema(vec!["PERSON"]), "Chinese");
        assert!(system.contains("written in `Chinese`"));
        assert!(!system.contains("Example 1:"));
        assert!(!system.contains("Sarah Chen"));
        assert!(!system.contains("entity<|#|>Alex"));
    }

    #[test]
    fn spec117_sota_prompts_use_resolved_caps_not_env_defaults() {
        let prompts = EntityExtractionPrompts::default();
        let caps = ExtractionCaps {
            max_entities: 17,
            max_total_records: 41,
        };
        let system = prompts.system_prompt_with_caps(&schema(vec!["PERSON"]), "English", caps);
        let user = prompts.user_prompt_with_caps("body", &["PERSON"], "English", caps);
        let cont = prompts.continue_extraction_prompt_with_caps("English", caps);
        assert!(system.contains("17"), "{system}");
        assert!(system.contains("41"), "{system}");
        assert!(user.contains("17") && user.contains("41"), "{user}");
        assert!(cont.contains("17") && cont.contains("41"), "{cont}");
    }
}
