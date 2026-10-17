//! Session scope switch (SPEC-158 UX "Post-login tenant chrome": re-issue session on switch).
//!
//! Access JWTs are bound to one tenant/workspace (SPEC-027 IMP-004), so changing the
//! `X-Tenant-ID` / `X-Workspace-ID` headers alone is rejected. This endpoint re-binds the
//! caller's *existing* web session (the `eq_refresh` cookie family) to a new scope after a
//! server-side authorization check; refresh keeps that scope and re-checks it every time.
//!
//! Security: a bare access token is not enough — the refresh cookie must be presented and
//! is rotated within its own family, so switching can never mint a new long-lived session,
//! and logout / IdP back-channel logout still end it.

use axum::{extract::State, http::HeaderMap, response::Response, Json};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tracing::info;
use utoipa::ToSchema;
use uuid::Uuid;

use axum::extract::FromRef;
use edgequake_audit::{AuditEventType, AuditResult};
use edgequake_auth::Role;

use crate::error::ApiError;
use crate::services::auth_memory_store::TakeWebRefreshOutcome;
use crate::services::auth_validation::{decide, TokenProfile};
use crate::services::federation::scope_guard::revalidate_scope;
use crate::services::federation::store::FederatedSession;
use crate::services::login_tokens::{
    mint_access_token, persist_new_refresh, SessionContext, SessionScope,
};
use crate::services::request_authorization::{authorize_scope_in, membership_role_in};
use crate::state::{AppState, PostgresRuntime};

use super::get_record_by_id;
use super::refresh_cookie::{
    cookie_secure_from_headers, refresh_token_for_json_body, resolve_refresh_token,
};
use super::session::with_refresh_cookie;

/// `federated_sessions.provider_slug` marking a password session re-bound by this endpoint.
pub(crate) const LOCAL_SCOPE_PROVIDER: &str = "local";

