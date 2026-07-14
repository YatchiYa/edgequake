//! Persist embedded PDF ImageXObjects as figure-bounded PNG assets.
//!
//! First principle (SPEC-047): VLM figure/chart/illustration analysis must be
//! bounded to the **image object** in the PDF. Full-page renders are for the
//! markdown viewer only; analyze `<drawing/>` paths prefer these assets.
//!
//! DRY: decoding uses [`edgequake_pdf2md::extract_embedded_images_from_bytes`]
//! (shared Pdfium singleton). This module only names files and writes PNGs.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use image::ImageFormat;
use tracing::{debug, warn};

use crate::drawing_tags::{page_figure_asset_filename, ASSETS_SUBDIR};
use crate::error::PdfConversionError;

/// Written figure asset under `{assets_root}/assets/page-NNNN-fig-MM.png`.
#[derive(Debug, Clone, PartialEq)]
pub struct WrittenFigureAsset {
    pub page_num: usize,
    pub index: usize,
    pub rel_path: String,
    pub width: u32,
    pub height: u32,
    /// PDF-space bbox when known (enables IoU dedup with region crops).
    pub bbox: Option<(f32, f32, f32, f32)>,
}

/// Extract + persist embedded figures as PNG assets.
///
/// Optional `page_filter` is 1-indexed. Empty filter means all pages.
pub async fn write_embedded_figure_assets(
    pdf_bytes: &[u8],
    assets_root: &Path,
    page_filter: Option<&[usize]>,
) -> Result<Vec<WrittenFigureAsset>, PdfConversionError> {
    let bytes = pdf_bytes.to_vec();
    let root = assets_root.to_path_buf();
    let filter = page_filter.map(|p| p.to_vec());
    tokio::task::spawn_blocking(move || {
        write_embedded_figure_assets_blocking(&bytes, &root, filter.as_deref())
    })
    .await
    .map_err(|e| PdfConversionError::Backend(format!("figure write task panicked: {e}")))?
}

fn write_embedded_figure_assets_blocking(
    pdf_bytes: &[u8],
    assets_root: &Path,
    page_filter: Option<&[usize]>,
) -> Result<Vec<WrittenFigureAsset>, PdfConversionError> {
    let extracted = edgequake_pdf2md::extract_embedded_images_from_bytes(pdf_bytes, None)
        .map_err(|e| PdfConversionError::Backend(format!("embedded figure extract: {e}")))?;

    let assets_dir = assets_root.join(ASSETS_SUBDIR);
    std::fs::create_dir_all(&assets_dir).map_err(|e| {
        PdfConversionError::Backend(format!("create assets dir {assets_dir:?}: {e}"))
    })?;

    let mut written = Vec::new();
    for fig in extracted {
        if let Some(pages) = page_filter {
            if !pages.contains(&fig.page_num) {
                continue;
            }
        }
        let filename = page_figure_asset_filename(fig.page_num, fig.index);
        let full_path: PathBuf = assets_dir.join(&filename);
        if let Err(e) = fig.image.save_with_format(&full_path, ImageFormat::Png) {
            warn!(
                page_num = fig.page_num,
                index = fig.index,
                path = %full_path.display(),
                error = %e,
                "Failed to write embedded figure PNG"
            );
            continue;
        }
        let rel_path = format!("{ASSETS_SUBDIR}/{filename}");
        debug!(
            page_num = fig.page_num,
            index = fig.index,
            width = fig.width,
            height = fig.height,
            path = %rel_path,
            "Wrote embedded figure asset"
        );
        written.push(WrittenFigureAsset {
            page_num: fig.page_num,
            index: fig.index,
            rel_path,
            width: fig.width,
            height: fig.height,
            bbox: Some(fig.bbox),
        });
    }
    Ok(written)
}

/// Group written figures by 1-indexed page number.
pub fn figures_by_page(written: &[WrittenFigureAsset]) -> HashMap<usize, Vec<WrittenFigureAsset>> {
    let mut map: HashMap<usize, Vec<WrittenFigureAsset>> = HashMap::new();
    for fig in written {
        map.entry(fig.page_num).or_default().push(fig.clone());
    }
    for list in map.values_mut() {
        list.sort_by_key(|f| f.index);
    }
    map
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    fn sample_pdf() -> Vec<u8> {
        let path =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("test-data/embedded_figure_sample.pdf");
        std::fs::read(&path).unwrap_or_else(|e| panic!("read sample pdf {path:?}: {e}"))
    }

    #[test]
    #[serial]
    fn writes_fig_asset_filename_ssot() {
        let dir = tempfile::tempdir().unwrap();
        let written = match write_embedded_figure_assets_blocking(&sample_pdf(), dir.path(), None) {
            Ok(w) => w,
            Err(e) => {
                eprintln!("pdfium unavailable: {e}");
                return;
            }
        };
        assert!(!written.is_empty());
        assert!(
            written[0].rel_path.contains("-fig-"),
            "analyze assets must use fig path, got {}",
            written[0].rel_path
        );
        assert!(
            written[0].width <= 80 && written[0].height <= 80,
            "must be object-sized, got {}x{}",
            written[0].width,
            written[0].height
        );
        assert!(dir.path().join(&written[0].rel_path).is_file());
    }
}
