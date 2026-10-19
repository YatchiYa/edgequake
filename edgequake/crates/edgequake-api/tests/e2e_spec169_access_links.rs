//! SPEC-169 e2e — one-time magic sign-in links, password-reset links and self-service password
//! change, through the real router. Requires `--features postgres`.
#![cfg(feature = "postgres")]

mod common;

use axum::http::StatusCode;
use common::spec169::*;
use edgequake_core::{Membership, MembershipRole};
use serde_json::{json, Value};

const NEW_PASSWORD: &str = "An0ther!Str0ng#Pass2027";

impl Org {
    /// Invite `email` and accept as a brand-new password account; returns the user id.
    async fn join(&self, email: &str, username: &str, role: &str) -> String {
        let (status, issued) = self.invite(&self.owner, email, role).await;
        assert_eq!(status, StatusCode::CREATED, "{issued}");
        let token = issued["token"].as_str().unwrap();
        let (status, joined) = self
            .call(
                "POST",
                &format!("/api/v1/auth/invitations/{token}/accept"),
                None,
                Some(json!({"username": username, "password": PASSWORD})),
            )
            .await;
        assert_eq!(status, StatusCode::CREATED, "{joined}");
        let dir = self.directory(&self.owner, "").await;
        row(&dir, email)["user_id"].as_str().unwrap().to_string()
    }

    async fn login(&self, username: &str, password: &str) -> (StatusCode, Value) {
        self.call(
            "POST",
            "/api/v1/auth/login",
            None,
            Some(json!({"username": username, "password": password})),
        )
        .await
    }

    async fn issue(&self, token: &str, user_id: &str, purpose: &str) -> (StatusCode, Value) {
        self.call(
            "POST",
            &format!(
                "/api/v1/tenants/{}/members/{user_id}/access-links",
                self.tenant
            ),
            Some(token),
            Some(json!({"purpose": purpose})),
        )
        .await
    }

    async fn redeem(&self, link: &str, body: Value) -> (StatusCode, Value) {
        self.call(
            "POST",
            &format!("/api/v1/auth/access-links/{link}/redeem"),
            None,
            Some(body),
        )
        .await
    }

    async fn preview(&self, link: &str) -> (StatusCode, Value) {
        self.call(
            "GET",
            &format!("/api/v1/auth/access-links/{link}"),
            None,
            None,
        )
        .await
    }
}

