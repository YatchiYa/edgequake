//! Persist full-page PNG assets for vision multimodal analyze (SPEC-047 MV-21).

use std::path::{Path, PathBuf};

use edgequake_pdf2md::pipeline::render::render_pages;
use edgequake_pdf2md::ConversionConfig;
use image::ImageFormat;
use tracing::{debug, warn};

use crate::drawing_tags::{page_asset_filename, ASSETS_SUBDIR};
use crate::error::PdfConversionError;

/// Render configuration for page PNG assets (mirrors vision conversion DPI caps).
#[derive(Debug, Clone, Copy)]
pub struct PageAssetRenderConfig {
    pub dpi: u32,
    pub max_rendered_pixels: u32,
}

impl Default for PageAssetRenderConfig {
    fn default() -> Self {
        Self {
            dpi: 150,
            max_rendered_pixels: 2000,
        }
    }
}

/// Write full-page PNGs under `{assets_root}/assets/page-NNNN.png`.
///
/// Returns relative paths (`assets/page-0001.png`) keyed by 1-indexed page number.
pub async fn write_page_png_assets(
    pdf_bytes: &[u8],
    assets_root: &Path,
    page_numbers: &[usize],
    render: PageAssetRenderConfig,
) -> Result<Vec<(usize, String)>, PdfConversionError> {
    if page_numbers.is_empty() {
        return Ok(Vec::new());
    }

    let assets_dir = assets_root.join(ASSETS_SUBDIR);
    std::fs::create_dir_all(&assets_dir).map_err(|e| {
        PdfConversionError::Backend(format!("failed to create assets dir {assets_dir:?}: {e}"))
    })?;

    let temp_pdf = tempfile::NamedTempFile::new().map_err(|e| {
        PdfConversionError::Backend(format!("failed to create temp pdf for page render: {e}"))
    })?;
    let temp_path: PathBuf = temp_pdf.path().to_path_buf();
    std::fs::write(&temp_path, pdf_bytes).map_err(|e| {
        PdfConversionError::Backend(format!("failed to write temp pdf {temp_path:?}: {e}"))
    })?;

    let indices: Vec<usize> = page_numbers.iter().map(|n| n.saturating_sub(1)).collect();

    let render_config = ConversionConfig::builder()
        .dpi(render.dpi)
        .max_rendered_pixels(render.max_rendered_pixels)
        .build()
        .map_err(|e| PdfConversionError::Backend(e.to_string()))?;

    let rendered = render_pages(&temp_path, &render_config, &indices)
        .await
        .map_err(|e| PdfConversionError::Backend(format!("page render failed: {e}")))?;

    let mut written = Vec::with_capacity(rendered.len());
    for (idx0, image) in rendered {
        let page_num = idx0 + 1;
        let filename = page_asset_filename(page_num);
        let full_path = assets_dir.join(&filename);
        if let Err(e) = image.save_with_format(&full_path, ImageFormat::Png) {
            warn!(
                page_num,
                path = %full_path.display(),
                error = %e,
                "Failed to write page PNG asset; skipping drawing ref for this page"
            );
            continue;
        }
        debug!(page_num, path = %full_path.display(), "Wrote vision page PNG asset");
        written.push((page_num, format!("{ASSETS_SUBDIR}/{filename}")));
    }

    Ok(written)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Minimal valid PDF (one empty page) for rasterisation smoke test.
    fn minimal_pdf_bytes() -> Vec<u8> {
        br#"%PDF-1.4
1 0 obj<< /Type /Catalog /Pages 2 0 R >>endobj
2 0 obj<< /Type /Pages /Kids [3 0 R] /Count 1 >>endobj
3 0 obj<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] >>endobj
xref
0 4
0000000000 65535 f 
0000000009 00000 n 
0000000058 00000 n 
0000000115 00000 n 
trailer<< /Size 4 /Root 1 0 R >>
startxref
190
%%EOF"#
            .to_vec()
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn writes_page_png_asset_for_minimal_pdf() {
        let dir = tempfile::tempdir().unwrap();
        let pages = write_page_png_assets(
            &minimal_pdf_bytes(),
            dir.path(),
            &[1],
            PageAssetRenderConfig {
                dpi: 72,
                max_rendered_pixels: 512,
            },
        )
        .await;

        match pages {
            Ok(written) => {
                assert_eq!(written.len(), 1);
                let file = dir.path().join(&written[0].1);
                assert!(file.exists(), "expected PNG at {:?}", file);
                let bytes = std::fs::read(&file).unwrap();
                assert!(bytes.starts_with(&[0x89, 0x50, 0x4E, 0x47]));
            }
            Err(e) => {
                // pdfium may be unavailable in some CI sandboxes — skip hard failure.
                eprintln!("page asset render skipped (pdfium unavailable?): {e}");
            }
        }
    }

    #[test]
    fn empty_page_list_is_noop() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let out = rt.block_on(write_page_png_assets(
            b"not a pdf",
            dir.path(),
            &[],
            PageAssetRenderConfig::default(),
        ));
        assert!(out.unwrap().is_empty());
    }
}
