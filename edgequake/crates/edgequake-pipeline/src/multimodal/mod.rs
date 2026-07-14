//! Multimodal chunk modality-relation injection (LightRAG `operate.extract_entities` subset).

mod injection;
mod retrieval_modality;

pub use injection::{
    inject_modality_relations, parse_mm_display_name, MmChunkSidecarMeta, MmHeadingBlock,
    MmSidecarBlock, MmSidecarRef,
};
pub use retrieval_modality::{
    map_image_type_to_retrieval_modality, resolve_retrieval_modality_from_content,
    resolve_retrieval_modality_from_mm, stamp_retrieval_modality_on_chunks, MODALITY_CHART,
    MODALITY_EQUATION, MODALITY_FIGURE, MODALITY_TABLE,
};
