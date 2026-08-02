//! SPEC-101 — Secure first-run setup status + initialize.
//!
//! Public endpoints used by the First-Run Wizard before any login-capable user exists.

use axum::{extract::State, http::StatusCode, Json};
use edgequake_core::{CreateWorkspaceRequest, Tenant, TenantPlan, UpdateWorkspaceRequest};
use serde::{Deserialize, Serialize};
use tracing::info;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::error::ApiError;
use crate::handlers::workspaces_types::{TenantResponse, WorkspaceResponse};
use crate::state::AppState;

/// Whether silent Default tenant/workspace should be seeded at boot (LAW-101-7).
pub fn should_provision_defaults_at_boot(auth_enabled: bool, dev_mode: bool) -> bool {
    if std::env::var("EDGEQUAKE_PROVISION_DEFAULTS")
        .map(|v| v.eq_ignore_ascii_case("true") || v == "1")
        .unwrap_or(false)
    {
        return true;
    }
    if std::env::var("EDGEQUAKE_BOOTSTRAP_ADMIN_PASSWORD")
        .map(|v| !v.trim().is_empty())
        .unwrap_or(false)
    {
        return true;
    }
    !auth_enabled || dev_mode
}

fn generate_slug(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<&str>>()
        .join("-")
}

fn tenant_to_response(tenant: &Tenant) -> TenantResponse {
    TenantResponse {
        id: tenant.tenant_id,
        name: tenant.name.clone(),
        slug: tenant.slug.clone(),
        plan: format!("{}", tenant.plan),
        is_active: tenant.is_active,
        max_workspaces: tenant.max_workspaces,
        default_llm_model: tenant.default_llm_model.clone(),
        default_llm_provider: tenant.default_llm_provider.clone(),
        default_llm_full_id: format!(
            "{}/{}",
            tenant.default_llm_provider, tenant.default_llm_model
        ),
        default_embedding_model: tenant.default_embedding_model.clone(),
        default_embedding_provider: tenant.default_embedding_provider.clone(),
        default_embedding_dimension: tenant.default_embedding_dimension,
        default_embedding_full_id: format!(
            "{}/{}",
            tenant.default_embedding_provider, tenant.default_embedding_model
        ),
        default_vision_llm_model: tenant.default_vision_llm_model.clone(),
        default_vision_llm_provider: tenant.default_vision_llm_provider.clone(),
        created_at: tenant.created_at.to_rfc3339(),
        updated_at: tenant.updated_at.to_rfc3339(),
    }
}

