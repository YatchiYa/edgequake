//! PDF document upload handlers.
//!
//! @implements SPEC-007: PDF Upload Support with Vision LLM Integration
//! @implements FEAT0701: PDF document upload endpoint
//! @implements FEAT0702: PDF status checking
//! @implements FEAT0703: PDF listing
//!
//! # Implements
//!
//! - **UC0701**: Upload PDF for processing
//! - **UC0702**: Check PDF processing status
//! - **UC0703**: List workspace PDFs
//!
//! # Enforces
//!
//! - **BR0701**: PDFs scoped to workspace
//! - **BR0702**: 100MB file size limit
//! - **BR0703**: Deduplication via SHA-256
//! - **BR0201**: Tenant isolation
//! - **BR0401**: Authentication required
//!
//! # Endpoints
//!
//! | Method | Path | Handler | Description |
//! |--------|------|---------|-------------|
//! | POST | `/api/v1/documents/pdf` | [`upload_pdf_document`] | Upload PDF file |
//! | GET | `/api/v1/documents/pdf/:id` | [`get_pdf_status`] | Get PDF status |
//! | GET | `/api/v1/documents/pdf` | [`list_pdfs`] | List PDFs |
//! | DELETE | `/api/v1/documents/pdf/:id` | [`delete_pdf`] | Delete PDF |

// Sub-modules organized by responsibility (SRP)
pub mod types;
pub mod upload;
pub mod status;
pub mod content;
mod helpers;
pub mod operations;

// Re-export all public items for backward compatibility
pub use types::*;
pub use upload::*;
pub use status::*;
pub use content::*;
pub use operations::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_estimate_processing_time() {
        // Small PDF, few pages
        let data = vec![0u8; 100_000]; // 100KB
        let time = helpers::estimate_processing_time(&data, Some(5));
        assert!(time >= 15 && time <= 30); // 5 pages * 3s + 0.1MB * 0.5

        // Large PDF, many pages
        let data = vec![0u8; 10_000_000]; // 10MB
        let time = helpers::estimate_processing_time(&data, Some(50));
        assert!(time >= 150 && time <= 200); // 50 pages * 3s + 10MB * 0.5
    }

    #[test]
    fn test_pdf_upload_options_vision_model() {
        let mut opts = PdfUploadOptions::default();
        opts.vision_provider = Some("openai".to_string());
        // OODA-04: Updated from gpt-4o-mini to gpt-4.1-nano per mission directive
        assert_eq!(opts.vision_model(), "gpt-4.1-nano");

        opts.vision_provider = Some("ollama".to_string());
        assert_eq!(opts.vision_model(), "gemma3:latest");

        opts.vision_model = Some("custom-model".to_string());
        assert_eq!(opts.vision_model(), "custom-model");

        // Test default (None provider = "openai" default)
        let default_opts = PdfUploadOptions::default();
        assert_eq!(default_opts.vision_model(), "gpt-4.1-nano");
    }

    /// OODA-17: Test PdfOperationResponse serialization
    #[test]
    fn test_pdf_operation_response_serialization() {
        // With task_id
        let response = PdfOperationResponse {
            success: true,
            pdf_id: "abc123".to_string(),
            message: "Retry initiated".to_string(),
            task_id: Some("task-456".to_string()),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"success\":true"));
        assert!(json.contains("\"pdf_id\":\"abc123\""));
        assert!(json.contains("\"task_id\":\"task-456\""));

        // Without task_id (skip_serializing_if)
        let response = PdfOperationResponse {
            success: true,
            pdf_id: "abc123".to_string(),
            message: "Cancelled".to_string(),
            task_id: None,
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(!json.contains("task_id"));
    }
}
