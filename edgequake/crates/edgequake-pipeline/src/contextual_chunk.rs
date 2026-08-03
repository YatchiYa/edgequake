//! SPEC-091 RM2 — contextual chunk preamble for embedding (RM-AC-08).

pub const CONTEXTUAL_CHUNK_ENV: &str = "EDGEQUAKE_CONTEXTUAL_CHUNK";
/// Cap preamble chars to bound embed token cost.
pub const MAX_PREAMBLE_CHARS: usize = 512;

pub fn contextual_chunk_enabled() -> bool {
    matches!(
        std::env::var(CONTEXTUAL_CHUNK_ENV)
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase()
            .as_str(),
        "on" | "1" | "true" | "yes"
    )
}

/// Text passed to the embedding model: optional preamble + content.
pub fn embedding_input(content: &str, context_preamble: Option<&str>) -> String {
    if !contextual_chunk_enabled() {
        return content.to_string();
    }
    let Some(pre) = context_preamble.map(str::trim).filter(|s| !s.is_empty()) else {
        return content.to_string();
    };
    let capped: String = pre.chars().take(MAX_PREAMBLE_CHARS).collect();
    format!("{capped}\n\n{content}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contract_spec091_contextual_preamble_flag_changes_input() {
        std::env::remove_var(CONTEXTUAL_CHUNK_ENV);
        assert_eq!(
            embedding_input("body", Some("About Acme Corp")),
            "body",
            "default off leaves content unchanged"
        );
        std::env::set_var(CONTEXTUAL_CHUNK_ENV, "on");
        let with = embedding_input("body", Some("About Acme Corp"));
        assert!(with.contains("About Acme Corp"));
        assert!(with.contains("body"));
        assert_ne!(with, "body");
        std::env::remove_var(CONTEXTUAL_CHUNK_ENV);
    }
}
