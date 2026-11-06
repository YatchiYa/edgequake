//! SPEC-184 — workspace ontology: save/validate/sync, draft, plan, realign, remove.
//!
//! ```bash
//! cargo test -p edgequake-api --test e2e_spec184_workspace_ontology
//! ```

mod common;

use axum::http::StatusCode;
use common::{
    create_test_app, delete_with_tenant, get_with_tenant, post_json, post_json_with_tenant,
    put_with_tenant,
};
use serde_json::{json, Value};

const USER: &str = "00000000-0000-0000-0000-0000000000f1";

struct Ws {
    app: axum::Router,
    tenant: String,
    id: String,
}

impl Ws {
    async fn new() -> Self {
        let app = create_test_app();
        let (status, tenant) = post_json(
            &app,
            "/api/v1/tenants",
            &json!({ "name": format!("SPEC184 {}", uuid::Uuid::new_v4()) }),
        )
        .await;
        assert!(status.is_success(), "{tenant}");
        let tenant = tenant["id"].as_str().unwrap().to_string();
        let (status, ws) = post_json(
            &app,
            &format!("/api/v1/tenants/{tenant}/workspaces"),
            &json!({
                "name": "Propulsion",
                "entity_types": ["PERSON", "ORGANIZATION", "PRODUCT", "ARTIFACT"],
                "kg_schema_preset": "manufacturing"
            }),
        )
        .await;
        assert!(status.is_success(), "{ws}");
        let id = ws["id"].as_str().unwrap().to_string();
        Self { app, tenant, id }
    }

    async fn get(&self, path: &str) -> (StatusCode, Value) {
        get_with_tenant(&self.app, path, &self.tenant, USER, &self.id).await
    }
    async fn post(&self, path: &str, body: Value) -> (StatusCode, Value) {
        post_json_with_tenant(&self.app, path, &body, &self.tenant, USER, &self.id).await
    }
    async fn put(&self, path: &str, body: Value) -> (StatusCode, Value) {
        put_with_tenant(&self.app, path, &body, &self.tenant, USER, &self.id).await
    }
    async fn delete(&self, path: &str) -> (StatusCode, Value) {
        delete_with_tenant(&self.app, path, &self.tenant, USER, &self.id).await
    }
    fn route(&self, tail: &str) -> String {
        format!("/api/v1/workspaces/{}/ontology{tail}", self.id)
    }
}

fn ontology() -> Value {
    json!({
        "name": "Propulsion",
        "entity_types": [
            {"id": "ENGINE", "description": "A gas turbine engine.", "parent": "PRODUCT", "synonyms": ["TURBOFAN"]},
            {"id": "PRODUCT", "description": "Something sold."},
            {"id": "COMPONENT", "description": "A part of an engine."}
        ],
        "relation_types": [
            {"id": "PART_OF", "description": "Is a part of.", "source_types": ["COMPONENT"], "target_types": ["ENGINE"]}
        ],
        "terms": [
            {"name": "LEAP-1A", "entity_type": "ENGINE", "aliases": ["LEAP 1A", "Leap1A"]},
            {"name": "CFM56", "entity_type": "ENGINE"}
        ]
    })
}

#[tokio::test]
async fn spec184_save_syncs_schema_and_delete_restores_it() {
    let ws = Ws::new().await;

    let (status, body) = ws.get(&ws.route("")).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(body["ontology"].is_null());

    // LAW-184-4: one alias, two terms → 400 listing the problem; nothing saved.
    let mut bad = ontology();
    bad["terms"][1]["aliases"] = json!(["leap 1a"]);
    let (status, body) = ws.put(&ws.route(""), bad).await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
    assert!(body.to_string().contains("already names"), "{body}");

    let (status, body) = ws.put(&ws.route(""), ontology()).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["counts"]["terms"], 2);
    assert_eq!(body["counts"]["aliases"], 2);

    let (_, workspace) = ws.get(&format!("/api/v1/workspaces/{}", ws.id)).await;
    assert_eq!(
        workspace["entity_types"],
        json!(["ENGINE", "PRODUCT", "COMPONENT"]),
        "{workspace}"
    );
    assert_eq!(workspace["relation_types"], json!(["PART_OF"]));
    assert_eq!(
        workspace["relation_edges"],
        json!([{"source": "COMPONENT", "relation": "PART_OF", "target": "ENGINE"}])
    );
    assert!(workspace["kg_schema_preset"].is_null(), "{workspace}");

    // Saving again keeps the first snapshot (restore goes back to the pre-ontology lists).
    let (status, _) = ws.put(&ws.route(""), ontology()).await;
    assert_eq!(status, StatusCode::OK);

    let (status, body) = ws.delete(&ws.route("")).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (_, workspace) = ws.get(&format!("/api/v1/workspaces/{}", ws.id)).await;
    assert_eq!(
        workspace["entity_types"],
        json!(["PERSON", "ORGANIZATION", "PRODUCT", "ARTIFACT"]),
        "{workspace}"
    );
    assert_eq!(workspace["kg_schema_preset"], "manufacturing");
    assert!(
        workspace["relation_types"]
            .as_array()
            .map_or(true, |a| a.is_empty()),
        "{workspace}"
    );
    let (_, body) = ws.get(&ws.route("")).await;
    assert!(body["ontology"].is_null());
}

