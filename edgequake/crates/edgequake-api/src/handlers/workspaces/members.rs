//! Tenant / workspace member management (memberships for password and SSO users).
//!
//! * Platform admins and tenant owners/admins ([`TenantAccess::Manage`]) manage members.
//! * Only owners and platform admins may grant, change or remove an `owner` membership.
//! * The last tenant-wide owner can be neither demoted nor removed (409).
//! * Removal is a hard delete: per-request membership checks and refresh / switch-scope
//!   revalidation stop the user on their very next call.

use std::collections::HashMap;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use edgequake_audit::{AuditEventType, AuditResult};
use edgequake_core::{Membership, MembershipRole};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use super::tenant_access::{require_tenant_access, TenantAccess};
use crate::error::ApiError;
use crate::handlers::auth::{ApiAuthenticated, RequestAuthContext};
use crate::state::AppState;

/// One membership of a tenant.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct MemberResponse {
    pub membership_id: Uuid,
    pub user_id: Uuid,
    /// None when the user record no longer exists.
    pub username: Option<String>,
    pub email: Option<String>,
    /// owner | admin | member | readonly
    pub role: String,
    /// None = every workspace of the tenant.
    pub workspace_id: Option<Uuid>,
    pub workspace_name: Option<String>,
    pub is_active: bool,
    pub joined_at: String,
}

/// Members of a tenant plus what the caller may do.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct MemberListResponse {
    pub items: Vec<MemberResponse>,
    /// Caller may grant / change / remove `owner` memberships.
    pub can_manage_owners: bool,
}

/// Add (or re-role) a member.
#[derive(Debug, Deserialize, ToSchema)]
pub struct AddMemberRequest {
    /// Username, email or user id.
    pub user: String,
    /// owner | admin | member | readonly
    pub role: String,
    /// None = all workspaces of the tenant.
    #[serde(default)]
    pub workspace_id: Option<Uuid>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateMemberRequest {
    /// owner | admin | member | readonly
    pub role: String,
}

fn internal(e: edgequake_core::Error) -> ApiError {
    ApiError::Internal(e.to_string())
}

fn parse_role(raw: &str) -> Result<MembershipRole, ApiError> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "owner" => Ok(MembershipRole::Owner),
        "admin" => Ok(MembershipRole::Admin),
        "member" => Ok(MembershipRole::Member),
        "readonly" => Ok(MembershipRole::Readonly),
        _ => Err(ApiError::BadRequest(
            "role must be owner, admin, member or readonly".into(),
        )),
    }
}

fn pg(state: &AppState) -> crate::state::PostgresRuntime {
    axum::extract::FromRef::from_ref(state)
}

async fn user_by_id(state: &AppState, id: &str) -> Result<Option<edgequake_auth::User>, ApiError> {
    crate::handlers::auth::get_user_by_id(
        &state.storage,
        Some(&pg(state)),
        &state.security,
        state.operational_stores.identity.as_deref(),
        id,
    )
    .await
}

/// Username, email or user id → active user.
async fn resolve_user(state: &AppState, login: &str) -> Result<edgequake_auth::User, ApiError> {
    let login = login.trim();
    let user = if Uuid::parse_str(login).is_ok() {
        user_by_id(state, login).await?
    } else {
        crate::handlers::auth::find_user_by_login(
            &state.storage,
            Some(&pg(state)),
            &state.security,
            login,
        )
        .await?
    };
    match user {
        Some(u) if u.is_active => Ok(u),
        Some(_) => Err(ApiError::BadRequest("User is inactive".into())),
        None => Err(ApiError::NotFound(format!("User '{login}' not found"))),
    }
}

/// Manage access to the tenant; returns whether the caller may touch `owner` memberships.
async fn authorize(
    state: &AppState,
    ctx: &RequestAuthContext,
    tenant_id: Uuid,
) -> Result<bool, ApiError> {
    state
        .workspace_service
        .get_tenant(tenant_id)
        .await
        .map_err(internal)?
        .ok_or_else(|| ApiError::NotFound(format!("Tenant {tenant_id} not found")))?;
    require_tenant_access(state, ctx, tenant_id, TenantAccess::Manage).await?;
    if ctx.role == edgequake_auth::Role::Admin {
        return Ok(true);
    }
    let user = Uuid::parse_str(&ctx.user_id)
        .map_err(|_| ApiError::forbidden_reason("tenant_access_denied"))?;
    Ok(state
        .workspace_service
        .get_user_role(user, tenant_id)
        .await
        .map_err(internal)?
        == Some(MembershipRole::Owner))
}

