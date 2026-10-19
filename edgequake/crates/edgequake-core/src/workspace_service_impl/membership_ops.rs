#[cfg(feature = "postgres")]
use uuid::Uuid;

#[cfg(feature = "postgres")]
use crate::{
    error::{Error, Result},
    types::{Membership, MembershipRole, TenantContext},
};

#[cfg(feature = "postgres")]
use super::rows::{MembershipRow, MEMBERSHIP_COLUMNS};
#[cfg(feature = "postgres")]
use super::WorkspaceServiceImpl;

#[cfg(feature = "postgres")]
impl WorkspaceServiceImpl {
    // ============ Membership Operations ============

    /// Insert, or revive a deactivated tombstone for the same (user, tenant, workspace).
    /// An already-active grant is a validation error (never silently overwritten).
    pub(super) async fn pg_add_membership(&self, membership: Membership) -> Result<Membership> {
        let row: Option<MembershipRow> = sqlx::query_as(&format!(
            r#"
            INSERT INTO memberships (membership_id, tenant_id, workspace_id, user_id, role, is_active, joined_at, metadata)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (user_id, tenant_id, workspace_id) DO UPDATE SET
                role = EXCLUDED.role, is_active = TRUE,
                joined_at = EXCLUDED.joined_at, metadata = EXCLUDED.metadata
            WHERE memberships.is_active = FALSE
            RETURNING {MEMBERSHIP_COLUMNS}
            "#
        ))
        .bind(membership.membership_id)
        .bind(membership.tenant_id)
        .bind(membership.workspace_id)
        .bind(membership.user_id)
        .bind(membership.role.to_string())
        .bind(membership.is_active)
        .bind(membership.joined_at)
        .bind(sqlx::types::Json(&membership.metadata))
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| Error::internal(format!("Failed to add membership: {}", e)))?;

