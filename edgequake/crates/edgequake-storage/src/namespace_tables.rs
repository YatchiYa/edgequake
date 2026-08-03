//! Namespace → public relation naming SSOT (SPEC-104 LAW-I1).
//!
//! Always available (not gated on the `postgres` feature) so OpenAPI / non-PG
//! builds and PostgresConfig share one formula.

/// Map namespace to a safe identifier segment: non `[A-Za-z0-9_]` → `_`.
pub fn sanitize_namespace_segment(namespace: &str) -> String {
    namespace
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// Table prefix for a workspace namespace (`eq_{sanitized}`).
///
/// `namespace = "default"` → `eq_default`.
pub fn table_prefix_for_namespace(namespace: &str) -> String {
    format!("eq_{}", sanitize_namespace_segment(namespace))
}

/// AGE graph catalog name: `eq_{prefix}_graph`.
///
/// `namespace = "default"` → `eq_eq_default_graph`.
pub fn age_graph_name_for_namespace(namespace: &str) -> String {
    format!("eq_{}_graph", table_prefix_for_namespace(namespace))
}

/// Unqualified KV table: `eq_{prefix}_kv`.
pub fn bare_kv_table_for_namespace(namespace: &str) -> String {
    format!("eq_{}_kv", table_prefix_for_namespace(namespace))
}

/// Unqualified vectors table: `eq_{prefix}_vectors`.
pub fn bare_vectors_table_for_namespace(namespace: &str) -> String {
    format!("eq_{}_vectors", table_prefix_for_namespace(namespace))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_namespace_matches_historical_eq_eq_default() {
        assert_eq!(table_prefix_for_namespace("default"), "eq_default");
        assert_eq!(
            age_graph_name_for_namespace("default"),
            "eq_eq_default_graph"
        );
        assert_eq!(bare_kv_table_for_namespace("default"), "eq_eq_default_kv");
        assert_eq!(
            bare_vectors_table_for_namespace("default"),
            "eq_eq_default_vectors"
        );
    }

    #[test]
    fn hyphen_namespace_maps_to_underscore() {
        assert_eq!(table_prefix_for_namespace("my-ws"), "eq_my_ws");
        assert_eq!(age_graph_name_for_namespace("my-ws"), "eq_eq_my_ws_graph");
    }
}
