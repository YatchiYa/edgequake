//! SPEC-169 e2e — Members directory, role changes, removal and invitations through the real router.
//! Requires `--features postgres` (the user store is a stub in non-PG builds).
#![cfg(feature = "postgres")]

mod common;

use axum::http::StatusCode;
use common::spec169::*;
use serde_json::{json, Value};

#[tokio::test]
async fn directory_lists_people_with_roles_sign_in_and_viewer_context() {
    let org = org().await;
    let dir = org.directory(&org.owner, "").await;

    assert_eq!(dir["tenant_name"], "Acme");
    assert_eq!(dir["summary"]["total"], 3);
    assert_eq!(dir["summary"]["by_role"]["owner"], 1);
    assert_eq!(dir["viewer"]["role"], "owner");
    assert_eq!(dir["viewer"]["max_assignable_role"], "owner");

    let owen = row(&dir, "owen@acme.test");
    assert_eq!(owen["role"], "owner");
    assert_eq!(owen["is_you"], true);
    assert_eq!(owen["is_last_owner"], true);
    assert_eq!(owen["sign_in"][0]["kind"], "sso");
    assert_eq!(owen["sign_in"][0]["provider_slug"], "idp");
    assert_eq!(row(&dir, "mia@acme.test")["role"], "member");

    let found = org.directory(&org.owner, "?q=ADA").await;
    assert_eq!(found["total"], 1);
    let admins = org.directory(&org.owner, "?role=admin").await;
    assert_eq!(admins["members"][0]["email"], "ada@acme.test");
    let paged = org.directory(&org.owner, "?page_size=2&page=2").await;
    assert_eq!(paged["members"].as_array().unwrap().len(), 1);
    assert_eq!(paged["total_pages"], 2);
}

#[tokio::test]
async fn only_managers_may_read_the_directory() {
    let org = org().await;
    let (status, _) = org
        .call("GET", &org.members_url(), Some(&org.member), None)
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    let (status, _) = org.call("GET", &org.members_url(), None, None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let (status, _) = org
        .call("GET", &org.members_url(), Some(&org.platform), None)
        .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "platform admins manage every organization"
    );
}