        row.map(|r| r.into_membership())
            .ok_or_else(|| Error::validation("Membership already exists"))
    }

    pub(super) async fn pg_get_user_memberships(&self, user_id: Uuid) -> Result<Vec<Membership>> {
        let rows: Vec<MembershipRow> = sqlx::query_as(&format!(
            "SELECT {MEMBERSHIP_COLUMNS} FROM memberships WHERE user_id = $1 AND is_active = TRUE"
        ))
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::internal(format!("Failed to get user memberships: {}", e)))?;

        Ok(rows.into_iter().map(|r| r.into_membership()).collect())
    }

    pub(super) async fn pg_get_tenant_memberships(
        &self,
        tenant_id: Uuid,
    ) -> Result<Vec<Membership>> {
        let rows: Vec<MembershipRow> = sqlx::query_as(&format!(
            "SELECT {MEMBERSHIP_COLUMNS} FROM memberships WHERE tenant_id = $1 AND is_active = TRUE"
        ))
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::internal(format!("Failed to get tenant memberships: {}", e)))?;

        Ok(rows.into_iter().map(|r| r.into_membership()).collect())
    }

    pub(super) async fn pg_update_membership_role(
        &self,
        membership_id: Uuid,
        role: MembershipRole,
    ) -> Result<Membership> {
        let result = sqlx::query("UPDATE memberships SET role = $2 WHERE membership_id = $1")
            .bind(membership_id)
            .bind(role.to_string())
            .execute(&self.pool)
            .await
            .map_err(|e| Error::internal(format!("Failed to update membership: {}", e)))?;

        if result.rows_affected() == 0 {
            return Err(Error::not_found(format!(
                "Membership {} not found",
                membership_id
            )));
        }

        // Fetch and return updated membership
        let row: MembershipRow = sqlx::query_as(&format!(
            "SELECT {MEMBERSHIP_COLUMNS} FROM memberships WHERE membership_id = $1"
        ))
        .bind(membership_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| Error::internal(format!("Failed to fetch updated membership: {}", e)))?;

        Ok(row.into_membership())
    }

    /// Administrator-chosen role: also detaches an SSO-managed row from IdP role sync.
    pub(super) async fn pg_assign_membership_role(
        &self,
        membership_id: Uuid,
        role: MembershipRole,
    ) -> Result<Membership> {
        let row: Option<MembershipRow> = sqlx::query_as(&format!(
            "UPDATE memberships SET role = $2, metadata = CASE \
               WHEN metadata->>'source' = 'sso' \
               THEN COALESCE(metadata, '{{}}'::jsonb) \
                    || jsonb_build_object('source', 'manual', 'previous_source', 'sso') \
               ELSE metadata END \
             WHERE membership_id = $1 RETURNING {MEMBERSHIP_COLUMNS}"
        ))
        .bind(membership_id)
        .bind(role.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| Error::internal(format!("Failed to assign membership role: {}", e)))?;
        row.map(|r| r.into_membership())
            .ok_or_else(|| Error::not_found(format!("Membership {} not found", membership_id)))
    }

    pub(super) async fn pg_remove_membership(&self, membership_id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM memberships WHERE membership_id = $1")
            .bind(membership_id)
            .execute(&self.pool)
            .await
            .map_err(|e| Error::internal(format!("Failed to remove membership: {}", e)))?;

        Ok(())
    }

    pub(super) async fn pg_deactivate_membership(&self, membership_id: Uuid) -> Result<()> {
        let result =
            sqlx::query("UPDATE memberships SET is_active = FALSE WHERE membership_id = $1")
                .bind(membership_id)
                .execute(&self.pool)
                .await
                .map_err(|e| Error::internal(format!("Failed to deactivate membership: {}", e)))?;
        if result.rows_affected() == 0 {
            return Err(Error::not_found(format!(
                "Membership {} not found",
                membership_id
            )));
        }
        Ok(())
    }

    pub(super) async fn pg_check_tenant_access(
        &self,
        user_id: Uuid,
        tenant_id: Uuid,
    ) -> Result<bool> {
        let exists: Option<(i32,)> = sqlx::query_as(
            "SELECT 1 FROM memberships m JOIN tenants t USING(tenant_id) WHERE m.user_id = $1 AND m.tenant_id = $2 AND m.is_active = TRUE AND t.is_active = TRUE LIMIT 1",
        )
        .bind(user_id)
        .bind(tenant_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| Error::internal(format!("Failed to check tenant access: {}", e)))?;

        Ok(exists.is_some())
    }

    pub(super) async fn pg_check_workspace_access(
        &self,
        user_id: Uuid,
        workspace_id: Uuid,
    ) -> Result<bool> {
        let exists: Option<(i32,)> = sqlx::query_as(
            "SELECT 1 FROM memberships m JOIN workspaces w ON w.tenant_id = m.tenant_id JOIN tenants t ON t.tenant_id=w.tenant_id WHERE m.user_id = $1 AND w.workspace_id = $2 AND m.is_active = TRUE AND w.is_active = TRUE AND t.is_active = TRUE AND (m.workspace_id IS NULL OR m.workspace_id = $2) LIMIT 1",
        )
        .bind(user_id)
        .bind(workspace_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| Error::internal(format!("Failed to check workspace access: {}", e)))?;

        Ok(exists.is_some())
    }

    pub(super) async fn pg_get_user_role(
        &self,
        user_id: Uuid,
        tenant_id: Uuid,
    ) -> Result<Option<MembershipRole>> {
        let role: Option<(String,)> = sqlx::query_as(
            "SELECT role FROM memberships WHERE user_id = $1 AND tenant_id = $2 AND is_active = TRUE AND workspace_id IS NULL ORDER BY CASE role WHEN 'owner' THEN 4 WHEN 'admin' THEN 3 WHEN 'member' THEN 2 ELSE 1 END DESC LIMIT 1",
        )
        .bind(user_id)
        .bind(tenant_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| Error::internal(format!("Failed to get user role: {}", e)))?;

        Ok(role.map(|(r,)| Self::parse_role(&r)))
    }

    // ============ Context Operations ============

    pub(super) async fn pg_build_context(
        &self,
        user_id: Uuid,
        tenant_id: Uuid,
        workspace_id: Option<Uuid>,
    ) -> Result<TenantContext> {
        // Verify tenant exists
        let tenant = self
            .pg_get_tenant(tenant_id)
            .await?
            .ok_or_else(|| Error::not_found(format!("Tenant {} not found", tenant_id)))?;
        if !tenant.is_active {
            return Err(Error::validation("Tenant is inactive"));
        }

        // Verify workspace if provided
        if let Some(ws_id) = workspace_id {
            let workspace = self
                .pg_get_workspace(ws_id)
                .await?
                .ok_or_else(|| Error::not_found(format!("Workspace {} not found", ws_id)))?;

            if !workspace.is_active || workspace.tenant_id != tenant_id {
                return Err(Error::validation(
                    "Workspace does not belong to the specified tenant",
                ));
            }
        }

        // Get user's role in this tenant
        let role = self
            .pg_get_user_memberships(user_id)
            .await?
            .into_iter()
            .filter(|m| m.applies_to(tenant_id, workspace_id))
            .map(|m| m.role)
            .max_by_key(MembershipRole::level)
            .ok_or_else(|| Error::validation("No active membership for scope"))?;

        Ok(TenantContext {
            tenant_id: Some(tenant_id),
            workspace_id,
            user_id: Some(user_id),
            role: Some(role),
        })
    }
}
