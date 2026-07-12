//! Context string formatting for LLM prompts (SPEC-047 Q1.1).
//!
//! # SOLID
//!
//! - **SRP:** Pure formatting of retrieved items for the prompt — no retrieval I/O.
//! - **OCP:** New header fields extend [`format_chunk_header`] without changing callers.
//! - **DIP:** [`QueryContext::to_context_string`] depends on this module, not the reverse.
//!
//! # DRY
//!
//! Truncation token estimates should use the same chunk header format so budget
//! accounting matches what the LLM actually sees.

use crate::context::{RetrievedChunk, RetrievedEntity, RetrievedRelationship};

/// Build the metadata suffix for a chunk header (`page=N`, `modality=…`).
///
/// Empty when no grounding metadata is present.
pub fn format_chunk_meta(chunk: &RetrievedChunk) -> String {
    let mut parts: Vec<String> = Vec::new();
    if let Some(page) = chunk.page_start {
        parts.push(format!("page={page}"));
    }
    if let Some(modality) = chunk.modality.as_ref() {
        let trimmed = modality.trim();
        if !trimmed.is_empty() {
            parts.push(format!("modality={trimmed}"));
        }
    }
    if parts.is_empty() {
        String::new()
    } else {
        format!(" {}", parts.join(" "))
    }
}

/// One chunk block as injected into the LLM context.
///
/// Example: `[1] (score: 0.850) page=12 modality=chart\n…content…\n\n`
pub fn format_chunk_block(ref_id: usize, chunk: &RetrievedChunk) -> String {
    let meta = format_chunk_meta(chunk);
    format!(
        "[{ref_id}] (score: {:.3}){meta}\n{}\n\n",
        chunk.score, chunk.content
    )
}

/// Format string used when estimating entity tokens (must stay close to prompt).
pub fn format_entity_line(entity: &RetrievedEntity) -> String {
    let degree_info = if entity.degree > 0 {
        format!(" [connections: {}]", entity.degree)
    } else {
        String::new()
    };
    format!(
        "- **{}** ({}){}: {}\n",
        entity.name, entity.entity_type, degree_info, entity.description
    )
}

/// Format string used when estimating relationship tokens.
pub fn format_relationship_line(rel: &RetrievedRelationship) -> String {
    if rel.description.is_empty() {
        format!(
            "- {} --[{}]--> {}\n",
            rel.source, rel.relation_type, rel.target
        )
    } else {
        format!(
            "- {} --[{}]--> {}: {}\n",
            rel.source, rel.relation_type, rel.target, rel.description
        )
    }
}

/// Assemble the full context string (entities → relationships → chunks).
pub fn format_query_context(
    entities: &[RetrievedEntity],
    relationships: &[RetrievedRelationship],
    chunks: &[RetrievedChunk],
) -> String {
    let mut parts = Vec::new();

    if !entities.is_empty() {
        parts.push("### Knowledge Graph Data (Entities)\n\n".to_string());
        for entity in entities {
            parts.push(format_entity_line(entity));
        }
        parts.push("\n".to_string());
    }

    if !relationships.is_empty() {
        parts.push("### Knowledge Graph Data (Relationships)\n\n".to_string());
        for rel in relationships {
            parts.push(format_relationship_line(rel));
        }
        parts.push("\n".to_string());
    }

    if !chunks.is_empty() {
        parts.push("### Document Chunks\n\n".to_string());
        parts.push(
            "Each chunk header may include `page=N` (1-indexed PDF page) and `modality=` \
(chart|figure|table|equation). Prefer evidence from matching pages/modalities when answering.\n\n"
                .to_string(),
        );
        for (i, chunk) in chunks.iter().enumerate() {
            parts.push(format_chunk_block(i + 1, chunk));
        }
    }

    parts.join("")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::{RetrievedChunk, RetrievedEntity};

    #[test]
    fn chunk_meta_includes_page_and_modality() {
        let chunk = RetrievedChunk::new("c1", "Revenue rose 12%", 0.91)
            .with_page(7)
            .with_modality("chart");
        let meta = format_chunk_meta(&chunk);
        assert!(meta.contains("page=7"));
        assert!(meta.contains("modality=chart"));
    }

    #[test]
    fn chunk_block_header_is_groundable() {
        let chunk = RetrievedChunk::new("c1", "Table row A", 0.8)
            .with_page(3)
            .with_modality("table");
        let block = format_chunk_block(1, &chunk);
        assert!(block.starts_with("[1] (score: 0.800) page=3 modality=table\n"));
        assert!(block.contains("Table row A"));
    }

    #[test]
    fn format_query_context_mentions_page_legend() {
        let chunk = RetrievedChunk::new("c1", "body", 0.5).with_page(2);
        let s = format_query_context(
            &[RetrievedEntity::new("Acme", "ORG", "A company")],
            &[],
            &[chunk],
        );
        assert!(s.contains("page=2"));
        assert!(s.contains("page=N"));
        assert!(s.contains("Document Chunks"));
    }
}
