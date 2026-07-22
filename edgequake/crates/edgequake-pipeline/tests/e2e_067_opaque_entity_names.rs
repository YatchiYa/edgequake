//! 067 — E2E-style ingest fixture: UUID-shaped entity names must not enter the KG;
//! semantic names (Acme Corp) must survive tuple + JSON parse paths.

use edgequake_pipeline::prompts::{JsonExtractionParser, TupleParser};
use edgequake_storage::{is_opaque_identifier, EntityId};

/// Simulated SOTA LLM response over a chunk that mentions both a UUID resource
/// id and a real organization (common in agentic/API docs).
const TUPLE_FIXTURE: &str = r#"entity<|#|>84b69e27-e38b-444a-83dd-5e6a537c6f12<|#|>ORGANIZATION<|#|>Anthropic resource identifier from the API docs
entity<|#|>Acme Corp<|#|>ORGANIZATION<|#|>Customer organization in the case study
entity<|#|>Gabriel Greenfield<|#|>PERSON<|#|>Document author
relation<|#|>Gabriel Greenfield<|#|>Acme Corp<|#|>authored for,works with<|#|>Author associated with Acme Corp
relation<|#|>84b69e27-e38b-444a-83dd-5e6a537c6f12<|#|>Acme Corp<|#|>references<|#|>Must be dropped — opaque endpoint
<|COMPLETE|>"#;

const JSON_FIXTURE: &str = r#"{
  "entities": [
    {
      "name": "84b69e27-e38b-444a-83dd-5e6a537c6f12",
      "type": "ORGANIZATION",
      "description": "Opaque resource id"
    },
    {
      "name": "Acme Corp",
      "type": "ORGANIZATION",
      "description": "Customer organization"
    },
    {
      "name": "arn:aws:s3:::bucket/key",
      "type": "TECHNOLOGY",
      "description": "S3 ARN must not be an entity name"
    }
  ],
  "relationships": [
    {
      "source": "Acme Corp",
      "target": "84b69e27-e38b-444a-83dd-5e6a537c6f12",
      "type": "USES",
      "description": "dropped opaque target"
    }
  ]
}"#;

#[test]
fn e2e_tuple_fixture_keeps_acme_rejects_uuid() {
    let parser = TupleParser::new();
    let result = parser.parse(TUPLE_FIXTURE, "fixture-chunk-067").unwrap();

    assert!(
        result.entities.iter().any(|e| e.name == "ACME_CORP"),
        "expected ACME_CORP, got {:?}",
        result.entities
    );
    assert!(
        result
            .entities
            .iter()
            .any(|e| e.name == "GABRIEL_GREENFIELD"),
        "expected person, got {:?}",
        result.entities
    );
    assert!(
        result
            .entities
            .iter()
            .all(|e| !is_opaque_identifier(&e.name)),
        "no opaque entity names: {:?}",
        result.entities
    );
    assert_eq!(result.entities.len(), 2);
    assert_eq!(result.relationships.len(), 1);
    assert_eq!(result.relationships[0].source, "GABRIEL_GREENFIELD");
    assert_eq!(result.relationships[0].target, "ACME_CORP");
}

#[test]
fn e2e_json_fixture_keeps_acme_rejects_uuid_and_arn() {
    let parser = JsonExtractionParser::new();
    let result = parser
        .parse(JSON_FIXTURE, "fixture-chunk-067-json")
        .unwrap();

    assert_eq!(result.entities.len(), 1);
    assert_eq!(result.entities[0].name, "ACME_CORP");
    assert!(result.relationships.is_empty());
}

#[test]
fn e2e_entity_id_new_rejects_uuid_keeps_mm_im() {
    assert!(EntityId::new("84b69e27-e38b-444a-83dd-5e6a537c6f12").is_empty());
    assert!(!EntityId::new("Acme Corp").is_empty());
    let mm = "im-019f7028-d3e3-7684-8b3b-a9259368329a-page-0002-fig-01";
    assert!(!EntityId::new(mm).is_empty());
}
