//! Bridge from an access-link redeem to the regular login machinery.

use axum::{extract::FromRef, http::HeaderMap, response::Response};

use crate::error::ApiError;
use crate::handlers::auth::{open_session, UserRecord};
use crate::state::{
    ApiSecurityConfig, AppState, AuthRuntime, OperationalStores, PostgresRuntime, StorageRuntime,
};

/// Sign-in bookkeeping (clears lockout counters, stamps last login) then mint the session.
pub async fn start_session(
    state: &AppState,
    headers: &HeaderMap,
    record: &mut UserRecord,
) -> Result<Response, ApiError> {
    let auth = AuthRuntime::from_ref(state);
    let storage = StorageRuntime::from_ref(state);
    let pg = PostgresRuntime::from_ref(state);
    let security = ApiSecurityConfig::from_ref(state);
    let stores = OperationalStores::from_ref(state);
    crate::services::login_lockout::record_successful_login(
        &storage,
        Some(&pg),
        &security,
        stores.identity.as_deref(),
        record,
    )
    .await?;
    open_session(&auth, &storage, &pg, &security, &stores, headers, record).await
}

/// Sign the person out of every other device: refresh tokens and open links.
pub async fn revoke_credentials(state: &AppState, user_id: uuid::Uuid) -> Result<(), ApiError> {
    let storage = StorageRuntime::from_ref(state);
    let pg = PostgresRuntime::from_ref(state);
    let security = ApiSecurityConfig::from_ref(state);
    crate::services::session_storage::revoke_user_refresh_tokens(
        &storage,
        Some(&pg),
        &security,
        user_id,
    )
    .await?;
    state
        .auth
        .access_links
        .revoke_for_user(user_id, None)
        .await?;
    Ok(())
}
