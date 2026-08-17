//! SPEC-094 parse error codes.

use axum::http::StatusCode;

use crate::error::ApiError;

/// Stable parse.* error codes from SPEC-094.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseErrorCode {
    InvalidRequest,
    UnsupportedMediaType,
    DocumentUnreadable,
    TooLarge,
    BackendUnavailable,
    Timeout,
    JobNotFound,
}

impl ParseErrorCode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::InvalidRequest => "parse.invalid_request",
            Self::UnsupportedMediaType => "parse.unsupported_media_type",
            Self::DocumentUnreadable => "parse.document_unreadable",
            Self::TooLarge => "parse.too_large",
            Self::BackendUnavailable => "parse.backend_unavailable",
            Self::Timeout => "parse.timeout",
            Self::JobNotFound => "parse.job_not_found",
        }
    }

    pub fn status(self) -> StatusCode {
        match self {
            Self::InvalidRequest => StatusCode::BAD_REQUEST,
            Self::UnsupportedMediaType => StatusCode::UNSUPPORTED_MEDIA_TYPE,
            Self::DocumentUnreadable => StatusCode::UNPROCESSABLE_ENTITY,
            Self::TooLarge => StatusCode::PAYLOAD_TOO_LARGE,
            Self::BackendUnavailable => StatusCode::BAD_GATEWAY,
            Self::Timeout => StatusCode::GATEWAY_TIMEOUT,
            Self::JobNotFound => StatusCode::NOT_FOUND,
        }
    }

    pub fn into_api_error(self, message: impl Into<String>) -> ApiError {
        ApiError::Parse {
            code: self.as_str(),
            message: message.into(),
            status: self.status(),
        }
    }
}
