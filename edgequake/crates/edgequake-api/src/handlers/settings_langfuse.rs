//! SPEC-124: Langfuse observability status (env-only; no secrets in response).

use std::sync::Mutex;
use std::time::Duration;

use axum::Json;
use edgequake_observability::{
    langfuse_otlp_headers_from_env, otel_feature_built, resolved_langfuse_api, LangfuseApi,
    LangfuseConfig, ObservabilityConfig,
};
use serde::{Deserialize, Serialize};
use tracing::warn;
use utoipa::ToSchema;

use crate::error::ApiResult;

/// In-process cache of the project id returned by Langfuse (non-secret).
/// Keyed by configured `LANGFUSE_BASE_URL` so a region/host change does not
/// reuse a stale id. Failures are not cached. `LANGFUSE_PROJECT_ID` bypasses this.
static RESOLVED_PROJECT_ID: Mutex<Option<CachedProjectId>> = Mutex::new(None);

#[derive(Clone, Debug)]
struct CachedProjectId {
    base_url: String,
    id: String,
}

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
    /// Non-secret base URL (`LANGFUSE_BASE_URL` / `LANGFUSE_HOST`).
    pub base_url: String,
    /// Same as base_url — Langfuse Cloud region / self-hosted host.
    pub ui_url: String,
    /// Project id for `/project/{id}/…` UI routes. None when export is off or unresolved.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,
    /// `{configured_base}/project/{id}` when export is on and project id is known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_ui_url: Option<String>,
    pub public_key_configured: bool,
    pub secret_key_configured: bool,
    /// Whether this binary was built with `--features otel`.
    pub otel_feature_built: bool,
    /// Export is actually possible (enabled && otel feature).
    pub export_active: bool,
    /// Requested transport: `auto` | `otlp` | `ingestion` (`EDGEQUAKE_LANGFUSE_API`).
    pub api: String,
    /// Transport wired at process init (`otlp` | `ingestion`). Omitted until observability starts.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_resolved: Option<String>,
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
            project_id: None,
            project_ui_url: None,
            public_key_configured: cfg.public_key_configured,
            secret_key_configured: cfg.secret_key_configured,
            otel_feature_built: built,
            export_active: cfg.enabled && built,
            api: LangfuseApi::from_env().to_string(),
            api_resolved: resolved_langfuse_api().map(|a| a.to_string()),
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
    let mut dto = LangfuseSettingsResponse::from(&obs.langfuse);
    if dto.export_active {
        if let Some(project_id) = resolve_langfuse_project_id(&obs.langfuse).await {
            dto.project_ui_url = Some(obs.langfuse.project_ui_url(&project_id));
            dto.project_id = Some(project_id);
        }
    }
    Ok(Json(dto))
}

async fn resolve_langfuse_project_id(cfg: &LangfuseConfig) -> Option<String> {
    if let Some(id) = LangfuseConfig::project_id_from_env() {
        return Some(id);
    }
    let base = cfg.base_url.trim_end_matches('/').to_string();
    if let Some(id) = cached_project_id(&base) {
        return Some(id);
    }
    let fetched = fetch_project_id_from_langfuse(cfg).await?;
    store_cached_project_id(base, fetched.clone());
    Some(fetched)
}

fn cached_project_id(base_url: &str) -> Option<String> {
    let guard = RESOLVED_PROJECT_ID
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    guard
        .as_ref()
        .filter(|c| c.base_url == base_url)
        .map(|c| c.id.clone())
}

fn store_cached_project_id(base_url: String, id: String) {
    let mut guard = RESOLVED_PROJECT_ID
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    *guard = Some(CachedProjectId { base_url, id });
}

#[cfg(test)]
fn clear_project_id_cache() {
    let mut guard = RESOLVED_PROJECT_ID
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    *guard = None;
}

#[derive(Debug, Deserialize)]
struct LangfuseProjectsResponse {
    data: Vec<LangfuseProjectRow>,
}

#[derive(Debug, Deserialize)]
struct LangfuseProjectRow {
    id: String,
}

