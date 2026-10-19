//! Public invitation accept flow (SPEC-169): preview the link, then join.
//!
//! * New person: `{username, password}` creates a password account and joins.
//! * Existing account: sign in first, then accept; the signed-in email must match the invite
//!   (an unauthenticated caller can never attach a membership to someone else's account).
//! * SSO invitees need no token: IdP organization + role mapping provision them (SPEC-158).

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use edgequake_auth::Role;
use edgequake_core::{Membership, Tenant};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::error::ApiError;
use crate::handlers::auth::user_provision::{
    assert_login_available, find_user_by_login, provision_password_user, IdentityCtx,
};
use crate::handlers::auth::ApiOptionalAuth;
use crate::services::federation::registry::ProviderSummary;
use crate::services::identity_storage::default_identity_scope;
use crate::services::invitations::model::normalize_email;
use crate::services::invitations::token::hash_token;
use crate::services::invitations::Invitation;
use crate::services::role_catalog::organization_role_label;
use crate::state::AppState;

const MAX_TOKEN_LEN: usize = 256;

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct InvitationPreview {
    pub tenant_name: String,
    pub email: String,
    pub role: String,
    pub role_label: String,
    pub invited_by: Option<String>,
    pub expires_at: String,
    /// True when the email already has an account: the invitee must sign in to accept.
    pub account_exists: bool,
    /// Configured SSO providers (an alternative way in for the same address).
    pub sso_providers: Vec<ProviderSummary>,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct AcceptInvitationRequest {
    /// Required (with `password`) when the email has no account yet.
    pub username: Option<String>,
    pub password: Option<String>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct AcceptInvitationResponse {
    pub tenant_id: String,
    pub tenant_name: String,
    pub role: String,
    pub email: String,
    pub username: String,
    /// True when a new password account was created.
    pub created_account: bool,
}

fn invalid() -> ApiError {
    ApiError::NotFound("invitation_invalid".into())
}

/// Resolve a token to an open, unexpired invitation (404 unknown/used/revoked, 410 expired).
async fn usable_invitation(state: &AppState, token: &str) -> Result<Invitation, ApiError> {
    if token.is_empty() || token.len() > MAX_TOKEN_LEN {
        return Err(invalid());
    }
    let invitation = state
        .auth
        .invitations
        .find_open_by_token(&hash_token(token))
        .await?
        .ok_or_else(invalid)?;
    if !invitation.is_usable(Utc::now()) {
        return Err(ApiError::Gone("invitation_expired".into()));
    }
    Ok(invitation)
}

async fn active_tenant(state: &AppState, invitation: &Invitation) -> Result<Tenant, ApiError> {
    state
        .workspace_service
        .get_tenant(invitation.tenant_id)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .filter(|t| t.is_active)
        .ok_or_else(invalid)
}

/// Public preview of an invitation link.
#[utoipa::path(
    get,
    path = "/api/v1/auth/invitations/{token}",
    tag = "Members",
    params(("token" = String, Path, description = "Single-use accept token")),
    responses(
        (status = 200, description = "Invitation details", body = InvitationPreview),
        (status = 404, description = "Unknown, used or revoked invitation"),
        (status = 410, description = "Invitation expired")
    )
)]
pub async fn preview_invitation(
    State(state): State<AppState>,
    Path(token): Path<String>,
) -> Result<Json<InvitationPreview>, ApiError> {
    let invitation = usable_invitation(&state, &token).await?;
    let tenant = active_tenant(&state, &invitation).await?;
    let account_exists = find_user_by_login(&IdentityCtx::from_state(&state), &invitation.email)
        .await?
        .is_some();
    let sso_providers = state
        .auth
        .all_providers()
        .iter()
        .filter(|p| p.config().is_runtime_builtin())
        .map(|p| ProviderSummary::of(p))
        .collect();
    Ok(Json(InvitationPreview {
        tenant_name: tenant.name,
        email: invitation.email.clone(),
        role: invitation.role.to_string(),
        role_label: organization_role_label(invitation.role).to_string(),
        invited_by: invitation.invited_by_name.clone(),
        expires_at: invitation.expires_at.to_rfc3339(),
        account_exists,
        sso_providers,
    }))
}

/// Who is joining: an existing signed-in account, or a freshly created one.
struct Joiner {
    user_id: uuid::Uuid,
    username: String,
    created: bool,
}

