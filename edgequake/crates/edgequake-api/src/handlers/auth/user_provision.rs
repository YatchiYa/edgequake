//! Password-account provisioning shared by `POST /users` and invitation accept (SPEC-169, DRY).

use axum::extract::FromRef;
use tracing::info;
use uuid::Uuid;

use super::{persist_user_record, UserRecord};
use crate::error::ApiError;
use crate::state::{
    ApiSecurityConfig, AppState, AuthRuntime, OperationalStores, PostgresRuntime, StorageRuntime,
};
use edgequake_auth::{Role, User};

/// The identity-facing runtimes, narrower than `AppState` (ISP). `POST /users` builds it from
/// its own extractors; other callers use [`IdentityCtx::from_state`].
pub(crate) struct IdentityCtx {
    pub auth: AuthRuntime,
    pub storage: StorageRuntime,
    pub pg: PostgresRuntime,
    pub security: ApiSecurityConfig,
    pub stores: OperationalStores,
}

impl IdentityCtx {
    pub fn from_state(state: &AppState) -> Self {
        Self {
            auth: AuthRuntime::from_ref(state),
            storage: StorageRuntime::from_ref(state),
            pg: PostgresRuntime::from_ref(state),
            security: ApiSecurityConfig::from_ref(state),
            stores: OperationalStores::from_ref(state),
        }
    }
}

/// `Conflict` when the username or email is already registered (PostgreSQL identity SSOT).
pub(crate) async fn assert_login_available(
    ctx: &IdentityCtx,
    username: &str,
    email: &str,
) -> Result<(), ApiError> {
    #[cfg(feature = "postgres")]
    {
        for (login, message) in [
            (username, "Username already exists"),
            (email, "Email already exists"),
        ] {
            if crate::services::identity_storage::find_user_record_by_login(
                &ctx.storage,
                Some(&ctx.pg),
                &ctx.security,
                login,
            )
            .await?
            .is_some()
            {
                return Err(ApiError::Conflict(message.to_string()));
            }
        }
    }
    #[cfg(not(feature = "postgres"))]
    let _ = (ctx, username, email);
    Ok(())
}

/// Existing account for a username or email.
pub(crate) async fn find_user_by_login(
    ctx: &IdentityCtx,
    login: &str,
) -> Result<Option<UserRecord>, ApiError> {
    #[cfg(feature = "postgres")]
    {
        crate::services::identity_storage::find_user_record_by_login(
            &ctx.storage,
            Some(&ctx.pg),
            &ctx.security,
            login,
        )
        .await
    }
    #[cfg(not(feature = "postgres"))]
    {
        let _ = (ctx, login);
        Ok(None)
    }
}

/// Hash the password, create the account with `role` and persist it.
pub(crate) async fn provision_password_user(
    ctx: &IdentityCtx,
    username: &str,
    email: &str,
    password: &str,
    role: Role,
) -> Result<User, ApiError> {
    let password_hash = ctx
        .auth
        .password
        .hash_password(password)
        .map_err(|e| ApiError::BadRequest(format!("Password error: {}", e)))?;

    let user = User::new(
        Uuid::new_v4().to_string(),
        username,
        email,
        password_hash,
        role,
    );
    persist_user_record(
        &ctx.storage,
        Some(&ctx.pg),
        &ctx.security,
        ctx.stores.identity.as_deref(),
        &UserRecord::from(&user),
    )
    .await?;

    info!("User created: {} ({})", user.username, user.user_id);
    Ok(user)
}