fn owner_guard(can_manage_owners: bool, roles: &[MembershipRole]) -> Result<(), ApiError> {
    if !can_manage_owners && roles.contains(&MembershipRole::Owner) {
        return Err(ApiError::forbidden_reason(
            "Only tenant owners or platform admins can manage owner memberships",
        ));
    }
    Ok(())
}

/// `m` is the only tenant-wide owner left (demoting / removing it would orphan the tenant).
fn is_last_owner(members: &[Membership], m: &Membership) -> bool {
    let tenant_owner = |x: &Membership| x.role == MembershipRole::Owner && x.workspace_id.is_none();
    tenant_owner(m)
        && !members
            .iter()
            .any(|x| tenant_owner(x) && x.user_id != m.user_id)
}

async fn tenant_members(state: &AppState, tenant_id: Uuid) -> Result<Vec<Membership>, ApiError> {
    state
        .workspace_service
        .get_tenant_memberships(tenant_id)
        .await
        .map_err(internal)
}

async fn find_member(
    state: &AppState,
    tenant_id: Uuid,
    membership_id: Uuid,
) -> Result<(Vec<Membership>, Membership), ApiError> {
    let members = tenant_members(state, tenant_id).await?;
    let m = members
        .iter()
        .find(|m| m.membership_id == membership_id)
        .cloned()
        .ok_or_else(|| ApiError::NotFound(format!("Membership {membership_id} not found")))?;
    Ok((members, m))
}

async fn to_response(
    state: &AppState,
    m: &Membership,
    names: &mut HashMap<Uuid, Option<String>>,
) -> Result<MemberResponse, ApiError> {
    let user = user_by_id(state, &m.user_id.to_string()).await?;
    let workspace_name = match m.workspace_id {
        None => None,
        Some(ws) => match names.get(&ws) {
            Some(name) => name.clone(),
            None => {
                let name = state
                    .workspace_service
                    .get_workspace(ws)
                    .await
                    .map_err(internal)?
                    .map(|w| w.name);
                names.insert(ws, name.clone());
                name
            }
        },
    };
    Ok(MemberResponse {
        membership_id: m.membership_id,
        user_id: m.user_id,
        username: user.as_ref().map(|u| u.username.clone()),
        email: user.map(|u| u.email),
        role: m.role.to_string(),
        workspace_id: m.workspace_id,
        workspace_name,
        is_active: m.is_active,
        joined_at: m.joined_at.to_rfc3339(),
    })
}

fn audit(state: &AppState, ctx: &RequestAuthContext, m: &Membership, action: &str) {
    crate::services::record_compliance_event(
        state,
        m.tenant_id.to_string(),
        AuditEventType::TenantAccess,
        action,
        AuditResult::Success,
        m.workspace_id.map(|w| w.to_string()),
        Some(ctx.user_id.clone()),
        Some(("membership".into(), m.membership_id.to_string())),
    );
    tracing::info!(
        audit.event = action,
        tenant_id = %m.tenant_id,
        membership_id = %m.membership_id,
        member_user_id = %m.user_id,
        role = %m.role,
        actor = %ctx.user_id,
        "Tenant membership changed"
    );
}

