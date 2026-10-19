//! Public, token-gated redeem of a one-time access link (SPEC-169).
//!
//! Preview is a harmless GET; redeem is a POST so mail scanners and link prefetchers can never
//! consume the link. Both outcomes end with the person signed in.

use axum::{
    extract::{Path, State},
    http::HeaderMap,
    response::Response,
    Json,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::session::{revoke_credentials, start_session};
use crate::error::ApiError;
use crate::handlers::auth::UserRecord;
use crate::handlers::auth_types::LoginResponse;
use crate::handlers::members::directory::{save_user_record, user_record};
use crate::services::access_links::{AccessLink, AccessPurpose};
use crate::services::invitations::token::hash_token;
use crate::state::AppState;

const MAX_TOKEN_LEN: usize = 256;

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct AccessLinkPreview {
    pub purpose: String,
    pub username: String,
    pub email: String,
    pub tenant_name: Option<String>,
    pub issued_by: Option<String>,
    pub expires_at: String,
}

#[derive(Debug, Clone, Default, Deserialize, ToSchema)]
pub struct RedeemAccessLinkRequest {
    /// Required for `password_reset`; ignored for `magic_login`.
    pub password: Option<String>,
}

fn invalid() -> ApiError {
    ApiError::NotFound("access_link_invalid".into())
}

/// Open, unexpired link for a token (404 unknown/used/revoked, 410 expired).
async fn usable_link(state: &AppState, token: &str) -> Result<AccessLink, ApiError> {
    if token.is_empty() || token.len() > MAX_TOKEN_LEN {
        return Err(invalid());
    }
    let link = state
        .auth
        .access_links
        .find_open_by_token(&hash_token(token))
        .await?
        .ok_or_else(invalid)?;
    if !link.is_usable(Utc::now()) {
        return Err(ApiError::Gone("access_link_expired".into()));
    }
    Ok(link)
}

/// The account behind a link; suspended or vanished accounts read as an invalid link.
async fn live_account(state: &AppState, link: &AccessLink) -> Result<UserRecord, ApiError> {
    user_record(state, &link.user_id.to_string())
        .await?
        .filter(|r| r.is_active)
        .ok_or_else(invalid)
}

/// Public preview of an access link.
#[utoipa::path(
    get,
    path = "/api/v1/auth/access-links/{token}",
    tag = "Authentication",
    params(("token" = String, Path, description = "Single-use access token")),
    responses(
        (status = 200, description = "Link details", body = AccessLinkPreview),
        (status = 404, description = "Unknown, used or revoked link"),
        (status = 410, description = "Link expired")
    )
)]
pub async fn preview_access_link(
    State(state): State<AppState>,
    Path(token): Path<String>,
) -> Result<Json<AccessLinkPreview>, ApiError> {
    let link = usable_link(&state, &token).await?;
    let record = live_account(&state, &link).await?;
    let tenant_name = state
        .workspace_service
        .get_tenant(link.tenant_id)
        .await
        .ok()
        .flatten()
        .map(|t| t.name);
    Ok(Json(AccessLinkPreview {
        purpose: link.purpose.as_str().to_string(),
        username: record.username,
        email: record.email,
        tenant_name,
        issued_by: link.created_by_name,
        expires_at: link.expires_at.to_rfc3339(),
    }))
}

/// Hash a chosen password up front, so a weak one never burns the single-use link.
fn hashed_password(
    state: &AppState,
    link: &AccessLink,
    body: &RedeemAccessLinkRequest,
) -> Result<Option<String>, ApiError> {
    if link.purpose != AccessPurpose::PasswordReset {
        return Ok(None);
    }
    let password = body
        .password
        .as_deref()
        .filter(|p| !p.is_empty())
        .ok_or_else(|| ApiError::BadRequest("A new password is required".into()))?;
    state
        .auth
        .password
        .hash_password(password)
        .map(Some)
        .map_err(|e| ApiError::BadRequest(format!("Password error: {e}")))
}

/// Redeem an access link: sign in (and, for a reset, set the new password).
#[utoipa::path(
    post,
    path = "/api/v1/auth/access-links/{token}/redeem",
    tag = "Authentication",
    params(("token" = String, Path, description = "Single-use access token")),
    request_body = RedeemAccessLinkRequest,
    responses(
        (status = 200, description = "Signed in", body = LoginResponse),
        (status = 400, description = "Missing or weak password"),
        (status = 404, description = "Unknown, used or revoked link"),
        (status = 410, description = "Link expired")
    )
)]
pub async fn redeem_access_link(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(token): Path<String>,
    Json(body): Json<RedeemAccessLinkRequest>,
) -> Result<Response, ApiError> {
    let preview = usable_link(&state, &token).await?;
    let mut record = live_account(&state, &preview).await?;
    let new_hash = hashed_password(&state, &preview, &body)?;

    let link = state
        .auth
        .access_links
        .claim(&hash_token(&token))
        .await?
        .ok_or_else(invalid)?;

    match finish(&state, &headers, &link, &mut record, new_hash).await {
        Ok(response) => {
            tracing::info!(user_id = %link.user_id, purpose = link.purpose.as_str(), "access link redeemed");
            Ok(response)
        }
        Err(e) => {
            state.auth.access_links.release(link.link_id).await?;
            Err(e)
        }
    }
}

/// Apply the credential change (reset only) and open the session.
async fn finish(
    state: &AppState,
    headers: &HeaderMap,
    link: &AccessLink,
    record: &mut UserRecord,
    new_hash: Option<String>,
) -> Result<Response, ApiError> {
    if let Some(hash) = new_hash {
        record.password_hash = hash;
        record.updated_at = Utc::now();
        save_user_record(state, record).await?;
        // A reset means "I may have lost control": end every other session and open link.
        revoke_credentials(state, link.user_id).await?;
    }
    start_session(state, headers, record).await
}
