//! `PATCH` / `DELETE /api/v1/tenants/{tenant_id}/members/{user_id}` (SPEC-169).
//!
//! Organization role and removal are decided by the pure rules in [`super::guards`];
//! account-level changes (suspend, instance role) are platform-admin only.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use edgequake_auth::Role;
use edgequake_core::{Membership, MembershipRole};
use uuid::Uuid;

use super::directory::{
    actor_for, grants_of, last_owner, load_people, parse_user_id, require_tenant, save_user_record,
    tenant_memberships, user_record, user_records,
};
use super::guards::{check_assign, check_remove, Actor};
use super::types::{MemberView, UpdateMemberRequest};
use super::view::primary_membership;
use crate::error::ApiError;
use crate::handlers::auth::{ApiAuthenticated, RequestAuthContext};
use crate::handlers::workspaces::tenant_access::{require_tenant_access, TenantAccess};
use crate::state::AppState;

fn internal(e: impl std::fmt::Display) -> ApiError {
    ApiError::Internal(e.to_string())
}

/// The target's grants and whether they hold the only Owner grant.
struct Target {
    id: Uuid,
    grants: Vec<Membership>,
    is_last_owner: bool,
}

impl Target {
    fn primary(&self) -> Result<&Membership, ApiError> {
        let refs: Vec<&Membership> = self.grants.iter().collect();
        primary_membership(&refs)
            .ok_or_else(|| ApiError::NotFound("Member not found in this organization".into()))
    }
}

async fn load_target(state: &AppState, tenant_id: Uuid, raw_id: &str) -> Result<Target, ApiError> {
    let id = parse_user_id(raw_id)?;
    let memberships = tenant_memberships(state, tenant_id).await?;
    let grants: Vec<Membership> = grants_of(&memberships, id).into_iter().cloned().collect();
    if grants.is_empty() {
        return Err(ApiError::NotFound(
            "Member not found in this organization".into(),
        ));
    }
    Ok(Target {
        id,
        grants,
        is_last_owner: last_owner(&memberships) == Some(id),
    })
}

async fn change_role(
    state: &AppState,
    actor: &Actor,
    target: &Target,
    role: &str,
) -> Result<(), ApiError> {
    let new_role: MembershipRole = role.parse().map_err(ApiError::BadRequest)?;
    let current = target.primary()?;
    check_assign(actor, current.role, new_role, target.is_last_owner)?;
    if current.role != new_role {
        state
            .workspace_service
            .assign_membership_role(current.membership_id, new_role)
            .await
            .map_err(internal)?;
    }
    Ok(())
}

/// Admins to keep: the platform must never lose its last administrator.
async fn other_admins(state: &AppState, excluding: Uuid) -> Result<usize, ApiError> {
    Ok(user_records(state)
        .await?
        .values()
        .filter(|r| {
            r.user_id != excluding.to_string() && Role::parse(&r.role) == Role::Admin && r.is_active
        })
        .count())
}

async fn change_account(
    state: &AppState,
    ctx: &RequestAuthContext,
    actor: &Actor,
    target: &Target,
    body: &UpdateMemberRequest,
) -> Result<(), ApiError> {
    if body.is_active.is_none() && body.instance_role.is_none() {
        return Ok(());
    }
    if !actor.instance_admin {
        return Err(ApiError::forbidden_reason("instance_admin_required"));
    }
    let mut record = user_record(state, &target.id.to_string())
        .await?
        .ok_or_else(|| ApiError::NotFound("User not found".into()))?;
    let is_self = ctx.user_id == record.user_id;
    let is_admin = Role::parse(&record.role) == Role::Admin;

    if let Some(active) = body.is_active {
        if is_self && !active {
            return Err(ApiError::Conflict(
                "You cannot suspend your own account.".into(),
            ));
        }
        if !active && is_admin && other_admins(state, target.id).await? == 0 {
            return Err(ApiError::Conflict(
                "Cannot suspend the last administrator.".into(),
            ));
        }
        record.is_active = active;
    }
    if let Some(raw) = body.instance_role.as_deref() {
        let new_role = Role::from_api_input(raw).map_err(ApiError::BadRequest)?;
        if is_admin && new_role != Role::Admin && other_admins(state, target.id).await? == 0 {
            return Err(ApiError::Conflict(
                "Cannot demote the last administrator.".into(),
            ));
        }
        record.role = new_role.to_string();
    }
    record.updated_at = Utc::now();
    save_user_record(state, &record).await
}