fn workspace_to_response(workspace: &edgequake_core::Workspace) -> WorkspaceResponse {
    WorkspaceResponse {
        id: workspace.workspace_id,
        tenant_id: workspace.tenant_id,
        name: workspace.name.clone(),
        slug: workspace.slug.clone(),
        description: workspace.description.clone(),
        is_active: workspace.is_active,
        max_documents: workspace.max_documents(),
        llm_model: workspace.llm_model.clone(),
        llm_provider: workspace.llm_provider.clone(),
        llm_full_id: workspace.llm_full_id(),
        embedding_model: workspace.embedding_model.clone(),
        embedding_provider: workspace.embedding_provider.clone(),
        embedding_dimension: workspace.embedding_dimension,
        embedding_full_id: workspace.embedding_full_id(),
        vision_llm_provider: workspace.vision_llm_provider.clone(),
        vision_llm_model: workspace.vision_llm_model.clone(),
        pdf_parser_backend: workspace
            .pdf_parser_backend
            .map(|backend| backend.as_str().to_string()),
        entity_types: workspace
            .metadata
            .get("entity_types")
            .and_then(|v| serde_json::from_value::<Vec<String>>(v.clone()).ok()),
        entity_types_strict: workspace
            .metadata
            .get("entity_types_strict")
            .and_then(|v| v.as_bool())
            .unwrap_or(true),
        extraction_language: workspace
            .metadata
            .get("extraction_language")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        created_at: workspace.created_at.to_rfc3339(),
        updated_at: workspace.updated_at.to_rfc3339(),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SetupStatusResponse {
    pub needs_setup: bool,
    pub has_login_users: bool,
    pub tenant_count: u64,
    pub workspace_count: u64,
    pub auth_enabled: bool,
    pub bootstrap_admin_configured: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SetupInitializeRequest {
    #[serde(default)]
    pub admin_username: Option<String>,
    #[serde(default)]
    pub admin_email: Option<String>,
    #[serde(default)]
    pub admin_password: Option<String>,
    pub tenant_name: String,
    #[serde(default)]
    pub tenant_description: Option<String>,
    pub workspace_name: String,
    #[serde(default)]
    pub workspace_slug: Option<String>,
    #[serde(default)]
    pub workspace_description: Option<String>,
    #[serde(default)]
    pub default_llm_model: Option<String>,
    #[serde(default)]
    pub default_llm_provider: Option<String>,
    #[serde(default)]
    pub default_embedding_model: Option<String>,
    #[serde(default)]
    pub default_embedding_provider: Option<String>,
    #[serde(default)]
    pub default_vision_llm_model: Option<String>,
    #[serde(default)]
    pub default_vision_llm_provider: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SetupInitializeResponse {
    pub tenant: TenantResponse,
    pub workspace: WorkspaceResponse,
    pub admin_username: Option<String>,
    pub already_initialized: bool,
}

async fn collect_setup_status(state: &AppState) -> Result<SetupStatusResponse, ApiError> {
    let auth_enabled = state.auth.config.auth_enabled;
    let bootstrap_admin_configured = std::env::var("EDGEQUAKE_BOOTSTRAP_ADMIN_PASSWORD")
        .map(|v| !v.trim().is_empty())
        .unwrap_or(false);

    let tenants = state
        .workspace_service
        .list_tenants(1000, 0)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    let tenant_count = tenants.len() as u64;

    let mut workspace_count = 0u64;
    for tenant in &tenants {
        let wss = state
            .workspace_service
            .list_workspaces(tenant.tenant_id)
            .await
            .map_err(|e| ApiError::Internal(e.to_string()))?;
        workspace_count += wss.len() as u64;
    }

    let has_login_users = count_login_users(state).await?;

    let needs_setup = if auth_enabled && !state.auth.config.dev_mode {
        !has_login_users || tenant_count == 0
    } else {
        tenant_count == 0
    };

    Ok(SetupStatusResponse {
        needs_setup,
        has_login_users,
        tenant_count,
        workspace_count,
        auth_enabled,
        bootstrap_admin_configured,
    })
}

async fn count_login_users(state: &AppState) -> Result<bool, ApiError> {
    #[cfg(feature = "postgres")]
    {
        if let Some(pool) = state.pg_pool.as_ref() {
            let n = crate::services::identity_storage::count_login_capable_users_pg(
                pool,
                &state.security,
            )
            .await
            .map_err(|e| ApiError::Internal(e.to_string()))?;
            return Ok(n > 0);
        }
        Ok(false)
    }
    #[cfg(not(feature = "postgres"))]
    {
        let _ = state;
        Ok(false)
    }
}

async fn create_first_run_admin(
    state: &AppState,
    request: &SetupInitializeRequest,
) -> Result<Option<String>, ApiError> {
    #[cfg(feature = "postgres")]
    {
        use chrono::Utc;
        use edgequake_auth::Role;

        use crate::handlers::auth::{persist_user_record, UserRecord};
        use crate::state::PostgresRuntime;

        let username = request
            .admin_username
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .unwrap_or("admin");
        let password = request
            .admin_password
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .ok_or_else(|| ApiError::BadRequest("admin_password is required".to_string()))?;
        if password.len() < 8 {
            return Err(ApiError::BadRequest(
                "admin_password must be at least 8 characters".to_string(),
            ));
        }
        if username.len() < 3 {
            return Err(ApiError::BadRequest(
                "admin_username must be at least 3 characters".to_string(),
            ));
        }
        let email = request
            .admin_email
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("{username}@localhost"));

        let password_hash = state
            .auth
            .password
            .hash_password(password)
            .map_err(|e| ApiError::Internal(format!("password hash failed: {e}")))?;

        let now = Utc::now();
        let record = UserRecord {
            user_id: Uuid::new_v4().to_string(),
            username: username.to_string(),
            email,
            password_hash,
            role: Role::Admin.to_string(),
            is_active: true,
            created_at: now,
            updated_at: now,
            last_login_at: None,
            failed_login_attempts: 0,
            locked_until: None,
            metadata: Default::default(),
        };

        let pg_runtime = PostgresRuntime {
            pool: state.pg_pool.clone(),
            capabilities: state.postgres_capabilities.clone(),
        };

        persist_user_record(&state.storage, Some(&pg_runtime), &state.security, &record).await?;

        info!(username = %username, "SPEC-101: created first-run admin via /setup/initialize");
        Ok(Some(username.to_string()))
    }
    #[cfg(not(feature = "postgres"))]
    {
        let _ = (state, request);
        Err(ApiError::BadRequest(
            "First-run admin creation requires PostgreSQL".to_string(),
        ))
    }
}

/// GET /api/v1/setup/status
#[utoipa::path(
    get,
    path = "/api/v1/setup/status",
    responses(
        (status = 200, description = "Setup status", body = SetupStatusResponse),
    ),
    tags = ["setup"],
    security(())
)]
pub async fn setup_status(
    State(state): State<AppState>,
) -> Result<Json<SetupStatusResponse>, ApiError> {
    Ok(Json(collect_setup_status(&state).await?))
}

/// POST /api/v1/setup/initialize
#[utoipa::path(
    post,
    path = "/api/v1/setup/initialize",
    request_body = SetupInitializeRequest,
    responses(
        (status = 201, description = "Initialized", body = SetupInitializeResponse),
        (status = 409, description = "Already initialized"),
        (status = 400, description = "Invalid request"),
    ),
    tags = ["setup"],
    security(())
)]
pub async fn setup_initialize(
    State(state): State<AppState>,
    Json(request): Json<SetupInitializeRequest>,
) -> Result<(StatusCode, Json<SetupInitializeResponse>), ApiError> {
    let status = collect_setup_status(&state).await?;
    if !status.needs_setup {
        return Err(ApiError::Conflict(
            "Instance already initialized".to_string(),
        ));
    }

    if request.tenant_name.trim().is_empty() {
        return Err(ApiError::BadRequest("tenant_name is required".to_string()));
    }
    if request.workspace_name.trim().is_empty() {
        return Err(ApiError::BadRequest(
            "workspace_name is required".to_string(),
        ));
    }

    let mut admin_username_out: Option<String> = None;

    if status.auth_enabled
        && !state.auth.config.dev_mode
        && !status.has_login_users
        && !status.bootstrap_admin_configured
    {
        admin_username_out = create_first_run_admin(&state, &request).await?;
    }

    let slug = generate_slug(&request.tenant_name);
    let mut tenant = Tenant::new(request.tenant_name.trim(), &slug).with_plan(TenantPlan::Pro);
    if let Some(desc) = request.tenant_description.as_ref() {
        tenant = tenant.with_description(desc);
    }
    if let (Some(model), Some(provider)) =
        (&request.default_llm_model, &request.default_llm_provider)
    {
        tenant = tenant.with_llm_config(model, provider);
    }
    if let (Some(model), Some(provider)) = (
        &request.default_embedding_model,
        &request.default_embedding_provider,
    ) {
        let dim = edgequake_core::Workspace::detect_dimension_from_model(model);
        tenant = tenant.with_embedding_config(model, provider, dim);
    }
    if let (Some(model), Some(provider)) = (
        &request.default_vision_llm_model,
        &request.default_vision_llm_provider,
    ) {
        tenant = tenant.with_vision_config(model, provider);
    }

    let created_tenant = state
        .workspace_service
        .create_tenant(tenant)
        .await
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;

    let mut default_ws = CreateWorkspaceRequest::new("Default Workspace").with_llm_config(
        &created_tenant.default_llm_model,
        &created_tenant.default_llm_provider,
    );
    default_ws = default_ws.with_embedding_config(
        &created_tenant.default_embedding_model,
        &created_tenant.default_embedding_provider,
        created_tenant.default_embedding_dimension,
    );
    default_ws.slug = Some("default".to_string());
    if let (Some(model), Some(provider)) = (
        created_tenant.default_vision_llm_model.as_ref(),
        created_tenant.default_vision_llm_provider.as_ref(),
    ) {
        default_ws.vision_llm_model = Some(model.clone());
        default_ws.vision_llm_provider = Some(provider.clone());
    }
    let _ = state
        .workspace_service
        .create_workspace(created_tenant.tenant_id, default_ws)
        .await;

    let workspaces = state
        .workspace_service
        .list_workspaces(created_tenant.tenant_id)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let ws_slug = request
        .workspace_slug
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .unwrap_or_else(|| generate_slug(&request.workspace_name));

    let workspace = if let Some(existing) = workspaces.first() {
        let mut update = UpdateWorkspaceRequest {
            name: Some(request.workspace_name.trim().to_string()),
            description: request.workspace_description.clone(),
            ..Default::default()
        };
        if let (Some(model), Some(provider)) =
            (&request.default_llm_model, &request.default_llm_provider)
        {
            update.llm_model = Some(model.clone());
            update.llm_provider = Some(provider.clone());
        }
        if let (Some(model), Some(provider)) = (
            &request.default_embedding_model,
            &request.default_embedding_provider,
        ) {
            update.embedding_model = Some(model.clone());
            update.embedding_provider = Some(provider.clone());
        }
        if let (Some(model), Some(provider)) = (
            &request.default_vision_llm_model,
            &request.default_vision_llm_provider,
        ) {
            update.vision_llm_model = Some(model.clone());
            update.vision_llm_provider = Some(provider.clone());
        }
        let _ = ws_slug;
        state
            .workspace_service
            .update_workspace(existing.workspace_id, update)
            .await
            .map_err(|e| ApiError::BadRequest(e.to_string()))?
    } else {
        let mut req = CreateWorkspaceRequest::new(request.workspace_name.trim());
        req.slug = Some(ws_slug);
        req.description = request.workspace_description.clone();
        if let (Some(model), Some(provider)) =
            (&request.default_llm_model, &request.default_llm_provider)
        {
            req = req.with_llm_config(model, provider);
        }
        if let (Some(model), Some(provider)) = (
            &request.default_embedding_model,
            &request.default_embedding_provider,
        ) {
            let dim = edgequake_core::Workspace::detect_dimension_from_model(model);
            req = req.with_embedding_config(model, provider, dim);
        }
        if let (Some(model), Some(provider)) = (
            &request.default_vision_llm_model,
            &request.default_vision_llm_provider,
        ) {
            req.vision_llm_model = Some(model.clone());
            req.vision_llm_provider = Some(provider.clone());
        }
        state
            .workspace_service
            .create_workspace(created_tenant.tenant_id, req)
            .await
            .map_err(|e| ApiError::BadRequest(e.to_string()))?
    };

    info!(
        tenant_id = %created_tenant.tenant_id,
        workspace_id = %workspace.workspace_id,
        "SPEC-101: first-run initialize completed"
    );

    Ok((
        StatusCode::CREATED,
        Json(SetupInitializeResponse {
            tenant: tenant_to_response(&created_tenant),
            workspace: workspace_to_response(&workspace),
            admin_username: admin_username_out,
            already_initialized: false,
        }),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn provision_defaults_true_when_flag_set() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        unsafe {
            std::env::set_var("EDGEQUAKE_PROVISION_DEFAULTS", "true");
            std::env::remove_var("EDGEQUAKE_BOOTSTRAP_ADMIN_PASSWORD");
        }
        assert!(should_provision_defaults_at_boot(true, false));
        unsafe {
            std::env::remove_var("EDGEQUAKE_PROVISION_DEFAULTS");
        }
    }

    #[test]
    fn skip_silent_defaults_on_secure_fresh_install() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        unsafe {
            std::env::remove_var("EDGEQUAKE_PROVISION_DEFAULTS");
            std::env::remove_var("EDGEQUAKE_BOOTSTRAP_ADMIN_PASSWORD");
        }
        assert!(!should_provision_defaults_at_boot(true, false));
        assert!(should_provision_defaults_at_boot(false, false));
        assert!(should_provision_defaults_at_boot(true, true));
    }
}
