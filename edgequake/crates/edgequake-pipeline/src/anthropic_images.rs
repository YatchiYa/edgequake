//! SPEC-083 C-25: Anthropic-compatible image source shaping.
//!
//! `edgequake-llm` (crates.io) Anthropic provider always serializes images as
//! `source.type = "base64"` even when [`ImageData::from_url`] was used. Until
//! that crate is patched/published, callers must either:
//! 1. Emit the correct Anthropic `source` object via [`anthropic_image_source_json`], or
//! 2. Materialize URL / data-URI images to base64 via [`materialize_image_for_anthropic`]
//!    before invoking the Anthropic provider.
//!
//! Official Anthropic Messages API accepts:
//! - `{ "type": "base64", "media_type": "...", "data": "..." }`
//! - `{ "type": "url", "url": "https://..." }`

use edgequake_llm::traits::ImageData;
use serde_json::{json, Value};
use thiserror::Error;

/// Errors while preparing images for Anthropic.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum AnthropicImageError {
    #[error("invalid data URL image")]
    InvalidDataUrl,
    #[error(
        "HTTP(S) image URL cannot be passed through edgequake-llm Anthropic base64 path; \
         fetch and convert to base64, or use anthropic_image_source_json for a custom client: {0}"
    )]
    HttpUrlNeedsMaterialize(String),
}

/// Build the Anthropic Messages API `source` object for an [`ImageData`].
///
/// Branches on [`ImageData::is_url`] (mime_type == `"url"`) like the OpenAI
/// provider's `to_api_url()` path — unlike upstream anthropic.rs which forces base64.
pub fn anthropic_image_source_json(img: &ImageData) -> Value {
    if img.is_url() {
        // data: URLs are not valid Anthropic `url` sources — treat as base64 after parse.
        if img.data.starts_with("data:") {
            if let Ok(materialized) = materialize_image_for_anthropic(img) {
                return anthropic_image_source_json(&materialized);
            }
        }
        json!({
            "type": "url",
            "url": img.data,
        })
    } else {
        json!({
            "type": "base64",
            "media_type": img.mime_type,
            "data": img.data,
        })
    }
}

/// Convert URL-shaped [`ImageData`] into base64 so the broken Anthropic provider
/// path still produces a valid request.
///
/// - Non-URL images are returned unchanged.
/// - `data:image/...;base64,...` URLs are decoded into [`ImageData::new`].
/// - `http(s)://` URLs return [`AnthropicImageError::HttpUrlNeedsMaterialize`]
///   (fetch belongs at the call site / custom HTTP client).
pub fn materialize_image_for_anthropic(img: &ImageData) -> Result<ImageData, AnthropicImageError> {
    if !img.is_url() {
        return Ok(img.clone());
    }

    let raw = img.data.trim();
    if let Some(rest) = raw.strip_prefix("data:") {
        return parse_data_url_image(rest);
    }

    if raw.starts_with("http://") || raw.starts_with("https://") {
        return Err(AnthropicImageError::HttpUrlNeedsMaterialize(
            raw.to_string(),
        ));
    }

    // Unknown URL scheme — still reject rather than send as bogus base64.
    Err(AnthropicImageError::HttpUrlNeedsMaterialize(
        raw.to_string(),
    ))
}

/// Materialize all images in a slice (skipping already-base64 entries).
pub fn materialize_images_for_anthropic(
    images: &[ImageData],
) -> Result<Vec<ImageData>, AnthropicImageError> {
    images.iter().map(materialize_image_for_anthropic).collect()
}

fn parse_data_url_image(rest: &str) -> Result<ImageData, AnthropicImageError> {
    // Format: image/png;base64,<payload>
    let (meta, data) = rest
        .split_once(',')
        .ok_or(AnthropicImageError::InvalidDataUrl)?;
    if !meta.contains("base64") {
        return Err(AnthropicImageError::InvalidDataUrl);
    }
    let mime = meta
        .split(';')
        .next()
        .filter(|m| m.starts_with("image/"))
        .ok_or(AnthropicImageError::InvalidDataUrl)?;
    if data.is_empty() {
        return Err(AnthropicImageError::InvalidDataUrl);
    }
    Ok(ImageData::new(data, mime))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_anthropic_url_image_source() {
        let url_img = ImageData::from_url("https://example.com/photo.jpg");
        let source = anthropic_image_source_json(&url_img);
        assert_eq!(source["type"], "url");
        assert_eq!(source["url"], "https://example.com/photo.jpg");
        assert!(source.get("data").is_none());
        assert!(source.get("media_type").is_none());

        let b64 = ImageData::new("aGVsbG8=", "image/png");
        let source_b64 = anthropic_image_source_json(&b64);
        assert_eq!(source_b64["type"], "base64");
        assert_eq!(source_b64["media_type"], "image/png");
        assert_eq!(source_b64["data"], "aGVsbG8=");
    }

    #[test]
    fn materialize_data_url_to_base64() {
        let img = ImageData::from_url("data:image/png;base64,iVBORw0KGgo=");
        let out = materialize_image_for_anthropic(&img).unwrap();
        assert!(!out.is_url());
        assert_eq!(out.mime_type, "image/png");
        assert_eq!(out.data, "iVBORw0KGgo=");
        let source = anthropic_image_source_json(&out);
        assert_eq!(source["type"], "base64");
    }

    #[test]
    fn http_url_materialize_is_blocked_until_fetched() {
        let img = ImageData::from_url("https://example.com/a.png");
        let err = materialize_image_for_anthropic(&img).unwrap_err();
        assert!(matches!(
            err,
            AnthropicImageError::HttpUrlNeedsMaterialize(_)
        ));
    }
}
