//! Re-export pipeline `LlmTextEmbedder` (SPEC-046 DIP adapter).
//!
//! Kept as a thin module so existing `crate::services::LlmTextEmbedder` paths
//! remain stable after the adapter moved into `edgequake-pipeline`.

pub use edgequake_pipeline::LlmTextEmbedder;