async fn view_of(
    state: &AppState,
    tenant_id: Uuid,
    viewer: &str,
    id: Uuid,
) -> Result<MemberView, ApiError> {
    load_people(state, tenant_id, viewer)
        .await?
        .into_iter()
        .find(|m| m.id == id.to_string())
        .ok_or_else(|| ApiError::NotFound("Member not found in this organization".into()))
}

/// Change a member's organization role, or (platform admins) suspend / change instance role.
#[utoipa::path(
    patch,
    path = "/api/v1/tenants/{tenant_id}/members/{user_id}",
    tag = "Members",
    security(("bearer_auth" = [])),
    params(
        ("tenant_id" = Uuid, Path, description = "Organization id"),
        ("user_id" = String, Path, description = "User id")
    ),
    request_body = UpdateMemberRequest,
    responses(
        (status = 200, description = "Updated member", body = MemberView),
        (status = 400, description = "Unknown role"),
        (status = 403, description = "Not allowed to change this member or grant this role"),
        (status = 404, description = "Member not found"),
        (status = 409, description = "Would orphan the organization or platform")
    )
)]
pub async fn update_member(
    State(state): State<AppState>,
    auth: ApiAuthenticated,
    Path((tenant_id, user_id)): Path<(Uuid, String)>,
    Json(body): Json<UpdateMemberRequest>,
) -> Result<Json<MemberView>, ApiError> {
    let ctx = auth.context();
    require_tenant_access(&state, ctx, tenant_id, TenantAccess::Manage).await?;
    require_tenant(&state, tenant_id).await?;
    let actor = actor_for(&state, ctx, tenant_id).await?;
    let target = load_target(&state, tenant_id, &user_id).await?;

    // Validate everything before mutating anything.
    if let Some(role) = body.role.as_deref() {
        let new_role: MembershipRole = role.parse().map_err(ApiError::BadRequest)?;
        check_assign(
            &actor,
            target.primary()?.role,
            new_role,
            target.is_last_owner,
        )?;
    }
    if let Some(raw) = body.instance_role.as_deref() {
        Role::from_api_input(raw).map_err(ApiError::BadRequest)?;
    }

    // Account change first: it carries the more restrictive guards.
    change_account(&state, ctx, &actor, &target, &body).await?;
    if let Some(role) = body.role.as_deref() {
        change_role(&state, &actor, &target, role).await?;
    }
    Ok(Json(
        view_of(&state, tenant_id, &ctx.user_id, target.id).await?,
    ))
}

/// Remove a person from the organization (their account is kept).
#[utoipa::path(
    delete,
    path = "/api/v1/tenants/{tenant_id}/members/{user_id}",
    tag = "Members",
    security(("bearer_auth" = [])),
    params(
        ("tenant_id" = Uuid, Path, description = "Organization id"),
        ("user_id" = String, Path, description = "User id")
    ),
    responses(
        (status = 204, description = "Member removed"),
        (status = 403, description = "Member outranks you"),
        (status = 404, description = "Member not found"),
        (status = 409, description = "Cannot remove the last owner")
    )
)]
pub async fn remove_member(
    State(state): State<AppState>,
    auth: ApiAuthenticated,
    Path((tenant_id, user_id)): Path<(Uuid, String)>,
) -> Result<StatusCode, ApiError> {
    let ctx = auth.context();
    require_tenant_access(&state, ctx, tenant_id, TenantAccess::Manage).await?;
    require_tenant(&state, tenant_id).await?;
    let actor = actor_for(&state, ctx, tenant_id).await?;
    let target = load_target(&state, tenant_id, &user_id).await?;
    let is_self = ctx.user_id == target.id.to_string();
    check_remove(
        &actor,
        target.primary()?.role,
        target.is_last_owner,
        is_self,
    )?;

    for grant in &target.grants {
        state
            .workspace_service
            .deactivate_membership(grant.membership_id)
            .await
            .map_err(internal)?;
    }
    Ok(StatusCode::NO_CONTENT)
}