#[derive(Debug, Deserialize, ToSchema)]
pub struct SwitchScopeRequest {
    pub tenant_id: Uuid,
    /// Omit to bind the first workspace of the tenant the caller can use (tenant switch).
    #[serde(default)]
    pub workspace_id: Option<Uuid>,
    /// Non-browser clients may send the refresh token here instead of the `eq_refresh` cookie.
    #[serde(default)]
    pub refresh_token: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SwitchScopeResponse {
    pub access_token: String,
    /// Always `Bearer`.
    pub token_type: String,
    pub expires_in: i64,
    /// Omitted for browser clients (HttpOnly `eq_refresh` cookie instead).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    pub tenant_id: String,
    pub workspace_id: String,
}

/// POST /api/v1/auth/switch-scope — re-bind the web session to another tenant/workspace.
#[utoipa::path(
    post,
    path = "/api/v1/auth/switch-scope",
    tag = "Authentication",
    request_body = SwitchScopeRequest,
    responses(
        (status = 200, description = "Session re-bound to the requested scope (refresh token rotated)", body = SwitchScopeResponse),
        (status = 401, description = "No valid web session or refresh token"),
        (status = 403, description = "No access to the requested scope, or API key credential")
    )
)]
pub async fn switch_scope(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<SwitchScopeRequest>,
) -> Result<Response, ApiError> {
    let credential = crate::middleware::extract_token_from_headers(&headers)
        .ok_or_else(ApiError::unauthorized)?;
    let decision = decide(&state, &credential, TokenProfile::WebSession, &headers)
        .await?
        .ok_or_else(ApiError::unauthorized)?;
    // API keys must never be exchanged for a web session (scope escalation).
    if decision.profile != TokenProfile::WebSession {
        return Err(ApiError::forbidden_reason("web_session_required"));
    }
    let presented = resolve_refresh_token(request.refresh_token.as_deref(), &headers)
        .ok_or_else(|| ApiError::auth_unauthorized("switch_scope", "refresh_token_missing", None))?;
    let user_id = decision.authenticated.auth.user_id;
    let user_uuid =
        Uuid::parse_str(&user_id).map_err(|_| ApiError::Internal("invalid user id".into()))?;

    let pg = PostgresRuntime::from_ref(&state);
    let record = get_record_by_id(
        &state.storage,
        Some(&pg),
        &state.security,
        state.operational_stores.identity.as_deref(),
        &user_id,
    )
    .await?
    .filter(|u| u.is_active)
    .ok_or_else(ApiError::unauthorized)?;
    let local_role = Role::parse(&record.role);
    let workspaces = state.workspace_service.as_ref();

    // Authorize BEFORE consuming the refresh token: a refused switch keeps the session.
    let tenant = request.tenant_id.to_string();
    let workspace_id = match request.workspace_id {
        Some(id) => id,
        None => first_usable_workspace(&state, &user_id, &local_role, request.tenant_id).await?,
    };
    let workspace = workspace_id.to_string();
    authorize_scope_in(workspaces, &user_id, &local_role, Some(&tenant), Some(&workspace)).await?;
    let platform_admin_entry = local_role == Role::Admin
        && membership_role_in(workspaces, &user_id, Some(&tenant), Some(&workspace))
            .await
            .is_err();

    let consumed = match crate::services::session_storage::take_web_refresh(
        &state.storage,
        Some(&pg),
        &state.security,
        state.operational_stores.sessions.as_deref(),
        &presented,
    )
    .await?
    {
        TakeWebRefreshOutcome::Consumed(r) if r.user_id == user_id => *r,
        _ => return Err(ApiError::auth_unauthorized("switch_scope", "refresh_token_invalid", None)),
    };
    let family_id = consumed.family_id;

    let scope = SessionScope {
        tenant_id: request.tenant_id,
        workspace_id: Some(workspace_id),
    };
    let existing = state.auth.federation.get_session(family_id).await?;
    if existing.as_ref().is_some_and(|s| s.revoked_at.is_some()) {
        return Err(ApiError::auth_unauthorized("switch_scope", "session_revoked", None));
    }
    // SSO families keep their IdP binding (back-channel logout) and membership-derived role.
    let (session, role) = match existing {
        Some(s) if s.provider_slug != LOCAL_SCOPE_PROVIDER => {
            let role = revalidate_scope(workspaces, user_uuid, &scope, &record.role).await?;
            (
                FederatedSession {
                    tenant_id: scope.tenant_id,
                    workspace_id: scope.workspace_id,
                    ..s
                },
                role,
            )
        }
        _ => (
            FederatedSession {
                family_id,
                user_id: user_uuid,
                provider_slug: LOCAL_SCOPE_PROVIDER.to_string(),
                issuer: LOCAL_SCOPE_PROVIDER.to_string(),
                subject: user_id.clone(),
                idp_sid: None,
                tenant_id: scope.tenant_id,
                workspace_id: scope.workspace_id,
                created_at: Utc::now(),
                revoked_at: None,
            },
            local_role,
        ),
    };

    let ctx = SessionContext {
        auth: &state.auth,
        storage: &state.storage,
        pg: &pg,
        security: &state.security,
        stores: &state.operational_stores,
    };
    let refresh_token = persist_new_refresh(&ctx, &user_id, family_id).await?;
    let (access_token, expires_in) = mint_access_token(&state.auth, user_uuid, role, &scope)?;
    state.auth.federation.put_session(&session).await?;
    crate::services::federation::access_jti::remember_access_token(
        state.auth.federation.as_ref(),
        family_id,
        &access_token,
    )
    .await?;

    info!(user_id = %user_id, tenant = %tenant, workspace = %workspace, "Session scope switched");
    crate::services::record_compliance_event(
        &state,
        tenant.clone(),
        AuditEventType::Authorization,
        if platform_admin_entry {
            "platform_admin_scope_entry"
        } else {
            "switch_scope"
        },
        AuditResult::Success,
        Some(workspace.clone()),
        Some(user_id.clone()),
        None,
    );

    Ok(with_refresh_cookie(
        SwitchScopeResponse {
            access_token,
            token_type: "Bearer".into(),
            expires_in,
            refresh_token: refresh_token_for_json_body(&headers, &refresh_token),
            tenant_id: tenant,
            workspace_id: workspace,
        },
        &refresh_token,
        cookie_secure_from_headers(&headers),
    ))
}

/// Default-slug workspace first, then oldest, among those the caller may use.
async fn first_usable_workspace(
    state: &AppState,
    user_id: &str,
    role: &Role,
    tenant_id: Uuid,
) -> Result<Uuid, ApiError> {
    let mut workspaces = state
        .workspace_service
        .list_workspaces(tenant_id)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    workspaces.retain(|w| w.is_active);
    workspaces.sort_by_key(|w| (w.slug != "default", w.created_at));
    let tenant = tenant_id.to_string();
    for w in workspaces {
        let id = w.workspace_id.to_string();
        if authorize_scope_in(
            state.workspace_service.as_ref(),
            user_id,
            role,
            Some(&tenant),
            Some(&id),
        )
        .await
        .is_ok()
        {
            return Ok(w.workspace_id);
        }
    }
    Err(ApiError::forbidden_reason(
        "No accessible workspace in this tenant",
    ))
}