async fn resolve_joiner(
    state: &AppState,
    invitation: &Invitation,
    caller: Option<&crate::handlers::auth::RequestAuthContext>,
    body: &AcceptInvitationRequest,
) -> Result<Joiner, ApiError> {
    if let Some(existing) =
        find_user_by_login(&IdentityCtx::from_state(state), &invitation.email).await?
    {
        let signed_in_as_invitee = caller.is_some_and(|c| c.user_id == existing.user_id);
        if !signed_in_as_invitee {
            return Err(ApiError::Unauthorized(None));
        }
        let user_id = uuid::Uuid::parse_str(&existing.user_id)
            .map_err(|_| ApiError::Internal("invalid user id".into()))?;
        return Ok(Joiner {
            user_id,
            username: existing.username,
            created: false,
        });
    }
    let (Some(username), Some(password)) = (
        body.username
            .as_deref()
            .map(str::trim)
            .filter(|u| !u.is_empty()),
        body.password.as_deref().filter(|p| !p.is_empty()),
    ) else {
        return Err(ApiError::BadRequest(
            "Username and password are required".into(),
        ));
    };
    let ctx = IdentityCtx::from_state(state);
    assert_login_available(&ctx, username, &invitation.email).await?;
    let user =
        provision_password_user(&ctx, username, &invitation.email, password, Role::User).await?;
    let user_id = uuid::Uuid::parse_str(&user.user_id)
        .map_err(|_| ApiError::Internal("invalid user id".into()))?;
    Ok(Joiner {
        user_id,
        username: user.username,
        created: true,
    })
}

/// A new account is auto-provisioned into the default organization on first persist; a person
/// invited elsewhere must not silently gain access to it.
async fn drop_default_provisioning(state: &AppState, joiner: &Joiner, keep_tenant: uuid::Uuid) {
    if !joiner.created {
        return;
    }
    let (default_tenant, default_workspace) = default_identity_scope();
    if keep_tenant == default_tenant {
        return;
    }
    let Ok(grants) = state
        .workspace_service
        .get_user_memberships(joiner.user_id)
        .await
    else {
        return;
    };
    for m in grants
        .iter()
        .filter(|m| m.tenant_id == default_tenant && m.workspace_id == Some(default_workspace))
    {
        if let Err(e) = state
            .workspace_service
            .deactivate_membership(m.membership_id)
            .await
        {
            tracing::warn!(error = %e, "could not drop default provisioning for invited user");
        }
    }
}

async fn grant(state: &AppState, joiner: &Joiner, invitation: &Invitation) -> Result<(), ApiError> {
    let mut membership = Membership::new(joiner.user_id, invitation.tenant_id, invitation.role);
    if let Some(workspace_id) = invitation.workspace_id {
        membership = membership.for_workspace(workspace_id);
    }
    membership.pin_manual_role();
    match state.workspace_service.add_membership(membership).await {
        Ok(_) => Ok(()),
        // Already a member: accepting is idempotent from the invitee's point of view.
        Err(e)
            if e.to_string()
                .to_ascii_lowercase()
                .contains("already exists") =>
        {
            Ok(())
        }
        Err(e) => Err(ApiError::Internal(e.to_string())),
    }
}

/// Accept an invitation and join the organization.
#[utoipa::path(
    post,
    path = "/api/v1/auth/invitations/{token}/accept",
    tag = "Members",
    params(("token" = String, Path, description = "Single-use accept token")),
    request_body = AcceptInvitationRequest,
    responses(
        (status = 201, description = "Joined", body = AcceptInvitationResponse),
        (status = 400, description = "Missing or weak credentials"),
        (status = 401, description = "Sign in as the invited account first"),
        (status = 404, description = "Unknown, used or revoked invitation"),
        (status = 409, description = "Username already taken"),
        (status = 410, description = "Invitation expired")
    )
)]
pub async fn accept_invitation(
    State(state): State<AppState>,
    ApiOptionalAuth(caller): ApiOptionalAuth,
    Path(token): Path<String>,
    Json(body): Json<AcceptInvitationRequest>,
) -> Result<(StatusCode, Json<AcceptInvitationResponse>), ApiError> {
    let preview = usable_invitation(&state, &token).await?;
    let tenant = active_tenant(&state, &preview).await?;

    // Validate and create the account before burning the single-use link...
    let joiner = resolve_joiner(&state, &preview, caller.as_ref(), &body).await?;
    // ...then claim it atomically so two concurrent accepts can never both succeed.
    let Some(invitation) = state
        .auth
        .invitations
        .claim(&hash_token(&token), joiner.user_id)
        .await?
    else {
        return Err(invalid());
    };
    if let Err(e) = grant(&state, &joiner, &invitation).await {
        state
            .auth
            .invitations
            .release(invitation.invitation_id)
            .await?;
        return Err(e);
    }
    drop_default_provisioning(&state, &joiner, tenant.tenant_id).await;
    tracing::info!(tenant_id = %tenant.tenant_id, user_id = %joiner.user_id, "invitation accepted");

    Ok((
        StatusCode::CREATED,
        Json(AcceptInvitationResponse {
            tenant_id: tenant.tenant_id.to_string(),
            tenant_name: tenant.name,
            role: invitation.role.to_string(),
            email: normalize_email(&invitation.email),
            username: joiner.username,
            created_account: joiner.created,
        }),
    ))
}
