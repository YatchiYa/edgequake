//! `POST /api/v1/tenants/{tenant_id}/members/{user_id}/access-links` (SPEC-169).

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use edgequake_auth::Role;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use super::guards::{check_issue, LinkTarget};
use crate::error::ApiError;
use crate::handlers::auth::ApiAuthenticated;
use crate::handlers::members::directory::{
    actor_for, grants_of, parse_user_id, require_tenant, tenant_memberships, user_record,
};
use crate::handlers::members::view::primary_membership;
use crate::handlers::workspaces::tenant_access::{require_tenant_access, TenantAccess};
use crate::services::access_links::{AccessLink, AccessPurpose};
use crate::services::invitations::token::{hash_token, new_token};
use crate::state::AppState;

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct IssueAccessLinkRequest {
    /// `magic_login` (sign in, no password) or `password_reset` (choose a new password).
    pub purpose: String,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct AccessLinkIssued {
    /// Single-use token. Shown once: only its hash is stored. The UI builds `/access/{token}`.
    pub token: String,
    pub purpose: String,
    pub username: String,
    pub email: String,
    pub expires_at: String,
    pub expires_in_minutes: i64,
}

fn internal(e: impl std::fmt::Display) -> ApiError {
    ApiError::Internal(e.to_string())
}

/// Create a one-time sign-in or password-reset link for a member.
#[utoipa::path(
    post,
    path = "/api/v1/tenants/{tenant_id}/members/{user_id}/access-links",
    tag = "Members",
    security(("bearer_auth" = [])),
    params(
        ("tenant_id" = Uuid, Path, description = "Organization id"),
        ("user_id" = String, Path, description = "User id")
    ),
    request_body = IssueAccessLinkRequest,
    responses(
        (status = 201, description = "Link created (token shown once)", body = AccessLinkIssued),
        (status = 400, description = "Unknown purpose"),
        (status = 403, description = "Target outranks you or is reachable by platform admins only"),
        (status = 404, description = "Member not found"),
        (status = 409, description = "Suspended, SSO-managed or yourself")
    )
)]
pub async fn issue_access_link(
    State(state): State<AppState>,
    auth: ApiAuthenticated,
    Path((tenant_id, user_id)): Path<(Uuid, String)>,
    Json(body): Json<IssueAccessLinkRequest>,
) -> Result<(StatusCode, Json<AccessLinkIssued>), ApiError> {
    let ctx = auth.context();
    let purpose = AccessPurpose::parse(&body.purpose)
        .ok_or_else(|| ApiError::BadRequest(format!("Unknown purpose: {}", body.purpose)))?;
    require_tenant_access(&state, ctx, tenant_id, TenantAccess::Manage).await?;
    require_tenant(&state, tenant_id).await?;
    let actor = actor_for(&state, ctx, tenant_id).await?;

    let target_id = parse_user_id(&user_id)?;
    let memberships = tenant_memberships(&state, tenant_id).await?;
    let grants = grants_of(&memberships, target_id);
    let membership = primary_membership(&grants)
        .ok_or_else(|| ApiError::NotFound("Member not found in this organization".into()))?;
    let record = user_record(&state, &target_id.to_string())
        .await?
        .ok_or_else(|| ApiError::NotFound("User not found".into()))?;

    let other_orgs = state
        .workspace_service
        .get_user_memberships(target_id)
        .await
        .map_err(internal)?
        .iter()
        .any(|m| m.is_active && m.tenant_id != tenant_id);
    let sso_managed = !state
        .auth
        .federation
        .identities_for_users(&[target_id])
        .await?
        .is_empty();
    check_issue(
        &actor,
        &LinkTarget {
            is_self: ctx.user_id == record.user_id,
            is_active: record.is_active,
            is_instance_admin: Role::parse(&record.role) == Role::Admin,
            in_other_organizations: other_orgs,
            sso_managed,
            role: membership.role,
        },
    )?;

    let token = new_token();
    let issuer = Uuid::parse_str(&ctx.user_id).ok().zip(
        user_record(&state, &ctx.user_id)
            .await
            .ok()
            .flatten()
            .map(|r| r.username),
    );
    let link = AccessLink::new(target_id, tenant_id, purpose, hash_token(&token), issuer);
    state.auth.access_links.issue(&link).await?;
    tracing::info!(
        %tenant_id, target = %target_id, purpose = purpose.as_str(), actor = %ctx.user_id,
        "access link issued"
    );

    Ok((
        StatusCode::CREATED,
        Json(AccessLinkIssued {
            token,
            purpose: purpose.as_str().to_string(),
            username: record.username,
            email: record.email,
            expires_at: link.expires_at.to_rfc3339(),
            expires_in_minutes: purpose.ttl().num_minutes(),
        }),
    ))
}
