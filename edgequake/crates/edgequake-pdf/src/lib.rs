pub mod backend;
pub mod chart_crop;
pub mod drawing_tags;
pub mod embedded_images;
pub mod error;
pub mod fallback;
pub mod inline_images;
pub mod page_assets;
pub mod region_assets;
pub mod vision_markdown;
pub mod vision_prompts;

pub use backend::{
    create_pdf_converter, PageDrawingAssetsConfig, PdfConversionConfig, PdfConverter,
    PdfParserBackend, VisionConversionConfig, VisionStatusHook,
};
pub use chart_crop::{
    chart_residual_candidate_pages, crop_png_to_ink_bbox, encode_png,
    filter_chart_pages_by_page_png_ink, ink_content_bbox, maybe_chart_specialize_bytes,
    page_markdown_suggests_chart, page_png_has_ink_residual, text_suggests_chart,
    write_chart_crop_assets, CHART_CROP_RENDER,
};
pub use drawing_tags::{
    asset_id_from_rel_path, asset_url_matches_rel, bind_figure_images_to_page_asset,
    caption_with_page_context, count_markdown_images_for_asset, dedupe_markdown_asset_images,
    finalize_page_asset_images, format_drawing_block, format_drawing_tag,
    format_inline_asset_image, inject_figure_local_images, insert_drawing_tag_after_first_image,
    is_drawing_eligible_asset_rel_path, is_full_page_asset_rel_path,
    markdown_has_durable_asset_image, page_asset_rel_path, page_chart_crop_rel_path,
    page_drawing_item_id, page_figure_asset_rel_path, page_figure_drawing_item_id,
    page_table_asset_rel_path, ASSETS_SUBDIR, EMPTY_VISION_PAGE_PLACEHOLDER,
};
pub use embedded_images::{figures_by_page, write_embedded_figure_assets, WrittenFigureAsset};
pub use error::PdfConversionError;
pub use fallback::{
    build_edgeparse_fallback_message, should_fallback_to_edgeparse, VisionFailureKind,
};
pub use inline_images::{
    scan_inline_image_refs, InlineImageAnalysis, InlineImageAnalyzer, NoopInlineImageAnalyzer,
};
pub use page_assets::{write_page_png_assets, PageAssetRenderConfig};
pub use region_assets::{
    should_write_region_figure, tables_by_page, write_caption_region_assets, WrittenTableAsset,
};
pub use vision_markdown::{
    assemble_vision_markdown, assemble_vision_markdown_with_figures,
    assemble_vision_markdown_with_options, assemble_vision_markdown_with_overrides,
    enrich_markdown_with_viewer_assets, inject_on_disk_region_assets, normalize_vision_pages,
    page_numbers_from_markdown, VisionPageSlice,
};
pub use vision_prompts::RAG_PAGE_VISION_SYSTEM_PROMPT;
