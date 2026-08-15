//! SPEC-124: Langfuse observability status (env-only; no secrets in response).

use axum::Json;
use edgequake_observability::{otel_feature_built, LangfuseConfig, ObservabilityConfig};
use serde::Serialize;
use utoipa::ToSchema;

use crate::error::ApiResult;

/// One env requirement for Settings UI.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct LangfuseRequirementDto {
    pub name: String,
    pub satisfied: bool,
}

/// Public Langfuse status for Settings / operators.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct LangfuseSettingsResponse {
    /// True when keys are present and not force-disabled (export may still need `otel` feature).
    pub enabled: bool,
    /// Non-secret base URL.
    pub base_url: String,
    /// Same as base_url — target for "Open in Langfuse".
    pub ui_url: String,
    pub public_key_configured: bool,
    pub secret_key_configured: bool,
    /// Whether this binary was built with `--features otel`.
    pub otel_feature_built: bool,
    /// Export is actually possible (enabled && otel feature).
    pub export_active: bool,
    /// Copyable operator snippet (placeholders only).
    pub env_snippet: String,
    pub config_requirements: Vec<LangfuseRequirementDto>,
}

impl From<&LangfuseConfig> for LangfuseSettingsResponse {
    fn from(cfg: &LangfuseConfig) -> Self {
        let built = otel_feature_built();
        Self {
            enabled: cfg.enabled,
            base_url: cfg.base_url.clone(),
            ui_url: cfg.ui_url.clone(),
            public_key_configured: cfg.public_key_configured,
            secret_key_configured: cfg.secret_key_configured,
            otel_feature_built: built,
            export_active: cfg.enabled && built,
            env_snippet: cfg.env_snippet(),
            config_requirements: cfg
                .config_requirements()
                .into_iter()
                .map(|r| LangfuseRequirementDto {
                    name: r.name,
                    satisfied: r.satisfied,
                })
                .collect(),
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/settings/langfuse",
    tag = "Settings",
    responses(
        (status = 200, description = "Langfuse observability status", body = LangfuseSettingsResponse)
    )
)]
pub async fn get_langfuse_settings() -> ApiResult<Json<LangfuseSettingsResponse>> {
    let obs = ObservabilityConfig::from_env();
    Ok(Json(LangfuseSettingsResponse::from(&obs.langfuse)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn response_never_embeds_secret_values() {
        let cfg = LangfuseConfig {
            enabled: true,
            base_url: "https://cloud.langfuse.com".into(),
            public_key_configured: true,
            secret_key_configured: true,
            ui_url: "https://cloud.langfuse.com".into(),
        };
        let dto = LangfuseSettingsResponse::from(&cfg);
        let json = serde_json::to_string(&dto).unwrap();
        // Placeholder appears; real secret material must not.
        assert!(dto.env_snippet.contains("sk-lf-..."));
        assert!(!json.contains("sk-lf-secret-real"));
        assert!(dto.env_snippet.contains("pk-lf-..."));
        assert_eq!(dto.ui_url, cfg.base_url);
    }
}
