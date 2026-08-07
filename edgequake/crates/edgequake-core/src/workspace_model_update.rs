//! Shared workspace model-config update logic (SPEC-013 / server-default reset).
//!
//! Clearing LLM/embedding overrides uses empty string (or `"none"`) — same contract as
//! vision LLM fields in [`UpdateWorkspaceRequest`].
//!
//! On clear / read without metadata keys, resolve **tenant → env default** (not env-only)
//! so "Use tenant defaults" matches ServerDefaultsCard.

use crate::types::{Tenant, Workspace};

/// Optional tenant defaults for LLM clear / inheritance (provider, model).
pub type TenantLlmDefaults<'a> = Option<(&'a str, &'a str)>;

/// Optional tenant defaults for embedding clear / inheritance (provider, model, dimension).
pub type TenantEmbeddingDefaults<'a> = Option<(&'a str, &'a str, usize)>;

/// True when the client intends to clear a workspace override (use server/env defaults).
pub fn is_clear_override_value(value: &str) -> bool {
    let normalized = value.trim().to_ascii_lowercase();
    normalized.is_empty() || normalized == "none"
}

fn resolve_llm_defaults(tenant: TenantLlmDefaults<'_>) -> (String, String) {
    if let Some((provider, model)) = tenant {
        let provider = provider.trim();
        let model = model.trim();
        if !provider.is_empty() && !model.is_empty() {
            return (model.to_string(), provider.to_string());
        }
    }
    Workspace::default_llm_config()
}

fn resolve_embedding_defaults(tenant: TenantEmbeddingDefaults<'_>) -> (String, String, usize) {
    if let Some((provider, model, dimension)) = tenant {
        let provider = provider.trim();
        let model = model.trim();
        if !provider.is_empty() && !model.is_empty() {
            let dim = if dimension == 0 {
                Workspace::detect_dimension_from_model(model)
            } else {
                dimension
            };
            return (model.to_string(), provider.to_string(), dim);
        }
    }
    Workspace::default_embedding_config()
}

/// Apply LLM model/provider update; empty values reset to tenant → [`Workspace::default_llm_config`].
pub fn apply_llm_config_update(
    workspace: &mut Workspace,
    llm_model: Option<String>,
    llm_provider: Option<String>,
) {
    apply_llm_config_update_with_tenant(workspace, llm_model, llm_provider, None);
}

/// Like [`apply_llm_config_update`], with optional tenant defaults on clear.
pub fn apply_llm_config_update_with_tenant(
    workspace: &mut Workspace,
    llm_model: Option<String>,
    llm_provider: Option<String>,
    tenant: TenantLlmDefaults<'_>,
) {
    let clear = llm_model
        .as_ref()
        .is_some_and(|v| is_clear_override_value(v))
        || llm_provider
            .as_ref()
            .is_some_and(|v| is_clear_override_value(v));

    if clear {
        let (model, provider) = resolve_llm_defaults(tenant);
        workspace.llm_model = model;
        workspace.llm_provider = provider;
        workspace.metadata.remove("llm_model");
        workspace.metadata.remove("llm_provider");
        return;
    }

    if let Some(llm_model) = llm_model {
        workspace.llm_model = llm_model.clone();
        workspace
            .metadata
            .insert("llm_model".to_string(), serde_json::json!(llm_model));
    }
    if let Some(llm_provider) = llm_provider {
        workspace.llm_provider = llm_provider.clone();
        workspace
            .metadata
            .insert("llm_provider".to_string(), serde_json::json!(llm_provider));
    }
}

/// Apply embedding update; empty values reset to tenant → [`Workspace::default_embedding_config`].
pub fn apply_embedding_config_update(
    workspace: &mut Workspace,
    embedding_model: Option<String>,
    embedding_provider: Option<String>,
    embedding_dimension: Option<usize>,
) {
    apply_embedding_config_update_with_tenant(
        workspace,
        embedding_model,
        embedding_provider,
        embedding_dimension,
        None,
    );
}

/// Like [`apply_embedding_config_update`], with optional tenant defaults on clear.
pub fn apply_embedding_config_update_with_tenant(
    workspace: &mut Workspace,
    embedding_model: Option<String>,
    embedding_provider: Option<String>,
    embedding_dimension: Option<usize>,
    tenant: TenantEmbeddingDefaults<'_>,
) {
    let clear = embedding_model
        .as_ref()
        .is_some_and(|v| is_clear_override_value(v))
        || embedding_provider
            .as_ref()
            .is_some_and(|v| is_clear_override_value(v))
        || embedding_dimension == Some(0);

    if clear {
        let (model, provider, dimension) = resolve_embedding_defaults(tenant);
        workspace.embedding_model = model;
        workspace.embedding_provider = provider;
        workspace.embedding_dimension = dimension;
        workspace.metadata.remove("embedding_model");
        workspace.metadata.remove("embedding_provider");
        workspace.metadata.remove("embedding_dimension");
        return;
    }

    if let Some(embedding_model) = embedding_model {
        workspace.embedding_model = embedding_model.clone();
        workspace.metadata.insert(
            "embedding_model".to_string(),
            serde_json::json!(embedding_model),
        );
    }
    if let Some(embedding_provider) = embedding_provider {
        workspace.embedding_provider = embedding_provider.clone();
        workspace.metadata.insert(
            "embedding_provider".to_string(),
            serde_json::json!(embedding_provider),
        );
    }
    if let Some(embedding_dimension) = embedding_dimension {
        workspace.embedding_dimension = embedding_dimension;
        workspace.metadata.insert(
            "embedding_dimension".to_string(),
            serde_json::json!(embedding_dimension),
        );
    }
}

