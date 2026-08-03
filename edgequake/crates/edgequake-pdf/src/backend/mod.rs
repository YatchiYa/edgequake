mod edgeparse;
mod vision;

use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::PdfConversionError;

pub use edgeparse::EdgeParsePdfConverter;
pub use vision::VisionPdfConverter;

/// Runtime-selectable PDF parser backend.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PdfParserBackend {
    #[default]
    Vision,
    EdgeParse,
}

impl PdfParserBackend {
    pub fn from_env_str(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "vision" | "llm" => Some(Self::Vision),
            "edgeparse" | "edge-parse" | "edge_parse" => Some(Self::EdgeParse),
            _ => None,
        }
    }

    pub fn from_env() -> Option<Self> {
        std::env::var("EDGEQUAKE_PDF_PARSER_BACKEND")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .and_then(|value| Self::from_env_str(&value))
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Vision => "vision",
            Self::EdgeParse => "edgeparse",
        }
    }
}

/// Optional status reporter for post-OCR work inside vision convert (PNG render, etc.).
///
/// Arguments: `(stage_message, stage_progress_0_to_1)`.
pub type VisionStatusHook = Arc<dyn Fn(&str, f64) + Send + Sync>;

/// Per-task vision conversion options preserved from the existing processor.
#[derive(Clone, Default)]
pub struct VisionConversionConfig {
    /// LLM provider id (e.g. `"ollama"`, `"openai"`). Passed to pdf2md factory.
    pub provider_name: Option<String>,
    pub model: Option<String>,
    pub concurrency: Option<usize>,
    pub dpi: Option<u32>,
    pub checkpoint_dir: Option<String>,
    pub no_resume: bool,
    pub progress_callback: Option<Arc<dyn edgequake_pdf2md::ConversionProgressCallback>>,
    /// Fired between OCR complete and markdown return (viewer PNG / chart crops).
    pub status_hook: Option<VisionStatusHook>,
    /// Optional page selection forwarded to pdf2md (SPEC-094).
    pub pages: Option<edgequake_pdf2md::PageSelection>,
}

impl std::fmt::Debug for VisionConversionConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VisionConversionConfig")
            .field("provider_name", &self.provider_name)
            .field("model", &self.model)
            .field("concurrency", &self.concurrency)
            .field("dpi", &self.dpi)
            .field("checkpoint_dir", &self.checkpoint_dir)
            .field("no_resume", &self.no_resume)
            .field(
                "progress_callback",
                &self.progress_callback.as_ref().map(|_| "<callback>"),
            )
            .field(
                "status_hook",
                &self.status_hook.as_ref().map(|_| "<status_hook>"),
            )
            .field("pages", &self.pages)
            .finish()
    }
}

/// When set, vision conversion writes page PNG assets under `assets_root` and
/// injects viewer `![…](assets/…)` links (SPEC-047 Phase C MV-21 / MV-28).
///
/// First principle: page PNGs serve the markdown viewer. Multimodal VLM
/// analyze (`<drawing/>` tags) is optional via [`Self::emit_analyze_tags`].
#[derive(Clone)]
pub struct PageDrawingAssetsConfig {
    /// Root passed to multimodal `resolve_image_asset` as `base_dir`.
    pub assets_root: PathBuf,
    /// Optional stable prefix for drawing ids (typically document id).
    pub id_prefix: Option<String>,
    /// When true, also emit `<drawing/>` tags for multimodal analyze scan (`i`).
    pub emit_analyze_tags: bool,
    /// When set, run the SPEC-049 two-pass VLM figure filter after all crops
    /// are written.  The provider should be the same vision LLM used for page
    /// OCR.  Results are written to `figure_filter_manifest.json` under
    /// `assets_root`.
    pub figure_filter_provider: Option<Arc<dyn edgequake_llm::LLMProvider>>,
}

impl std::fmt::Debug for PageDrawingAssetsConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PageDrawingAssetsConfig")
            .field("assets_root", &self.assets_root)
            .field("id_prefix", &self.id_prefix)
            .field("emit_analyze_tags", &self.emit_analyze_tags)
            .field(
                "figure_filter_provider",
                &self.figure_filter_provider.as_ref().map(|p| p.name()),
            )
            .finish()
    }
}

/// Configuration shared by PDF conversion backends.
#[derive(Clone, Debug, Default)]
pub struct PdfConversionConfig {
    pub page_count_hint: Option<usize>,
    pub table_method: Option<String>,
    pub filename: Option<String>,
    pub vision: Option<VisionConversionConfig>,
    pub page_drawing_assets: Option<PageDrawingAssetsConfig>,
    /// Optional page selection (vision backends; EdgeParse ignores for now).
    pub pages: Option<edgequake_pdf2md::PageSelection>,
}

#[async_trait]
pub trait PdfConverter: Send + Sync {
    async fn convert(
        &self,
        pdf_bytes: &[u8],
        config: &PdfConversionConfig,
    ) -> Result<String, PdfConversionError>;

    fn backend_name(&self) -> &'static str;
}

pub fn create_pdf_converter(backend: PdfParserBackend) -> Arc<dyn PdfConverter> {
    match backend {
        PdfParserBackend::Vision => Arc::new(VisionPdfConverter::new()),
        PdfParserBackend::EdgeParse => Arc::new(EdgeParsePdfConverter),
    }
}

#[cfg(test)]
mod tests {
    use super::PdfParserBackend;

    #[test]
    fn backend_env_aliases_roundtrip() {
        assert_eq!(
            PdfParserBackend::from_env_str("vision"),
            Some(PdfParserBackend::Vision)
        );
        assert_eq!(
            PdfParserBackend::from_env_str("edge-parse"),
            Some(PdfParserBackend::EdgeParse)
        );
        assert_eq!(PdfParserBackend::Vision.as_str(), "vision");
        assert_eq!(PdfParserBackend::EdgeParse.as_str(), "edgeparse");
    }
}
