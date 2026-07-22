//! Inject multimodal entity nodes + association edges after LLM extraction.

use crate::chunker::TextChunk;
use crate::extractor::{ExtractedEntity, ExtractedRelationship, ExtractionResult};
use serde::{Deserialize, Serialize};

use super::display::{parse_mm_display_name, resolve_mm_entity_display, MmDisplayInput};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MmSidecarRef {
    #[serde(rename = "type")]
    pub ref_type: String,
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MmSidecarBlock {
    #[serde(rename = "type")]
    pub sidecar_type: String,
    pub id: String,
    pub refs: Vec<MmSidecarRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MmHeadingBlock {
    pub level: u32,
    pub heading: String,
    #[serde(default)]
    pub parent_headings: Vec<String>,
}

/// Sidecar metadata persisted by EdgeQuake analyze stage (JSON-compatible with api `MultimodalChunk`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MmChunkSidecarMeta {
    pub item_id: String,
    pub modality: String,
    pub text: String,
    pub sidecar: MmSidecarBlock,
    #[serde(default)]
    pub heading: Option<MmHeadingBlock>,
    #[serde(default)]
    pub llm_cache_list: Vec<String>,
}

fn chunk_matches_mm(chunk_content: &str, mm: &MmChunkSidecarMeta) -> bool {
    super::retrieval_modality::chunk_matches_mm_sidecar(chunk_content, mm)
}

/// Augment extractions with mm entity + association edges (LightRAG operate L3622+).
///
/// SPEC-046 EQ-046-15: also creates a synthetic extraction for mm chunks that
/// never received an LLM extraction slot (orphan guarantee — entity always lands).
///
/// `doc_title`: optional human document title / file stem for display_name prefix (066).
pub fn inject_modality_relations(
    extractions: &mut Vec<ExtractionResult>,
    chunks: &[TextChunk],
    mm_chunks: &[MmChunkSidecarMeta],
    file_path: &str,
    doc_title: Option<&str>,
) {
    if mm_chunks.is_empty() {
        return;
    }

    let mut covered_mm_ids: std::collections::HashSet<String> = std::collections::HashSet::new();

    for extraction in extractions.iter_mut() {
        let Some(chunk) = chunks.iter().find(|c| c.id == extraction.source_chunk_id) else {
            continue;
        };
        let Some(mm) = mm_chunks
            .iter()
            .find(|m| chunk_matches_mm(&chunk.content, m))
        else {
            continue;
        };
        covered_mm_ids.insert(mm.sidecar.id.clone());
        inject_into_extraction(
            extraction,
            mm,
            &chunk.content,
            file_path,
            &chunk.id,
            doc_title,
        );
    }

    // Orphan mm chunks: no extraction row matched — synthesize one so the
    // drawing/table/equation entity is never dropped from the KG.
    for mm in mm_chunks {
        if covered_mm_ids.contains(&mm.sidecar.id) {
            continue;
        }
        if !matches!(
            mm.sidecar.sidecar_type.as_str(),
            "drawing" | "table" | "equation"
        ) {
            continue;
        }
        let Some(chunk) = chunks.iter().find(|c| chunk_matches_mm(&c.content, mm)) else {
            // Chunk text may not be in the pipeline chunk list (e.g. appended
            // after chunking). Still inject with a synthetic chunk id.
            let synth_id = format!("mm-orphan-{}", mm.item_id);
            let mut extraction = ExtractionResult::new(synth_id.clone());
            inject_into_extraction(
                &mut extraction,
                mm,
                &mm.text,
                file_path,
                &synth_id,
                doc_title,
            );
            if !extraction.entities.is_empty() {
                extractions.push(extraction);
            }
            continue;
        };
        let mut extraction = ExtractionResult::new(chunk.id.clone());
        inject_into_extraction(
            &mut extraction,
            mm,
            &chunk.content,
            file_path,
            &chunk.id,
            doc_title,
        );
        if !extraction.entities.is_empty() {
            extractions.push(extraction);
        }
    }
}

/// Compact node description: keep markers + first body chars (full text stays in mm chunks).
fn compact_mm_description(content: &str) -> String {
    const MAX: usize = 500;
    let trimmed = content.trim();
    if trimmed.chars().count() <= MAX {
        return trimmed.to_string();
    }
    let mut out: String = trimmed.chars().take(MAX).collect();
    out.push('…');
    out
}

fn inject_into_extraction(
    extraction: &mut ExtractionResult,
    mm: &MmChunkSidecarMeta,
    content: &str,
    file_path: &str,
    chunk_id: &str,
    doc_title: Option<&str>,
) {
    let sidecar_type = mm.sidecar.sidecar_type.as_str();
    if !matches!(sidecar_type, "drawing" | "table" | "equation") {
        return;
    }
    let entity_name = mm.sidecar.id.clone();

    let heading_label = mm
        .heading
        .as_ref()
        .map(|h| h.heading.trim())
        .filter(|s| !s.is_empty())
        .unwrap_or("");

    let resolved = resolve_mm_entity_display(MmDisplayInput {
        item_id: &entity_name,
        content,
        heading: if heading_label.is_empty() {
            None
        } else {
            Some(heading_label)
        },
        caption: None,
        doc_title: doc_title.or(Some(file_path)),
        sidecar_type,
    });

    if !extraction.entities.iter().any(|e| e.name == entity_name) {
        extraction.entities.push(
            ExtractedEntity::new(
                entity_name.clone(),
                sidecar_type,
                compact_mm_description(content),
            )
            .with_source_chunk_id(chunk_id)
            .with_source_file_path(file_path)
            .with_mm_display(
                resolved.display_name.clone(),
                resolved.page,
                resolved.fig,
                resolved.asset_id_hint.clone(),
                resolved.mm_subtype.clone(),
            ),
        );
    } else if let Some(ent) = extraction
        .entities
        .iter_mut()
        .find(|e| e.name == entity_name)
    {
        // Refresh display metadata on re-inject without changing identity.
        if ent.display_name.is_none() {
            ent.display_name = Some(resolved.display_name.clone());
            ent.page_num = resolved.page;
            ent.figure_index = resolved.fig;
            ent.asset_id = resolved.asset_id_hint.clone();
            ent.mm_subtype = resolved.mm_subtype.clone();
        }
    }

    let location = if heading_label.is_empty() {
        "of document".to_string()
    } else {
        format!("in section {heading_label} of document")
    };
    let display = parse_mm_display_name(content, &resolved.display_name);

    let targets: Vec<String> = extraction
        .entities
        .iter()
        .map(|e| e.name.clone())
        .filter(|n| n != &entity_name)
        .collect();

    for tgt in targets {
        let already = extraction.relationships.iter().any(|r| {
            r.source == entity_name && r.target == tgt && r.relation_type == "associated with"
        });
        if already {
            continue;
        }
        let description =
            format!("{tgt} is associated with {sidecar_type} {display} {location} \"{file_path}\"");
        extraction.relationships.push(
            ExtractedRelationship::new(&entity_name, &tgt, "associated with")
                .with_description(description)
                .with_weight(1.0)
                .with_keywords(vec!["associated with".into(), "contained in".into()])
                .with_source_chunk_id(chunk_id)
                .with_source_file_path(file_path),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunker::TextChunk;

    #[test]
    fn parse_mm_display_name_reads_image_label() {
        let content = "[Image Name]系统架构图\n[Image Type]Photo\n\n模块交互关系";
        assert_eq!(parse_mm_display_name(content, "d1"), "系统架构图");
        assert_eq!(
            parse_mm_display_name("no marker", "fallback-id"),
            "fallback-id"
        );
    }

    #[test]
    fn parse_mm_display_name_reads_chart_and_figure_labels() {
        assert_eq!(
            parse_mm_display_name("[Chart Name]revenue\n[Image Type]Chart\n\nbody", "x"),
            "revenue"
        );
        assert_eq!(
            parse_mm_display_name("[Figure Name]arch\n[Image Type]Flowchart\n\nbody", "x"),
            "arch"
        );
    }

    #[test]
    fn inject_modality_relations_adds_entity_and_edges() {
        let mm = MmChunkSidecarMeta {
            item_id: "d1".into(),
            modality: "drawing".into(),
            text: "[Chart Name]系统架构图\n[Image Type]Chart\n\n模块交互关系".into(),
            sidecar: MmSidecarBlock {
                sidecar_type: "drawing".into(),
                id: "d1".into(),
                refs: vec![MmSidecarRef {
                    ref_type: "drawing".into(),
                    id: "d1".into(),
                }],
            },
            heading: Some(MmHeadingBlock {
                level: 0,
                heading: "章节A".into(),
                parent_headings: vec![],
            }),
            llm_cache_list: vec!["default:analysis:abc123".into()],
        };
        let chunks = vec![TextChunk {
            id: "doc-mm-chunk-0".into(),
            content: mm.text.clone(),
            index: 0,
            start_offset: 0,
            end_offset: 0,
            start_line: 1,
            end_line: 1,
            token_count: 10,
            embedding: None,
            section: None,
            page_start: None,
            page_end: None,
            modality: None,
        }];
        let mut extractions = vec![ExtractionResult {
            entities: vec![ExtractedEntity::new(
                "OTHER_ENTITY",
                "CONCEPT",
                "related concept",
            )],
            relationships: vec![],
            source_chunk_id: "doc-mm-chunk-0".into(),
            metadata: Default::default(),
            input_tokens: 0,
            output_tokens: 0,
            extraction_time_ms: 0,
        }];
        inject_modality_relations(
            &mut extractions,
            &chunks,
            &[mm],
            "demo.pdf",
            Some("Demo Doc"),
        );
        let drawing = extractions[0]
            .entities
            .iter()
            .find(|e| e.name == "d1")
            .expect("drawing entity");
        assert!(
            drawing
                .display_name
                .as_deref()
                .is_some_and(|d| d.contains("系统架构图")),
            "display_name={:?}",
            drawing.display_name
        );
        assert_eq!(extractions[0].relationships.len(), 1);
        assert!(extractions[0].relationships[0]
            .description
            .contains("系统架构图"));
    }

    #[test]
    fn inject_creates_orphan_extraction_when_no_llm_slot() {
        let mm = MmChunkSidecarMeta {
            item_id: "t1".into(),
            modality: "table".into(),
            text: "[Table Name]Perf\n\nrows".into(),
            sidecar: MmSidecarBlock {
                sidecar_type: "table".into(),
                id: "t1".into(),
                refs: vec![],
            },
            heading: None,
            llm_cache_list: vec![],
        };
        let chunks = vec![TextChunk {
            id: "doc-mm-table".into(),
            content: mm.text.clone(),
            index: 0,
            start_offset: 0,
            end_offset: 0,
            start_line: 1,
            end_line: 1,
            token_count: 5,
            embedding: None,
            section: None,
            page_start: None,
            page_end: None,
            modality: None,
        }];
        // Empty extractions — LLM never saw this chunk
        let mut extractions = Vec::new();
        inject_modality_relations(&mut extractions, &chunks, &[mm], "demo.pdf", None);
        assert_eq!(extractions.len(), 1);
        assert!(extractions[0].entities.iter().any(|e| e.name == "t1"));
        assert_eq!(extractions[0].entities[0].entity_type, "table");
        assert!(
            extractions[0].entities[0]
                .display_name
                .as_deref()
                .is_some_and(|d| d.contains("Perf")),
            "display_name={:?}",
            extractions[0].entities[0].display_name
        );
    }
}