/// List the members of a tenant.
///
/// GET /api/v1/tenants/{tenant_id}/members
#[utoipa::path(
    get,
    path = "/api/v1/tenants/{tenant_id}/members",
    params(("tenant_id" = Uuid, Path, description = "Tenant ID")),
    responses(
        (status = 200, description = "Tenant members", body = MemberListResponse),
        (status = 403, description = "Requires platform admin or tenant owner/admin"),
        (status = 404, description = "Tenant not found"),
    ),
    tags = ["tenants"]
)]
pub async fn list_tenant_members(
    State(state): State<AppState>,
    auth: ApiAuthenticated,
    Path(tenant_id): Path<Uuid>,
) -> Result<Json<MemberListResponse>, ApiError> {
    let can_manage_owners = authorize(&state, auth.context(), tenant_id).await?;
    let mut members = tenant_members(&state, tenant_id).await?;
    members.sort_by_key(|m| m.joined_at);
    let mut names = HashMap::new();
    let mut items = Vec::with_capacity(members.len());
    // ponytail: one user lookup per member; batch it if tenants grow to thousands of members.
    for m in &members {
        items.push(to_response(&state, m, &mut names).await?);
    }
    Ok(Json(MemberListResponse {
        items,
        can_manage_owners,
    }))
}

/// Add a member to a tenant (or one of its workspaces); re-roles an existing grant.
///
/// POST /api/v1/tenants/{tenant_id}/members
#[utoipa::path(
    post,
    path = "/api/v1/tenants/{tenant_id}/members",
    params(("tenant_id" = Uuid, Path, description = "Tenant ID")),
    request_body = AddMemberRequest,
    responses(
        (status = 200, description = "Existing membership updated", body = MemberResponse),
        (status = 201, description = "Membership created", body = MemberResponse),
        (status = 400, description = "Invalid role, workspace or inactive user"),
        (status = 403, description = "Not allowed (owner grants need an owner or platform admin)"),
        (status = 404, description = "Tenant or user not found"),
        (status = 409, description = "Would demote the last owner"),
    ),
    tags = ["tenants"]
)]
pub async fn add_tenant_member(
    State(state): State<AppState>,
    auth: ApiAuthenticated,
    Path(tenant_id): Path<Uuid>,
    Json(request): Json<AddMemberRequest>,
) -> Result<(StatusCode, Json<MemberResponse>), ApiError> {
    let ctx = auth.context();
    let can_manage_owners = authorize(&state, ctx, tenant_id).await?;
    let role = parse_role(&request.role)?;
    owner_guard(can_manage_owners, &[role])?;
    if let Some(ws) = request.workspace_id {
        let valid = state
            .workspace_service
            .get_workspace(ws)
            .await
            .map_err(internal)?
            .is_some_and(|w| w.is_active && w.tenant_id == tenant_id);
        if !valid {
            return Err(ApiError::BadRequest(
                "workspace_id is not an active workspace of this tenant".into(),
            ));
        }
    }
    let user = resolve_user(&state, &request.user).await?;
    let user_id = Uuid::parse_str(&user.user_id)
        .map_err(|_| ApiError::BadRequest("User id is not a UUID".into()))?;

    let members = tenant_members(&state, tenant_id).await?;
    let mut names = HashMap::new();
    if let Some(existing) = members
        .iter()
        .find(|m| m.user_id == user_id && m.workspace_id == request.workspace_id)
    {
        owner_guard(can_manage_owners, &[existing.role])?;
        if role != MembershipRole::Owner && is_last_owner(&members, existing) {
            return Err(ApiError::Conflict(
                "Cannot demote the last tenant owner".into(),
            ));
        }
        let updated = state
            .workspace_service
            .update_membership_role(existing.membership_id, role)
            .await
            .map_err(internal)?;
        audit(&state, ctx, &updated, "member_role_updated");
        return Ok((
            StatusCode::OK,
            Json(to_response(&state, &updated, &mut names).await?),
        ));
    }

    let mut membership = Membership::new(user_id, tenant_id, role);
    if let Some(ws) = request.workspace_id {
        membership = membership.for_workspace(ws);
    }
    membership
        .metadata
        .insert("source".into(), serde_json::json!("manual"));
    membership
        .metadata
        .insert("granted_by".into(), serde_json::json!(ctx.user_id));
    let created = state
        .workspace_service
        .add_membership(membership)
        .await
        .map_err(internal)?;
    audit(&state, ctx, &created, "member_added");
    Ok((
        StatusCode::CREATED,
        Json(to_response(&state, &created, &mut names).await?),
    ))
}

