//! SPEC-094: Standalone PDF → Markdown parse API.
//!
//! Exposes the same [`edgequake_pdf::PdfConverter`] the ingestion pipeline uses,
//! without persisting documents, assets, or graph rows.

mod backends;
mod errors;
mod handler;
mod intake;
mod jobs;
mod metrics_hook;
mod options;
mod service;
mod types;

pub use backends::*;
pub use handler::*;
pub use jobs::*;
pub use types::*;

// Re-export for internal callers (handlers + jobs).
#[allow(unused_imports)]
pub(crate) use errors::ParseErrorCode;
#[allow(unused_imports)]
pub(crate) use service::{run_parse, ParseLimits};
