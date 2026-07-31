//! SPEC-094 contract: parse routes are registered in OpenAPI + routes.rs.

use std::fs;
use std::path::PathBuf;

fn read_api_src(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    fs::read_to_string(path).unwrap_or_else(|e| panic!("read {rel}: {e}"))
}

#[test]
fn spec094_parse_routes_registered() {
    let routes = read_api_src("src/routes.rs");
    assert!(routes.contains(r#""/parse""#));
    assert!(routes.contains(r#""/parse/backends""#));
    assert!(routes.contains(r#""/parse/jobs/{id}""#));

    let openapi = read_api_src("src/openapi.rs");
    assert!(openapi.contains("parse_document"));
    assert!(openapi.contains("list_parse_backends"));
    assert!(openapi.contains("get_parse_job"));

    let handler = read_api_src("src/handlers/parse/handler.rs");
    assert!(handler.contains("path = \"/api/v1/parse\""));
}
