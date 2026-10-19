//! SPEC-169 shared fixture: an organization (Acme) with an Owner, an Admin and a Member,
//! provisioned through SSO, driven through the real router.
#![cfg(feature = "postgres")]
#![allow(dead_code)]

use super::spec158::*;
use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use edgequake_core::MembershipRole;
use serde_json::{json, Value};
use tower::ServiceExt;
use uuid::Uuid;

pub const PASSWORD: &str = "Str0ng!Passw0rd#2026";

pub struct Org {
    pub sso: Sso,
    pub tenant: Uuid,
    pub owner: String,
    pub admin: String,
    pub member: String,
    pub platform: String,
    pub member_id: String,
    pub admin_id: String,
    pub owner_id: String,
}

/// Acme with one Owner, one Admin and one Member, all provisioned through SSO.
pub async fn org() -> Org {
    let mut sso = Sso::start(
        edgequake_api::services::federation::policy::FederationPolicy {
            require_org: true,
            ..policy()
        },
        None,
    )
    .await;
    let platform = sso.platform_admin_token().await;
    let acme = sso.create_tenant("Acme", "acme").await;

    let mut tokens = Vec::new();
    for (sub, email) in [
        ("o", "owen@acme.test"),
        ("a", "ada@acme.test"),
        ("m", "mia@acme.test"),
    ] {
        let res = sso
            .login(
                "",
                sub,
                json!({"email": email, "organization": ["acme"], "preferred_username": sub}),
            )
            .await;
        assert_eq!(res.status(), StatusCode::OK);
        let body = json_body(res).await;
        tokens.push((
            body["access_token"].as_str().unwrap().to_string(),
            body["user"]["user_id"].as_str().unwrap().to_string(),
        ));
    }
    for ((_, uid), role) in tokens.iter().zip([
        MembershipRole::Owner,
        MembershipRole::Admin,
        MembershipRole::Member,
    ]) {
        let m = sso
            .state
            .workspace_service
            .get_user_memberships(uid.parse().unwrap())
            .await
            .unwrap()
            .pop()
            .unwrap();
        sso.state
            .workspace_service
            .assign_membership_role(m.membership_id, role)
            .await
            .unwrap();
    }
    sso.enforce_auth();
    Org {
        owner: tokens[0].0.clone(),
        admin: tokens[1].0.clone(),
        member: tokens[2].0.clone(),
        owner_id: tokens[0].1.clone(),
        admin_id: tokens[1].1.clone(),
        member_id: tokens[2].1.clone(),
        tenant: acme.tenant_id,
        platform,
        sso,
    }
}

impl Org {
    pub async fn call(
        &self,
        method: &str,
        uri: &str,
        token: Option<&str>,
        body: Option<Value>,
    ) -> (StatusCode, Value) {
        let mut req = Request::builder()
            .method(method)
            .uri(uri)
            .header(header::CONTENT_TYPE, "application/json");
        if let Some(t) = token {
            req = req.header(header::AUTHORIZATION, format!("Bearer {t}"));
        }
        let res = self
            .sso
            .app()
            .oneshot(
                req.body(Body::from(body.map_or_else(String::new, |b| b.to_string())))
                    .unwrap(),
            )
            .await
            .unwrap();
        let status = res.status();
        let bytes = axum::body::to_bytes(res.into_body(), usize::MAX)
            .await
            .unwrap();
        let json = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
        (status, json)
    }

    pub fn members_url(&self) -> String {
        format!("/api/v1/tenants/{}/members", self.tenant)
    }

    pub async fn directory(&self, token: &str, query: &str) -> Value {
        let (status, body) = self
            .call(
                "GET",
                &format!("{}{query}", self.members_url()),
                Some(token),
                None,
            )
            .await;
        assert_eq!(status, StatusCode::OK, "{body}");
        body
    }

    pub async fn invite(&self, token: &str, email: &str, role: &str) -> (StatusCode, Value) {
        self.call(
            "POST",
            &format!("/api/v1/tenants/{}/invitations", self.tenant),
            Some(token),
            Some(json!({"email": email, "role": role})),
        )
        .await
    }
}

pub fn row<'a>(directory: &'a Value, email: &str) -> &'a Value {
    directory["members"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["email"] == email)
        .unwrap_or_else(|| panic!("{email} not in {directory}"))
}
