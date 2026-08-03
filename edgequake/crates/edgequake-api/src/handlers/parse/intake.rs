//! Multipart and raw PDF intake for SPEC-094.

use axum::body::Bytes;
use axum::http::{header, HeaderMap};
use axum_extra::extract::Multipart;

use super::errors::ParseErrorCode;
use super::types::ParseOptions;
use crate::error::ApiResult;
use crate::file_validation::sanitize_filename;
use crate::multipart_upload::stream_field_to_tempfile;

/// Parsed upload ready for conversion.
pub struct ParsedIntake {
    pub filename: String,
    pub bytes: Vec<u8>,
    pub options: ParseOptions,
}

/// Parse multipart form: `file` + optional JSON `options`.
pub async fn intake_multipart(mut multipart: Multipart) -> ApiResult<ParsedIntake> {
    let mut streamed = None;
    let mut options = ParseOptions::default();
    let mut saw_file = false;

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        ParseErrorCode::InvalidRequest.into_api_error(format!("Invalid multipart body: {e}"))
    })? {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "file" => {
                let filename = field
                    .file_name()
                    .map(sanitize_filename)
                    .unwrap_or_else(|| "document.pdf".to_string());
                let content_type = field.content_type().map(|m| m.to_string());
                if let Some(ct) = content_type.as_deref() {
                    if !ct.eq_ignore_ascii_case("application/pdf")
                        && !ct.eq_ignore_ascii_case("application/octet-stream")
                        && !filename.to_ascii_lowercase().ends_with(".pdf")
                    {
                        return Err(ParseErrorCode::UnsupportedMediaType.into_api_error(format!(
                            "Unsupported media type '{ct}'; expected application/pdf"
                        )));
                    }
                } else if !filename.to_ascii_lowercase().ends_with(".pdf") {
                    return Err(ParseErrorCode::UnsupportedMediaType
                        .into_api_error("Unsupported file type; expected a PDF"));
                }
                streamed = Some(stream_field_to_tempfile(field, filename).await?);
                saw_file = true;
            }
            "options" => {
                let text = field.text().await.map_err(|e| {
                    ParseErrorCode::InvalidRequest
                        .into_api_error(format!("Failed to read options part: {e}"))
                })?;
                options = serde_json::from_str(&text).map_err(|e| {
                    ParseErrorCode::InvalidRequest
                        .into_api_error(format!("Invalid options JSON: {e}"))
                })?;
            }
            _ => {
                // Ignore unknown parts.
            }
        }
    }

    if !saw_file {
        return Err(
            ParseErrorCode::InvalidRequest.into_api_error("Missing or unreadable file part")
        );
    }

    let streamed = streamed.ok_or_else(|| {
        ParseErrorCode::InvalidRequest.into_api_error("Missing or unreadable file part")
    })?;
    let (filename, bytes) = streamed.into_bytes()?;
    if bytes.is_empty() {
        return Err(ParseErrorCode::InvalidRequest.into_api_error("Empty file upload"));
    }
    validate_pdf_magic(&bytes)?;

    Ok(ParsedIntake {
        filename,
        bytes,
        options,
    })
}

/// Parse raw `application/pdf` body with optional `X-Filename` and query options.
pub fn intake_raw_pdf(
    headers: &HeaderMap,
    body: Bytes,
    options: ParseOptions,
) -> ApiResult<ParsedIntake> {
    let content_type = headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/pdf");
    let mime = content_type
        .split(';')
        .next()
        .unwrap_or(content_type)
        .trim();
    if !mime.eq_ignore_ascii_case("application/pdf")
        && !mime.eq_ignore_ascii_case("application/octet-stream")
    {
        return Err(ParseErrorCode::UnsupportedMediaType.into_api_error(format!(
            "Unsupported media type '{mime}'; expected application/pdf"
        )));
    }

    if body.is_empty() {
        return Err(ParseErrorCode::InvalidRequest.into_api_error("Empty PDF body"));
    }
    validate_pdf_magic(&body)?;

    let filename = headers
        .get("x-filename")
        .and_then(|v| v.to_str().ok())
        .map(sanitize_filename)
        .unwrap_or_else(|| "document.pdf".to_string());

    Ok(ParsedIntake {
        filename,
        bytes: body.to_vec(),
        options,
    })
}

pub fn prefer_respond_async(headers: &HeaderMap) -> bool {
    headers
        .get("prefer")
        .and_then(|v| v.to_str().ok())
        .map(|v| {
            v.to_ascii_lowercase()
                .split(',')
                .any(|part| part.trim() == "respond-async")
        })
        .unwrap_or(false)
}

pub fn validate_pdf_magic(bytes: &[u8]) -> ApiResult<()> {
    if bytes.len() < 5 || &bytes[0..5] != b"%PDF-" {
        return Err(ParseErrorCode::DocumentUnreadable
            .into_api_error("Encrypted or malformed document (missing PDF header)"));
    }
    Ok(())
}