#[tokio::test]
async fn spec184_draft_reads_tables_without_a_model() {
    let ws = Ws::new().await;
    let csv = "Terme;Catégorie;Synonymes;Définition\nLEAP-1A;Moteur;LEAP 1A|Leap1A;Moteur de l'A320neo\nCFM56;Moteur;;\n";
    let (status, body) = ws
        .post(
            &ws.route("/draft"),
            json!({"text": csv, "file_name": "referentiel.csv"}),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["format"], "table");
    assert_eq!(body["counts"]["terms"], 2);
    assert_eq!(body["ontology"]["entity_types"][0]["id"], "MOTEUR");
    assert_eq!(body["ontology"]["name"], "referentiel");
    // Nothing saved by a draft.
    let (_, current) = ws.get(&ws.route("")).await;
    assert!(current["ontology"].is_null());

    let (status, _) = ws.post(&ws.route("/draft"), json!({"text": "  "})).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn spec184_plan_and_realign_fold_aliases_and_retype_terms() {
    let ws = Ws::new().await;
    for (name, ty) in [
        ("LEAP 1A", "PRODUCT"),
        ("Leap1A", "PRODUCT"),
        ("CFM56", "ARTIFACT"),
        ("Airbus", "ORGANIZATION"),
    ] {
        let (status, body) = ws
            .post(
                "/api/v1/graph/entities",
                json!({"entity_name": name, "entity_type": ty, "description": format!("{name} from a document."), "source_id": "manual_entry"}),
            )
            .await;
        assert!(status.is_success(), "{name}: {body}");
    }
    let (status, body) = ws
        .post(
            "/api/v1/graph/relationships",
            json!({"src_id": "LEAP 1A", "tgt_id": "Airbus", "keywords": "POWERS", "description": "powers aircraft", "source_id": "manual_entry"}),
        )
        .await;
    let relationship_created = status.is_success();
    if !relationship_created {
        eprintln!("relationship create unavailable in this state ({status}): {body}");
    }

    // No ontology yet: plan explains, realign refuses.
    let (status, _) = ws.post(&ws.route("/plan"), json!({})).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    // A candidate ontology can be planned before it is saved.
    let (status, body) = ws
        .post(&ws.route("/plan"), json!({"ontology": ontology()}))
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(
        body["plan"]["merges"].as_array().unwrap().len(),
        2,
        "{body}"
    );
    assert_eq!(body["plan"]["retypes"][0]["entity"], "CFM56");
    assert_eq!(body["plan"]["terms_in_graph"], 2);

    let (status, _) = ws.put(&ws.route(""), ontology()).await;
    assert_eq!(status, StatusCode::OK);
    let (status, body) = ws.post(&ws.route("/realign"), json!({})).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["failed"], 0, "{body}");
    assert_eq!(body["merged"], 2, "{body}");
    assert_eq!(body["retyped"], 1, "{body}");
    assert_eq!(body["remaining"], 0);

    let (status, entity) = ws.get("/api/v1/graph/entities/LEAP-1A").await;
    assert_eq!(status, StatusCode::OK, "{entity}");
    assert_eq!(entity["entity"]["entity_type"], "ENGINE", "{entity}");
    assert!(
        entity["entity"]["description"]
            .as_str()
            .unwrap()
            .contains("Leap1A from a document."),
        "{entity}"
    );
    let (status, _) = ws.get("/api/v1/graph/entities/LEAP1A").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (_, cfm) = ws.get("/api/v1/graph/entities/CFM56").await;
    assert_eq!(cfm["entity"]["entity_type"], "ENGINE", "{cfm}");
    if relationship_created {
        let (_, hood) = ws.get("/api/v1/graph/entities/LEAP-1A/neighborhood").await;
        assert!(
            hood.to_string().contains("AIRBUS"),
            "relationship moved to the canonical entity: {hood}"
        );
    }

    // Idempotent: nothing left to do.
    let (_, body) = ws.post(&ws.route("/plan"), json!({})).await;
    assert!(
        body["plan"]["merges"].as_array().unwrap().is_empty(),
        "{body}"
    );
    assert!(
        body["plan"]["retypes"].as_array().unwrap().is_empty(),
        "{body}"
    );
}

#[tokio::test]
async fn spec184_path_must_be_the_selected_workspace() {
    let ws = Ws::new().await;
    let other = uuid::Uuid::new_v4();
    let (status, _) = put_with_tenant(
        &ws.app,
        &format!("/api/v1/workspaces/{}/ontology", ws.id),
        &ontology(),
        &ws.tenant,
        USER,
        &other.to_string(),
    )
    .await;
    assert!(
        status == StatusCode::BAD_REQUEST
            || status == StatusCode::FORBIDDEN
            || status == StatusCode::NOT_FOUND,
        "{status}"
    );
}
