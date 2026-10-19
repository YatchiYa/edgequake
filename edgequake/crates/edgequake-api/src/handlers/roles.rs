//! `GET /api/v1/roles` — the role & permission catalog the Members screen renders (SPEC-169).

use axum::{extract::State, Json};

use crate::error::ApiError;
use crate::handlers::auth::ApiAuthenticated;
use crate::services::role_catalog::{build_role_catalog, RoleCatalog};
use crate::state::AppState;

/// Organization roles, instance roles and the capability matrix derived from enforcement.
#[utoipa::path(
    get,
    path = "/api/v1/roles",
    tag = "Members",
    security(("bearer_auth" = [])),
    responses((status = 200, description = "Role catalog", body = RoleCatalog))
)]
pub async fn get_role_catalog(
    State(state): State<AppState>,
    _auth: ApiAuthenticated,
) -> Result<Json<RoleCatalog>, ApiError> {
    Ok(Json(build_role_catalog(&state.auth.rbac)))
}