/// Change a member's role.
///
/// PATCH /api/v1/tenants/{tenant_id}/members/{membership_id}
#[utoipa::path(
    patch,
    path = "/api/v1/tenants/{tenant_id}/members/{membership_id}",
    params(
        ("tenant_id" = Uuid, Path, description = "Tenant ID"),
        ("membership_id" = Uuid, Path, description = "Membership ID"),
    ),
    request_body = UpdateMemberRequest,
    responses(
        (status = 200, description = "Membership updated", body = MemberResponse),
        (status = 400, description = "Invalid role"),
        (status = 403, description = "Not allowed"),
        (status = 404, description = "Membership not found"),
        (status = 409, description = "Would demote the last owner"),
    ),
    tags = ["tenants"]
)]
pub async fn update_tenant_member(
    State(state): State<AppState>,
    auth: ApiAuthenticated,
    Path((tenant_id, membership_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<UpdateMemberRequest>,
) -> Result<Json<MemberResponse>, ApiError> {
    let ctx = auth.context();
    let can_manage_owners = authorize(&state, ctx, tenant_id).await?;
    let role = parse_role(&request.role)?;
    let (members, current) = find_member(&state, tenant_id, membership_id).await?;
    owner_guard(can_manage_owners, &[role, current.role])?;
    if role != MembershipRole::Owner && is_last_owner(&members, &current) {
        return Err(ApiError::Conflict(
            "Cannot demote the last tenant owner".into(),
        ));
    }
    let updated = state
        .workspace_service
        .update_membership_role(membership_id, role)
        .await
        .map_err(internal)?;
    audit(&state, ctx, &updated, "member_role_updated");
    Ok(Json(
        to_response(&state, &updated, &mut HashMap::new()).await?,
    ))
}

/// Remove a member (hard delete; effective on the member's next request).
///
/// DELETE /api/v1/tenants/{tenant_id}/members/{membership_id}
#[utoipa::path(
    delete,
    path = "/api/v1/tenants/{tenant_id}/members/{membership_id}",
    params(
        ("tenant_id" = Uuid, Path, description = "Tenant ID"),
        ("membership_id" = Uuid, Path, description = "Membership ID"),
    ),
    responses(
        (status = 204, description = "Membership removed"),
        (status = 403, description = "Not allowed"),
        (status = 404, description = "Membership not found"),
        (status = 409, description = "Cannot remove the last owner"),
    ),
    tags = ["tenants"]
)]
pub async fn remove_tenant_member(
    State(state): State<AppState>,
    auth: ApiAuthenticated,
    Path((tenant_id, membership_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    let ctx = auth.context();
    let can_manage_owners = authorize(&state, ctx, tenant_id).await?;
    let (members, current) = find_member(&state, tenant_id, membership_id).await?;
    owner_guard(can_manage_owners, &[current.role])?;
    if is_last_owner(&members, &current) {
        return Err(ApiError::Conflict(
            "Cannot remove the last tenant owner".into(),
        ));
    }
    state
        .workspace_service
        .remove_membership(membership_id)
        .await
        .map_err(internal)?;
    audit(&state, ctx, &current, "member_removed");
    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn last_owner_counts_only_other_tenant_wide_owners() {
        let t = Uuid::new_v4();
        let alice = Membership::new(Uuid::new_v4(), t, MembershipRole::Owner);
        let ws_owner =
            Membership::new(Uuid::new_v4(), t, MembershipRole::Owner).for_workspace(Uuid::new_v4());
        let admin = Membership::new(Uuid::new_v4(), t, MembershipRole::Admin);
        let mut members = vec![alice.clone(), ws_owner.clone(), admin.clone()];
        assert!(is_last_owner(&members, &alice));
        assert!(!is_last_owner(&members, &ws_owner));
        assert!(!is_last_owner(&members, &admin));
        members.push(Membership::new(Uuid::new_v4(), t, MembershipRole::Owner));
        assert!(!is_last_owner(&members, &alice));
        assert!(parse_role("Owner").is_ok() && parse_role("root").is_err());
        assert!(owner_guard(false, &[MembershipRole::Owner]).is_err());
        assert!(owner_guard(false, &[MembershipRole::Admin]).is_ok());
    }
}
