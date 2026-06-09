//! Shared tenant/workspace scope matching for documents and PDFs.
//!
//! WHY: Listing, deletion, recovery, and duplicate detection must agree on whether
//! a document belongs to the active workspace. UUID aliases (`default`) must match
//! stored UUID metadata.

use uuid::Uuid;

use crate::middleware::TenantContext;

fn parse_workspace_uuid_or_default(workspace_id: Option<&str>) -> Option<Uuid> {
    crate::middleware::resolve_workspace_uuid(workspace_id)
}

fn is_legacy_default_workspace_context(workspace_id: Option<&str>) -> bool {
    match workspace_id.map(str::trim) {
        None | Some("") | Some("default") => true,
        Some(value) => match Uuid::parse_str(value) {
            Ok(uuid) => {
                uuid == crate::middleware::default_tenant_uuid()
                    || uuid == crate::middleware::default_workspace_uuid()
            }
            Err(_) => false,
        },
    }
}

fn is_legacy_default_tenant_context(tenant_id: Option<&str>) -> bool {
    match tenant_id.map(str::trim) {
        None | Some("") | Some("default") => true,
        Some(value) => match Uuid::parse_str(value) {
            Ok(uuid) => uuid == crate::middleware::default_tenant_uuid(),
            Err(_) => false,
        },
    }
}

/// Check whether metadata belongs to the requester's workspace (UUID-normalized).
pub fn metadata_matches_workspace_context(
    metadata: &serde_json::Value,
    tenant_ctx: &TenantContext,
) -> bool {
    let stored_workspace_raw = metadata
        .get("workspace_id")
        .and_then(|value| value.as_str())
        .map(str::trim);

    if matches!(stored_workspace_raw, None | Some("") | Some("default")) {
        return is_legacy_default_workspace_context(tenant_ctx.workspace_id.as_deref());
    }

    let Some(ctx_workspace_id) =
        parse_workspace_uuid_or_default(tenant_ctx.workspace_id.as_deref())
    else {
        return true;
    };

    let Some(stored_workspace_id) = parse_workspace_uuid_or_default(stored_workspace_raw) else {
        return false;
    };

    stored_workspace_id == ctx_workspace_id
}

/// Check whether metadata belongs to the requester's tenant (UUID-normalized).
pub fn metadata_matches_tenant_id_context(
    metadata: &serde_json::Value,
    tenant_ctx: &TenantContext,
) -> bool {
    let stored_tenant_raw = metadata
        .get("tenant_id")
        .and_then(|value| value.as_str())
        .map(str::trim);

    if matches!(stored_tenant_raw, None | Some("") | Some("default")) {
        return is_legacy_default_tenant_context(tenant_ctx.tenant_id.as_deref());
    }

    let Some(ctx_tenant_id) =
        crate::middleware::resolve_tenant_uuid(tenant_ctx.tenant_id.as_deref())
    else {
        return true;
    };

    let Some(stored_tenant_id) = crate::middleware::resolve_tenant_uuid(stored_tenant_raw) else {
        return false;
    };

    stored_tenant_id == ctx_tenant_id
}

/// Check whether a metadata payload belongs to the requester's tenant + workspace.
pub fn metadata_matches_tenant_context(
    metadata: &serde_json::Value,
    tenant_ctx: &TenantContext,
) -> bool {
    metadata_matches_workspace_context(metadata, tenant_ctx)
        && metadata_matches_tenant_id_context(metadata, tenant_ctx)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::middleware::{default_tenant_uuid, default_workspace_uuid};

    fn ctx(tenant: &str, workspace: &str) -> TenantContext {
        TenantContext {
            tenant_id: Some(tenant.to_string()),
            workspace_id: Some(workspace.to_string()),
            user_id: None,
        }
    }

    #[test]
    fn uuid_stored_metadata_visible_from_default_workspace_alias() {
        let metadata = serde_json::json!({
            "workspace_id": default_workspace_uuid().to_string(),
            "tenant_id": default_tenant_uuid().to_string(),
        });
        let tenant_ctx = ctx("default", "default");
        assert!(metadata_matches_tenant_context(&metadata, &tenant_ctx));
    }

    #[test]
    fn uuid_stored_metadata_hidden_from_other_workspace() {
        let metadata = serde_json::json!({
            "workspace_id": default_workspace_uuid().to_string(),
            "tenant_id": default_tenant_uuid().to_string(),
        });
        let other = uuid::Uuid::new_v4().to_string();
        let tenant_ctx = ctx("default", &other);
        assert!(!metadata_matches_tenant_context(&metadata, &tenant_ctx));
    }

    #[test]
    fn legacy_default_metadata_visible_from_default_alias() {
        let metadata = serde_json::json!({
            "workspace_id": "default",
            "tenant_id": "default",
        });
        let tenant_ctx = ctx("default", "default");
        assert!(metadata_matches_tenant_context(&metadata, &tenant_ctx));
    }
}