/// GET `{LANGFUSE_BASE_URL}/api/public/projects` — uses the configured host, not a hardcoded region.
async fn fetch_project_id_from_langfuse(cfg: &LangfuseConfig) -> Option<String> {
    let headers = langfuse_otlp_headers_from_env()?;
    let url = format!("{}/api/public/projects", cfg.base_url.trim_end_matches('/'));
    let mut req = reqwest::Client::new()
        .get(&url)
        .timeout(Duration::from_secs(3));
    for (k, v) in headers {
        req = req.header(k, v);
    }
    let resp = match req.send().await {
        Ok(r) => r,
        Err(e) => {
            warn!(error = %e, url = %url, "Langfuse project lookup failed");
            return None;
        }
    };
    if !resp.status().is_success() {
        warn!(status = %resp.status(), url = %url, "Langfuse project lookup HTTP error");
        return None;
    }
    let body: LangfuseProjectsResponse = match resp.json().await {
        Ok(b) => b,
        Err(e) => {
            warn!(error = %e, "Langfuse project lookup JSON parse failed");
            return None;
        }
    };
    body.data
        .into_iter()
        .map(|p| p.id)
        .find(|id| !id.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn response_never_embeds_secret_values() {
        let cfg = LangfuseConfig {
            enabled: true,
            base_url: "https://us.cloud.langfuse.com".into(),
            public_key_configured: true,
            secret_key_configured: true,
            ui_url: "https://us.cloud.langfuse.com".into(),
        };
        let dto = LangfuseSettingsResponse::from(&cfg);
        let json = serde_json::to_string(&dto).unwrap();
        // Placeholder appears; real secret material must not.
        assert!(dto.env_snippet.contains("sk-lf-..."));
        assert!(!json.contains("sk-lf-secret-real"));
        assert!(dto.env_snippet.contains("pk-lf-..."));
        assert_eq!(dto.ui_url, cfg.base_url);
        assert_eq!(dto.base_url, "https://us.cloud.langfuse.com");
        assert!(dto.project_id.is_none());
        assert!(
            matches!(dto.api.as_str(), "auto" | "otlp" | "ingestion"),
            "api={}",
            dto.api
        );
        assert!(dto.api_resolved.is_none());
        assert!(!json.contains("sk-lf-secret-real"));
    }

    #[test]
    fn session_href_uses_configured_base() {
        let cfg = LangfuseConfig {
            enabled: true,
            base_url: "https://us.cloud.langfuse.com".into(),
            public_key_configured: true,
            secret_key_configured: true,
            ui_url: "https://us.cloud.langfuse.com".into(),
        };
        assert_eq!(
            cfg.session_ui_url("clkproj", "a059b323-9c6b-40ba-8044-66ed80b69653"),
            "https://us.cloud.langfuse.com/project/clkproj/sessions/a059b323-9c6b-40ba-8044-66ed80b69653"
        );
    }

    fn with_env_vars(pairs: &[(&str, Option<&str>)], f: impl FnOnce()) {
        let previous: Vec<(String, Option<String>)> = pairs
            .iter()
            .map(|(k, _)| ((*k).to_string(), std::env::var(k).ok()))
            .collect();
        for (k, v) in pairs {
            match v {
                Some(val) => unsafe { std::env::set_var(k, val) },
                None => unsafe { std::env::remove_var(k) },
            }
        }
        f();
        for (k, prev) in previous {
            match prev {
                Some(val) => unsafe { std::env::set_var(&k, val) },
                None => unsafe { std::env::remove_var(&k) },
            }
        }
    }

    fn mock_cfg(base_url: &str) -> LangfuseConfig {
        LangfuseConfig {
            enabled: true,
            base_url: base_url.trim_end_matches('/').to_string(),
            public_key_configured: true,
            secret_key_configured: true,
            ui_url: base_url.trim_end_matches('/').to_string(),
        }
    }

    struct EnvRestore {
        pk: Option<String>,
        sk: Option<String>,
        pid: Option<String>,
    }

    impl EnvRestore {
        fn pin_test_keys() -> Self {
            let prev = Self {
                pk: std::env::var("LANGFUSE_PUBLIC_KEY").ok(),
                sk: std::env::var("LANGFUSE_SECRET_KEY").ok(),
                pid: std::env::var("LANGFUSE_PROJECT_ID").ok(),
            };
            unsafe {
                std::env::set_var("LANGFUSE_PUBLIC_KEY", "pk-lf-test");
                std::env::set_var("LANGFUSE_SECRET_KEY", "sk-lf-test");
                std::env::remove_var("LANGFUSE_PROJECT_ID");
            }
            prev
        }
    }

    impl Drop for EnvRestore {
        fn drop(&mut self) {
            unsafe {
                match self.pk.take() {
                    Some(v) => std::env::set_var("LANGFUSE_PUBLIC_KEY", v),
                    None => std::env::remove_var("LANGFUSE_PUBLIC_KEY"),
                }
                match self.sk.take() {
                    Some(v) => std::env::set_var("LANGFUSE_SECRET_KEY", v),
                    None => std::env::remove_var("LANGFUSE_SECRET_KEY"),
                }
                match self.pid.take() {
                    Some(v) => std::env::set_var("LANGFUSE_PROJECT_ID", v),
                    None => std::env::remove_var("LANGFUSE_PROJECT_ID"),
                }
            }
        }
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn fetches_project_id_from_configured_host() {
        clear_project_id_cache();
        let _env = EnvRestore::pin_test_keys();
        let server = wiremock::MockServer::start().await;
        wiremock::Mock::given(wiremock::matchers::method("GET"))
            .and(wiremock::matchers::path("/api/public/projects"))
            .respond_with(
                wiremock::ResponseTemplate::new(200).set_body_json(serde_json::json!({
                    "data": [{ "id": "clkproj-us", "name": "EdgeQuake" }]
                })),
            )
            .expect(1)
            .mount(&server)
            .await;

        let cfg = mock_cfg(&server.uri());
        let id = fetch_project_id_from_langfuse(&cfg).await;
        assert_eq!(id.as_deref(), Some("clkproj-us"));
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn caches_project_id_for_configured_host() {
        clear_project_id_cache();
        let _env = EnvRestore::pin_test_keys();
        let server = wiremock::MockServer::start().await;
        wiremock::Mock::given(wiremock::matchers::method("GET"))
            .and(wiremock::matchers::path("/api/public/projects"))
            .respond_with(
                wiremock::ResponseTemplate::new(200).set_body_json(serde_json::json!({
                    "data": [{ "id": "cached-proj" }]
                })),
            )
            .expect(1)
            .mount(&server)
            .await;

        let cfg = mock_cfg(&server.uri());
        let first = resolve_langfuse_project_id(&cfg).await;
        let second = resolve_langfuse_project_id(&cfg).await;
        assert_eq!(first.as_deref(), Some("cached-proj"));
        assert_eq!(second, first);
    }

    #[test]
    #[serial_test::serial]
    fn env_project_id_skips_api_cache() {
        clear_project_id_cache();
        with_env_vars(&[("LANGFUSE_PROJECT_ID", Some("from-env"))], || {
            assert_eq!(
                LangfuseConfig::project_id_from_env().as_deref(),
                Some("from-env")
            );
        });
    }
}