#[tokio::test]
async fn role_changes_respect_rank_and_unknown_roles_are_rejected() {
    let org = org().await;
    let mia = format!("{}/{}", org.members_url(), org.member_id);

    // Admin promotes a member to admin, but cannot mint an owner.
    let (status, body) = org
        .call(
            "PATCH",
            &mia,
            Some(&org.admin),
            Some(json!({"role": "admin"})),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["role"], "admin");
    let (status, _) = org
        .call(
            "PATCH",
            &mia,
            Some(&org.admin),
            Some(json!({"role": "owner"})),
        )
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    // Admin cannot touch an owner.
    let owen = format!("{}/{}", org.members_url(), org.owner_id);
    let (status, _) = org
        .call(
            "PATCH",
            &owen,
            Some(&org.admin),
            Some(json!({"role": "member"})),
        )
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    // Unknown role: 400, never coerced.
    let (status, _) = org
        .call(
            "PATCH",
            &mia,
            Some(&org.owner),
            Some(json!({"role": "superuser"})),
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    // The last owner is protected, until another owner exists.
    let (status, _) = org
        .call(
            "PATCH",
            &owen,
            Some(&org.owner),
            Some(json!({"role": "admin"})),
        )
        .await;
    assert_eq!(status, StatusCode::CONFLICT);
    let ada = format!("{}/{}", org.members_url(), org.admin_id);
    let (status, _) = org
        .call(
            "PATCH",
            &ada,
            Some(&org.owner),
            Some(json!({"role": "owner"})),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    let (status, _) = org
        .call(
            "PATCH",
            &owen,
            Some(&org.owner),
            Some(json!({"role": "admin"})),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn admin_chosen_role_survives_the_next_sso_sign_in() {
    let org = org().await;
    let mia = format!("{}/{}", org.members_url(), org.member_id);
    let (status, _) = org
        .call(
            "PATCH",
            &mia,
            Some(&org.owner),
            Some(json!({"role": "readonly"})),
        )
        .await;
    assert_eq!(status, StatusCode::OK);

    let again = org
        .sso
        .login(
            "",
            "m",
            json!({"email": "mia@acme.test", "organization": ["acme"]}),
        )
        .await;
    assert_eq!(again.status(), StatusCode::OK);
    let dir = org.directory(&org.owner, "").await;
    assert_eq!(row(&dir, "mia@acme.test")["role"], "readonly");
    assert_eq!(
        row(&dir, "mia@acme.test")["role_managed_by"],
        Value::Null,
        "pinned, no longer IdP-managed"
    );
}

#[tokio::test]
async fn account_level_changes_are_platform_admin_only() {
    let org = org().await;
    let mia = format!("{}/{}", org.members_url(), org.member_id);
    let (status, _) = org
        .call(
            "PATCH",
            &mia,
            Some(&org.owner),
            Some(json!({"is_active": false})),
        )
        .await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "an organization owner cannot suspend a platform account"
    );
    let (status, _) = org
        .call(
            "PATCH",
            &mia,
            Some(&org.owner),
            Some(json!({"instance_role": "admin"})),
        )
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    let (status, body) = org
        .call(
            "PATCH",
            &mia,
            Some(&org.platform),
            Some(json!({"is_active": false})),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["status"], "deactivated");
}

#[tokio::test]
async fn removal_follows_rank_and_never_orphans_the_organization() {
    let org = org().await;
    let owen = format!("{}/{}", org.members_url(), org.owner_id);
    let (status, _) = org.call("DELETE", &owen, Some(&org.admin), None).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    let (status, _) = org.call("DELETE", &owen, Some(&org.owner), None).await;
    assert_eq!(status, StatusCode::CONFLICT, "the last owner cannot leave");

    let mia = format!("{}/{}", org.members_url(), org.member_id);
    let (status, _) = org.call("DELETE", &mia, Some(&org.admin), None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let dir = org.directory(&org.owner, "").await;
    assert_eq!(dir["summary"]["total"], 2);
    let (status, _) = org.call("DELETE", &mia, Some(&org.admin), None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn invitation_lifecycle_create_preview_accept_and_replay() {
    let org = org().await;

    // Rank-limited: an admin cannot invite an owner.
    let (status, _) = org.invite(&org.admin, "new@acme.test", "owner").await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    let (status, _) = org.invite(&org.owner, "not-an-email", "member").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let (status, _) = org.invite(&org.owner, "mia@acme.test", "member").await;
    assert_eq!(status, StatusCode::CONFLICT, "already a member");

    let (status, issued) = org.invite(&org.admin, "New@Acme.test", "admin").await;
    assert_eq!(status, StatusCode::CREATED, "{issued}");
    let token = issued["token"].as_str().unwrap().to_string();
    assert_eq!(issued["invitation"]["email"], "new@acme.test");
    assert_eq!(issued["invitation"]["status"], "invited");
    let (status, _) = org.invite(&org.admin, "new@acme.test", "member").await;
    assert_eq!(status, StatusCode::CONFLICT, "already invited");

    let dir = org.directory(&org.owner, "?status=invited").await;
    assert_eq!(dir["total"], 1);
    assert_eq!(dir["members"][0]["kind"], "invitation");
    assert_eq!(dir["members"][0]["role"], "admin");

    // Public preview (no session).
    let (status, preview) = org
        .call(
            "GET",
            &format!("/api/v1/auth/invitations/{token}"),
            None,
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{preview}");
    assert_eq!(preview["tenant_name"], "Acme");
    assert_eq!(preview["role_label"], "Admin");
    assert_eq!(preview["account_exists"], false);
    assert!(
        !preview.to_string().contains(&token),
        "the preview never echoes the token"
    );

    // Accept creates the account and the grant.
    let accept = format!("/api/v1/auth/invitations/{token}/accept");
    let (status, _) = org.call("POST", &accept, None, Some(json!({}))).await;
    assert_eq!(
        status,
        StatusCode::BAD_REQUEST,
        "credentials required for a new account"
    );
    let (status, joined) = org
        .call(
            "POST",
            &accept,
            None,
            Some(json!({"username": "nina", "password": PASSWORD})),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED, "{joined}");
    assert_eq!(joined["created_account"], true);
    assert_eq!(joined["role"], "admin");

    let dir = org.directory(&org.owner, "").await;
    assert_eq!(row(&dir, "new@acme.test")["role"], "admin");
    assert_eq!(row(&dir, "new@acme.test")["sign_in"][0]["kind"], "password");
    assert_eq!(dir["summary"]["invited"], 0);

    // Single use.
    let (status, _) = org
        .call(
            "POST",
            &accept,
            None,
            Some(json!({"username": "nina2", "password": PASSWORD})),
        )
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, _) = org
        .call(
            "GET",
            &format!("/api/v1/auth/invitations/{token}"),
            None,
            None,
        )
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn revoked_resent_and_garbage_links_behave() {
    let org = org().await;
    let (_, first) = org.invite(&org.owner, "a@acme.test", "member").await;
    let id = first["invitation"]["id"].as_str().unwrap().to_string();
    let old_token = first["token"].as_str().unwrap().to_string();

    // Resend rotates the token: the old link dies, the new one works.
    let (status, resent) = org
        .call(
            "POST",
            &format!("/api/v1/tenants/{}/invitations/{id}/resend", org.tenant),
            Some(&org.owner),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{resent}");
    let new_token = resent["token"].as_str().unwrap().to_string();
    assert_ne!(new_token, old_token);
    let (status, _) = org
        .call(
            "GET",
            &format!("/api/v1/auth/invitations/{old_token}"),
            None,
            None,
        )
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, _) = org
        .call(
            "GET",
            &format!("/api/v1/auth/invitations/{new_token}"),
            None,
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK);

    // Revoke kills the link and frees the address.
    let (status, _) = org
        .call(
            "DELETE",
            &format!("/api/v1/tenants/{}/invitations/{id}", org.tenant),
            Some(&org.owner),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, _) = org
        .call(
            "GET",
            &format!("/api/v1/auth/invitations/{new_token}"),
            None,
            None,
        )
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, _) = org.invite(&org.owner, "a@acme.test", "member").await;
    assert_eq!(status, StatusCode::CREATED);

    let (status, _) = org
        .call(
            "GET",
            "/api/v1/auth/invitations/not-a-real-token",
            None,
            None,
        )
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, _) = org
        .call(
            "POST",
            "/api/v1/auth/invitations/not-a-real-token/accept",
            None,
            Some(json!({})),
        )
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn expired_invitation_is_gone() {
    let org = org().await;
    let (_, issued) = org.invite(&org.owner, "late@acme.test", "member").await;
    let token = issued["token"].as_str().unwrap();
    let hash = edgequake_api::services::invitations::token::hash_token(token);
    let store = &org.sso.state.auth.invitations;
    let inv = store.find_open_by_token(&hash).await.unwrap().unwrap();
    store
        .rotate_token(
            org.tenant,
            inv.invitation_id,
            &hash,
            chrono::Utc::now() - chrono::Duration::hours(1),
        )
        .await
        .unwrap();

    let (status, _) = org
        .call(
            "GET",
            &format!("/api/v1/auth/invitations/{token}"),
            None,
            None,
        )
        .await;
    assert_eq!(status, StatusCode::GONE);
    let dir = org.directory(&org.owner, "").await;
    assert_eq!(row(&dir, "late@acme.test")["status"], "expired");
}

#[tokio::test]
async fn existing_account_must_sign_in_to_accept() {
    let org = org().await;
    let uid = org
        .sso
        .create_local_user("pat", "pat@acme.test", PASSWORD)
        .await;
    let (_, issued) = org.invite(&org.owner, "pat@acme.test", "member").await;
    let token = issued["token"].as_str().unwrap();
    let accept = format!("/api/v1/auth/invitations/{token}/accept");

    let (_, preview) = org
        .call(
            "GET",
            &format!("/api/v1/auth/invitations/{token}"),
            None,
            None,
        )
        .await;
    assert_eq!(preview["account_exists"], true);

    // Anonymous and foreign sessions can never attach a grant to someone else's account.
    let (status, _) = org.call("POST", &accept, None, Some(json!({}))).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let (status, _) = org
        .call("POST", &accept, Some(&org.member), Some(json!({})))
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    let pat_token = org
        .sso
        .state
        .auth
        .jwt
        .generate_token(uid.parse().unwrap(), edgequake_auth::Role::User)
        .unwrap();
    let (status, joined) = org
        .call("POST", &accept, Some(&pat_token), Some(json!({})))
        .await;
    assert_eq!(status, StatusCode::CREATED, "{joined}");
    assert_eq!(joined["created_account"], false);
}

#[tokio::test]
async fn role_catalog_is_served_from_the_enforcement_predicates() {
    let org = org().await;
    let (status, catalog) = org
        .call("GET", "/api/v1/roles", Some(&org.member), None)
        .await;
    assert_eq!(status, StatusCode::OK, "{catalog}");
    let ids: Vec<&str> = catalog["organization_roles"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["id"].as_str().unwrap())
        .collect();
    assert_eq!(ids, ["owner", "admin", "member", "readonly"]);
    assert!(!catalog["capabilities"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn legacy_user_api_rejects_unknown_roles_instead_of_coercing_them() {
    let org = org().await;
    let (status, _) = org
        .call(
            "POST",
            "/api/v1/users",
            Some(&org.platform),
            Some(json!({"username": "vic", "email": "vic@x.test", "password": PASSWORD, "role": "viewer-ish"})),
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let (status, body) = org
        .call(
            "POST",
            "/api/v1/users",
            Some(&org.platform),
            Some(json!({"username": "vic", "email": "vic@x.test", "password": PASSWORD, "role": "viewer"})),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    assert_eq!(
        body["user"]["role"], "readonly",
        "`viewer` is read-only, never `user`"
    );
}