fn token_of(issued: &Value) -> String {
    issued["token"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn magic_link_signs_in_once_without_a_password() {
    let org = org().await;
    let nina = org.join("nina@acme.test", "nina", "member").await;

    let (status, issued) = org.issue(&org.owner, &nina, "magic_login").await;
    assert_eq!(status, StatusCode::CREATED, "{issued}");
    assert_eq!(issued["purpose"], "magic_login");
    assert_eq!(issued["expires_in_minutes"], 15);
    let link = token_of(&issued);

    let (status, preview) = org.preview(&link).await;
    assert_eq!(status, StatusCode::OK, "{preview}");
    assert_eq!(preview["username"], "nina");
    assert_eq!(preview["tenant_name"], "Acme");
    assert!(
        !preview.to_string().contains(&link),
        "the preview never echoes the token"
    );

    let (status, session) = org.redeem(&link, json!({})).await;
    assert_eq!(status, StatusCode::OK, "{session}");
    assert_eq!(session["user"]["username"], "nina");
    let access = session["access_token"].as_str().unwrap();
    let (status, me) = org.call("GET", "/api/v1/auth/me", Some(access), None).await;
    assert_eq!(status, StatusCode::OK, "{me}");

    // The old password is untouched, and the link is spent.
    assert_eq!(org.login("nina", PASSWORD).await.0, StatusCode::OK);
    assert_eq!(org.redeem(&link, json!({})).await.0, StatusCode::NOT_FOUND);
    assert_eq!(org.preview(&link).await.0, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn reset_link_sets_a_new_password_and_signs_everyone_else_out() {
    let org = org().await;
    let nina = org.join("nina@acme.test", "nina", "member").await;
    let (_, before) = org.login("nina", PASSWORD).await;
    let stale_refresh = before["refresh_token"].as_str().map(str::to_string);

    let (status, issued) = org.issue(&org.admin, &nina, "password_reset").await;
    assert_eq!(status, StatusCode::CREATED, "{issued}");
    assert_eq!(issued["expires_in_minutes"], 24 * 60);
    let link = token_of(&issued);

    // A missing or weak password never burns the link.
    assert_eq!(
        org.redeem(&link, json!({})).await.0,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        org.redeem(&link, json!({"password": "short"})).await.0,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        org.redeem(&link, json!({"password": "password"})).await.0,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(org.preview(&link).await.0, StatusCode::OK, "still usable");

    let (status, session) = org.redeem(&link, json!({"password": NEW_PASSWORD})).await;
    assert_eq!(status, StatusCode::OK, "{session}");
    assert!(session["access_token"].as_str().is_some());

    assert_eq!(
        org.login("nina", PASSWORD).await.0,
        StatusCode::UNAUTHORIZED,
        "old password is dead"
    );
    assert_eq!(org.login("nina", NEW_PASSWORD).await.0, StatusCode::OK);
    assert_eq!(
        org.redeem(&link, json!({"password": NEW_PASSWORD})).await.0,
        StatusCode::NOT_FOUND
    );

    if let Some(stale) = stale_refresh {
        let (status, _) = org
            .call(
                "POST",
                "/api/v1/auth/refresh",
                None,
                Some(json!({"refresh_token": stale})),
            )
            .await;
        assert_eq!(
            status,
            StatusCode::UNAUTHORIZED,
            "sessions from before the reset are revoked"
        );
    }
}

#[tokio::test]
async fn reissuing_replaces_the_previous_link_of_the_same_purpose() {
    let org = org().await;
    let nina = org.join("nina@acme.test", "nina", "member").await;
    let first = token_of(&org.issue(&org.owner, &nina, "magic_login").await.1);
    let reset = token_of(&org.issue(&org.owner, &nina, "password_reset").await.1);
    let second = token_of(&org.issue(&org.owner, &nina, "magic_login").await.1);

    assert_eq!(
        org.preview(&first).await.0,
        StatusCode::NOT_FOUND,
        "superseded"
    );
    assert_eq!(org.preview(&second).await.0, StatusCode::OK);
    assert_eq!(
        org.preview(&reset).await.0,
        StatusCode::OK,
        "other purpose is independent"
    );
}

#[tokio::test]
async fn issuing_follows_rank_and_never_reaches_across_organizations() {
    let org = org().await;
    let nina = org.join("nina@acme.test", "nina", "member").await;
    let pat = org.join("pat@acme.test", "pat", "admin").await;

    assert_eq!(
        org.call(
            "POST",
            &format!("{}/{nina}/access-links", org.members_url()),
            None,
            Some(json!({"purpose": "magic_login"}))
        )
        .await
        .0,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        org.issue(&org.member, &nina, "magic_login").await.0,
        StatusCode::FORBIDDEN,
        "members cannot issue"
    );
    assert_eq!(
        org.issue(&org.owner, &nina, "teleport").await.0,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        org.issue(&org.admin, &pat, "magic_login").await.0,
        StatusCode::FORBIDDEN,
        "peer admin"
    );
    assert_eq!(
        org.issue(&org.admin, &nina, "magic_login").await.0,
        StatusCode::CREATED
    );
    assert_eq!(
        org.issue(&org.owner, &pat, "magic_login").await.0,
        StatusCode::CREATED
    );
    assert_eq!(
        org.issue(&org.admin, &org.owner_id, "magic_login").await.0,
        StatusCode::FORBIDDEN
    );

    // Never for yourself; never for people their identity provider manages.
    assert_eq!(
        org.issue(&org.owner, &org.owner_id, "password_reset")
            .await
            .0,
        StatusCode::CONFLICT
    );
    assert_eq!(
        org.issue(&org.owner, &org.member_id, "password_reset")
            .await
            .0,
        StatusCode::CONFLICT
    );

    // Someone who also belongs to another organization is out of an org owner's reach.
    let globex = org.sso.create_tenant("Globex", "globex").await;
    org.sso
        .state
        .workspace_service
        .add_membership(Membership::new(
            nina.parse().unwrap(),
            globex.tenant_id,
            MembershipRole::Member,
        ))
        .await
        .unwrap();
    let (status, body) = org.issue(&org.owner, &nina, "magic_login").await;
    assert_eq!(status, StatusCode::FORBIDDEN, "{body}");
    assert_eq!(
        org.issue(&org.platform, &nina, "magic_login").await.0,
        StatusCode::CREATED
    );
}

#[tokio::test]
async fn suspended_accounts_cannot_hold_or_use_a_link() {
    let org = org().await;
    let nina = org.join("nina@acme.test", "nina", "member").await;
    let link = token_of(&org.issue(&org.owner, &nina, "magic_login").await.1);

    let (status, body) = org
        .call(
            "PATCH",
            &format!("{}/{nina}", org.members_url()),
            Some(&org.platform),
            Some(json!({"is_active": false})),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");

    assert_eq!(
        org.redeem(&link, json!({})).await.0,
        StatusCode::NOT_FOUND,
        "suspended after issue"
    );
    assert_eq!(
        org.issue(&org.owner, &nina, "magic_login").await.0,
        StatusCode::CONFLICT
    );
}

#[tokio::test]
async fn garbage_and_expired_links_behave() {
    let org = org().await;
    assert_eq!(
        org.preview("not-a-real-token").await.0,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        org.redeem(&"x".repeat(300), json!({})).await.0,
        StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn changing_your_own_password_needs_the_current_one() {
    let org = org().await;
    org.join("nina@acme.test", "nina", "member").await;
    let (_, session) = org.login("nina", PASSWORD).await;
    let token = session["access_token"].as_str().unwrap().to_string();
    let change = |current: &str, new: &str| {
        org.call(
            "POST",
            "/api/v1/auth/password",
            Some(&token),
            Some(json!({"current_password": current, "new_password": new})),
        )
    };

    assert_eq!(
        change("wrong-Password1!", NEW_PASSWORD).await.0,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        change(PASSWORD, PASSWORD).await.0,
        StatusCode::BAD_REQUEST,
        "unchanged"
    );
    assert_eq!(change(PASSWORD, "weak").await.0, StatusCode::BAD_REQUEST);
    assert_eq!(
        org.call(
            "POST",
            "/api/v1/auth/password",
            None,
            Some(json!({"current_password": PASSWORD, "new_password": NEW_PASSWORD}))
        )
        .await
        .0,
        StatusCode::UNAUTHORIZED
    );

    let (status, fresh) = change(PASSWORD, NEW_PASSWORD).await;
    assert_eq!(status, StatusCode::OK, "{fresh}");
    assert!(
        fresh["access_token"].as_str().is_some(),
        "this device stays signed in"
    );
    assert_eq!(
        org.login("nina", PASSWORD).await.0,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(org.login("nina", NEW_PASSWORD).await.0, StatusCode::OK);
}