fn metadata_has_nonempty(
    metadata: &std::collections::HashMap<String, serde_json::Value>,
    key: &str,
) -> bool {
    metadata
        .get(key)
        .and_then(|v| v.as_str())
        .is_some_and(|s| !s.trim().is_empty())
}

/// When metadata lacks llm/embedding keys, fill DTO fields from tenant → env default.
///
/// Call after loading a workspace so GET/list/update responses match ServerDefaultsCard.
pub fn resolve_inherited_model_fields(workspace: &mut Workspace, tenant: Option<&Tenant>) {
    let has_llm = metadata_has_nonempty(&workspace.metadata, "llm_model")
        || metadata_has_nonempty(&workspace.metadata, "llm_provider");
    if !has_llm {
        let (model, provider) = resolve_llm_defaults(tenant.map(|t| {
            (
                t.default_llm_provider.as_str(),
                t.default_llm_model.as_str(),
            )
        }));
        workspace.llm_model = model;
        workspace.llm_provider = provider;
    }

    let has_emb = metadata_has_nonempty(&workspace.metadata, "embedding_model")
        || metadata_has_nonempty(&workspace.metadata, "embedding_provider");
    if !has_emb {
        let (model, provider, dimension) = resolve_embedding_defaults(tenant.map(|t| {
            (
                t.default_embedding_provider.as_str(),
                t.default_embedding_model.as_str(),
                t.default_embedding_dimension,
            )
        }));
        workspace.embedding_model = model;
        workspace.embedding_provider = provider;
        workspace.embedding_dimension = dimension;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn restore_env(key: &str, value: Option<String>) {
        match value {
            Some(v) => std::env::set_var(key, v),
            None => std::env::remove_var(key),
        }
    }

    #[test]
    #[serial_test::serial]
    fn clear_llm_resets_to_env_defaults() {
        let key_runtime = "EDGEQUAKE_LLM_PROVIDER";
        let key_model = "EDGEQUAKE_LLM_MODEL";
        let key_default = "EDGEQUAKE_DEFAULT_LLM_PROVIDER";
        let prev_runtime = std::env::var(key_runtime).ok();
        let prev_model = std::env::var(key_model).ok();
        let prev_default = std::env::var(key_default).ok();

        std::env::remove_var(key_default);
        std::env::set_var(key_runtime, "ollama");
        std::env::set_var(key_model, "gemma4:latest");

        let mut ws = Workspace::new(Uuid::new_v4(), "t", "t");
        ws.llm_provider = "mock".to_string();
        ws.llm_model = "stale-model".to_string();
        ws.metadata
            .insert("llm_provider".to_string(), serde_json::json!("mock"));
        ws.metadata
            .insert("llm_model".to_string(), serde_json::json!("stale-model"));

        apply_llm_config_update(&mut ws, Some(String::new()), Some(String::new()));

        assert_eq!(ws.llm_provider, "ollama");
        assert_eq!(ws.llm_model, "gemma4:latest");
        assert!(!ws.metadata.contains_key("llm_provider"));
        assert!(!ws.metadata.contains_key("llm_model"));

        restore_env(key_default, prev_default);
        restore_env(key_runtime, prev_runtime);
        restore_env(key_model, prev_model);
    }

    #[test]
    #[serial_test::serial]
    fn clear_llm_prefers_tenant_over_env() {
        let key_runtime = "EDGEQUAKE_LLM_PROVIDER";
        let key_model = "EDGEQUAKE_LLM_MODEL";
        let prev_runtime = std::env::var(key_runtime).ok();
        let prev_model = std::env::var(key_model).ok();

        std::env::set_var(key_runtime, "ollama");
        std::env::set_var(key_model, "gemma4:latest");

        let mut ws = Workspace::new(Uuid::new_v4(), "t", "t");
        ws.llm_provider = "ollama".into();
        ws.llm_model = "gemma4:latest".into();
        ws.metadata
            .insert("llm_provider".to_string(), serde_json::json!("ollama"));
        ws.metadata
            .insert("llm_model".to_string(), serde_json::json!("gemma4:latest"));

        apply_llm_config_update_with_tenant(
            &mut ws,
            Some(String::new()),
            Some(String::new()),
            Some(("mistral", "mistral-small-latest")),
        );

        assert_eq!(ws.llm_provider, "mistral");
        assert_eq!(ws.llm_model, "mistral-small-latest");
        assert!(!ws.metadata.contains_key("llm_provider"));
        assert!(!ws.metadata.contains_key("llm_model"));

        restore_env(key_runtime, prev_runtime);
        restore_env(key_model, prev_model);
    }

    #[test]
    #[serial_test::serial]
    fn clear_embedding_prefers_tenant_over_env() {
        let key_runtime = "EDGEQUAKE_EMBEDDING_PROVIDER";
        let key_model = "EDGEQUAKE_EMBEDDING_MODEL";
        let prev_runtime = std::env::var(key_runtime).ok();
        let prev_model = std::env::var(key_model).ok();

        std::env::set_var(key_runtime, "ollama");
        std::env::set_var(key_model, "embeddinggemma");

        let mut ws = Workspace::new(Uuid::new_v4(), "t", "t");
        ws.embedding_provider = "ollama".into();
        ws.embedding_model = "embeddinggemma".into();
        ws.embedding_dimension = 768;
        ws.metadata.insert(
            "embedding_provider".to_string(),
            serde_json::json!("ollama"),
        );
        ws.metadata.insert(
            "embedding_model".to_string(),
            serde_json::json!("embeddinggemma"),
        );

        apply_embedding_config_update_with_tenant(
            &mut ws,
            Some(String::new()),
            Some(String::new()),
            Some(0),
            Some(("mistral", "mistral-embed", 1024)),
        );

        assert_eq!(ws.embedding_provider, "mistral");
        assert_eq!(ws.embedding_model, "mistral-embed");
        assert_eq!(ws.embedding_dimension, 1024);
        assert!(!ws.metadata.contains_key("embedding_provider"));
        assert!(!ws.metadata.contains_key("embedding_model"));

        restore_env(key_runtime, prev_runtime);
        restore_env(key_model, prev_model);
    }

    #[test]
    #[serial_test::serial]
    fn resolve_inherited_uses_tenant_when_metadata_empty() {
        let key_runtime = "EDGEQUAKE_LLM_PROVIDER";
        let key_model = "EDGEQUAKE_LLM_MODEL";
        let prev_runtime = std::env::var(key_runtime).ok();
        let prev_model = std::env::var(key_model).ok();

        std::env::set_var(key_runtime, "ollama");
        std::env::set_var(key_model, "gemma4:latest");

        let mut tenant = Tenant::new("Acme", "acme");
        tenant.default_llm_provider = "mistral".into();
        tenant.default_llm_model = "mistral-small-latest".into();
        tenant.default_embedding_provider = "mistral".into();
        tenant.default_embedding_model = "mistral-embed".into();
        tenant.default_embedding_dimension = 1024;

        let mut ws = Workspace::new(tenant.tenant_id, "ws", "ws");
        // Simulate into_workspace env fill with no metadata overrides.
        ws.metadata.remove("llm_model");
        ws.metadata.remove("llm_provider");
        ws.metadata.remove("embedding_model");
        ws.metadata.remove("embedding_provider");
        ws.metadata.remove("embedding_dimension");
        ws.llm_provider = "ollama".into();
        ws.llm_model = "gemma4:latest".into();
        ws.embedding_provider = "ollama".into();
        ws.embedding_model = "embeddinggemma".into();

        resolve_inherited_model_fields(&mut ws, Some(&tenant));

        assert_eq!(ws.llm_provider, "mistral");
        assert_eq!(ws.llm_model, "mistral-small-latest");
        assert_eq!(ws.embedding_provider, "mistral");
        assert_eq!(ws.embedding_model, "mistral-embed");
        assert_eq!(ws.embedding_dimension, 1024);

        restore_env(key_runtime, prev_runtime);
        restore_env(key_model, prev_model);
    }

    #[test]
    #[serial_test::serial]
    fn clear_llm_without_tenant_uses_default_llm_config() {
        let key_default = "EDGEQUAKE_DEFAULT_LLM_PROVIDER";
        let key_runtime = "EDGEQUAKE_LLM_PROVIDER";
        let key_model = "EDGEQUAKE_LLM_MODEL";
        let prev_default = std::env::var(key_default).ok();
        let prev_runtime = std::env::var(key_runtime).ok();
        let prev_model = std::env::var(key_model).ok();

        std::env::set_var(key_default, "ollama");
        std::env::set_var(key_runtime, "mistral");
        std::env::set_var(key_model, "mistral-small-latest");

        let mut ws = Workspace::new(Uuid::new_v4(), "t", "t");
        ws.llm_provider = "mock".into();
        ws.llm_model = "stale".into();

        apply_llm_config_update(&mut ws, Some(String::new()), Some(String::new()));

        let (expected_model, expected_provider) = Workspace::default_llm_config();
        assert_eq!(ws.llm_provider, expected_provider);
        assert_eq!(ws.llm_model, expected_model);

        restore_env(key_default, prev_default);
        restore_env(key_runtime, prev_runtime);
        restore_env(key_model, prev_model);
    }
}
