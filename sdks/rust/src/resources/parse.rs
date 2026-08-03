//! SPEC-094: Stateless PDF → Markdown parse resource.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::client::EdgeQuakeClient;
use crate::error::Result;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ParseOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backend: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dpi: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub concurrency: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pages: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub table_method: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emit_assets: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_fallback: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_page_timings: Option<bool>,
    #[serde(rename = "async", skip_serializing_if = "Option::is_none")]
    pub force_async: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseMetrics {
    pub total_ms: u64,
    #[serde(default)]
    pub render_ms: Option<u64>,
    #[serde(default)]
    pub ocr_ms: Option<u64>,
    #[serde(default)]
    pub assemble_ms: Option<u64>,
    #[serde(default)]
    pub pages_per_second: Option<f64>,
    #[serde(default)]
    pub prompt_tokens: Option<u64>,
    #[serde(default)]
    pub completion_tokens: Option<u64>,
    #[serde(default)]
    pub estimated_cost_usd: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseResponse {
    pub markdown: String,
    pub backend: String,
    pub backend_effective: String,
    pub fallback_applied: bool,
    pub page_count: u32,
    pub metrics: ParseMetrics,
    #[serde(default)]
    pub page_timings: Option<serde_json::Value>,
    #[serde(default)]
    pub warnings: Vec<String>,
    pub request_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseAsyncAccepted {
    pub job_id: String,
    pub status: String,
    pub request_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseJobStatusResponse {
    pub job_id: String,
    pub status: String,
    #[serde(default)]
    pub result: Option<ParseResponse>,
    #[serde(default)]
    pub error: Option<serde_json::Value>,
    pub request_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseBackendsResponse {
    pub backends: Vec<serde_json::Value>,
    pub limits: serde_json::Value,
    pub default_backend: String,
}

/// Either a sync parse result or a 202 async acceptance.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ParseOutcome {
    Completed(ParseResponse),
    Accepted(ParseAsyncAccepted),
}

pub struct ParseResource<'a> {
    pub(crate) client: &'a EdgeQuakeClient,
}

impl<'a> ParseResource<'a> {
    /// POST /api/v1/parse
    pub async fn parse(
        &self,
        file_bytes: Vec<u8>,
        filename: &str,
        options: ParseOptions,
    ) -> Result<ParseOutcome> {
        let mut fields = HashMap::new();
        let options_json = serde_json::to_string(&options).unwrap_or_else(|_| "{}".into());
        fields.insert("options".into(), options_json);
        self.client
            .upload_multipart(
                "/api/v1/parse",
                file_bytes,
                filename,
                "application/pdf",
                fields,
            )
            .await
    }

    /// GET /api/v1/parse/backends
    pub async fn backends(&self) -> Result<ParseBackendsResponse> {
        self.client.get("/api/v1/parse/backends").await
    }

    /// GET /api/v1/parse/jobs/{id}
    pub async fn job(&self, id: &str) -> Result<ParseJobStatusResponse> {
        self.client
            .get(&format!("/api/v1/parse/jobs/{id}"))
            .await
    }
}
