//! `POST /api/v1/auth/password` — change your own password (SPEC-169).

use axum::{extract::State, http::HeaderMap, response::Response, Json};
use chrono::Utc;
use serde::Deserialize;
use utoipa::ToSchema;

use super::session::{revoke_credentials, start_session};
use crate::error::ApiError;
use crate::handlers::auth::ApiAuthenticated;
use crate::handlers::auth_types::LoginResponse;
use crate::handlers::members::directory::{save_user_record, user_record};
use crate::state::AppState;

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

/// Change the signed-in user's password; every other session is signed out.
///
/// Returns a fresh session so the device that made the change stays signed in.
#[utoipa::path(
    post,
    path = "/api/v1/auth/password",
    tag = "Authentication",
    security(("bearer_auth" = [])),
    request_body = ChangePasswordRequest,
    responses(
        (status = 200, description = "Password changed; new session", body = LoginResponse),
        (status = 400, description = "Wrong current password, or weak / unchanged new password"),
        (status = 409, description = "Account signs in through single sign-on"),
        (status = 423, description = "Account locked")
    )
)]
pub async fn change_password(
    State(state): State<AppState>,
    auth: ApiAuthenticated,
    headers: HeaderMap,
    Json(body): Json<ChangePasswordRequest>,
) -> Result<Response, ApiError> {
    let ctx = auth.context();
    let mut record = user_record(&state, &ctx.user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("User not found".into()))?;
    crate::services::login_lockout::ensure_login_allowed(&record)?;
    if let Ok(id) = uuid::Uuid::parse_str(&record.user_id) {
        if !state
            .auth
            .federation
            .identities_for_users(&[id])
            .await?
            .is_empty()
        {
            return Err(ApiError::Conflict(
                "You sign in through single sign-on; your identity provider manages your password."
                    .into(),
            ));
        }
    }

    let current_ok = state
        .auth
        .password
        .verify_password(&body.current_password, &record.password_hash)
        .map_err(|e| ApiError::Internal(format!("password_verify failed: {e}")))?;
    if !current_ok {
        return Err(wrong_current(&state, &mut record).await);
    }
    if body.current_password == body.new_password {
        return Err(ApiError::BadRequest(
            "The new password must differ from the current one".into(),
        ));
    }
    record.password_hash = state
        .auth
        .password
        .hash_password(&body.new_password)
        .map_err(|e| ApiError::BadRequest(format!("Password error: {e}")))?;
    record.updated_at = Utc::now();
    save_user_record(&state, &record).await?;

    let user_id = uuid::Uuid::parse_str(&record.user_id)
        .map_err(|_| ApiError::Internal("invalid user id".into()))?;
    revoke_credentials(&state, user_id).await?;
    start_session(&state, &headers, &mut record).await
}

/// Count the miss (a stolen session must not be a free password oracle), then say no.
async fn wrong_current(
    state: &AppState,
    record: &mut crate::handlers::auth::UserRecord,
) -> ApiError {
    use axum::extract::FromRef;
    let storage = crate::state::StorageRuntime::from_ref(state);
    let pg = crate::state::PostgresRuntime::from_ref(state);
    let security = crate::state::ApiSecurityConfig::from_ref(state);
    let stores = crate::state::OperationalStores::from_ref(state);
    match crate::services::login_lockout::record_failed_login(
        &storage,
        Some(&pg),
        &security,
        stores.identity.as_deref(),
        &state.auth.config,
        record,
    )
    .await
    {
        Err(locked) => locked,
        Ok(()) => ApiError::BadRequest("current_password_incorrect".into()),
    }
}
