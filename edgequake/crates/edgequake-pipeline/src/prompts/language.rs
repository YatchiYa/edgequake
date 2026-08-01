//! Extraction output language helpers (SPEC-096 / GH-352).
//!
//! Keep language instruction wording and allowlist canonicalize here so
//! JSON primary + gleaning prompts share one SSOT (LAW-L2 / LAW-L3).

use super::SUPPORTED_LANGUAGES;

/// Default when workspace and env overrides are unset.
pub const DEFAULT_EXTRACTION_LANGUAGE: &str = "English";

/// Env var for fleet-wide extraction language default (LightRAG SUMMARY_LANGUAGE analogue).
pub const EXTRACTION_LANGUAGE_ENV: &str = "EDGEQUAKE_EXTRACTION_LANGUAGE";

/// Shared language instruction block for JSON extractors (LAW-L2, LAW-L4).
pub fn json_language_instruction(language: &str) -> String {
    format!(
        r#"## Output Language
- Write all natural-language string values (entity names, entity descriptions, relationship descriptions, and free-form relationship types) in **{language}**.
- Keep JSON object/array keys exactly as specified in English (`entities`, `relationships`, `name`, `type`, `description`, `source`, `target`).
- Proper nouns may be retained in their original form when translation would create ambiguity."#
    )
}

/// Case-insensitive match against [`SUPPORTED_LANGUAGES`]; returns canonical display name.
///
/// Whitespace-only and unknown values return `None`.
pub fn canonicalize_extraction_language(raw: &str) -> Option<&'static str> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    SUPPORTED_LANGUAGES
        .iter()
        .copied()
        .find(|lang| lang.eq_ignore_ascii_case(trimmed))
}

/// Whether the value clears a workspace override (`""` / `"none"`).
pub fn is_extraction_language_clear(raw: &str) -> bool {
    let t = raw.trim();
    t.is_empty() || t.eq_ignore_ascii_case("none")
}

/// Resolve effective extraction language (LAW-L3).
///
/// Precedence: workspace metadata override → env → `"English"`.
/// Invalid env values warn and fall through (no crash).
pub fn resolve_extraction_language(
    workspace_language: Option<&str>,
    env_language: Option<&str>,
) -> String {
    if let Some(ws) = workspace_language {
        let trimmed = ws.trim();
        if !trimmed.is_empty() {
            if let Some(canonical) = canonicalize_extraction_language(trimmed) {
                return canonical.to_string();
            }
            // Workspace values should already be validated at API; fall through safely.
            tracing::warn!(
                value = %trimmed,
                "Invalid workspace extraction_language; falling through to env/default"
            );
        }
    }

    if let Some(env) = env_language {
        let trimmed = env.trim();
        if !trimmed.is_empty() {
            if let Some(canonical) = canonicalize_extraction_language(trimmed) {
                return canonical.to_string();
            }
            tracing::warn!(
                value = %trimmed,
                env = EXTRACTION_LANGUAGE_ENV,
                "Unsupported {}; treating as unset → English",
                EXTRACTION_LANGUAGE_ENV
            );
        }
    }

    DEFAULT_EXTRACTION_LANGUAGE.to_string()
}

/// Read env and resolve with optional workspace override.
pub fn resolve_extraction_language_from_env(workspace_language: Option<&str>) -> String {
    let env = std::env::var(EXTRACTION_LANGUAGE_ENV).ok();
    resolve_extraction_language(workspace_language, env.as_deref())
}

/// Read `extraction_language` string from workspace metadata JSON map.
pub fn extraction_language_from_metadata(
    metadata: &std::collections::HashMap<String, serde_json::Value>,
) -> Option<String> {
    metadata
        .get("extraction_language")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spec096_json_language_instruction_mentions_language_and_english_keys() {
        let block = json_language_instruction("Chinese");
        assert!(block.contains("Chinese"));
        assert!(block.contains("entities"));
        assert!(block.contains("Proper nouns"));
    }

    #[test]
    fn spec096_canonicalize_language() {
        assert_eq!(canonicalize_extraction_language("chinese"), Some("Chinese"));
        assert_eq!(canonicalize_extraction_language("CHINESE"), Some("Chinese"));
        assert_eq!(canonicalize_extraction_language("Chinese"), Some("Chinese"));
        assert_eq!(canonicalize_extraction_language("ZH"), None);
        assert_eq!(canonicalize_extraction_language("Klingon"), None);
        assert_eq!(canonicalize_extraction_language("   "), None);
    }

    #[test]
    fn spec096_resolve_language_precedence() {
        assert_eq!(
            resolve_extraction_language(Some("Chinese"), Some("French")),
            "Chinese"
        );
        assert_eq!(
            resolve_extraction_language(None, Some("French")),
            "French"
        );
        assert_eq!(
            resolve_extraction_language(Some(""), Some("French")),
            "French"
        );
        assert_eq!(
            resolve_extraction_language(None, Some("Klingon")),
            "English"
        );
        assert_eq!(resolve_extraction_language(None, None), "English");
        assert_eq!(
            resolve_extraction_language(Some("japanese"), None),
            "Japanese"
        );
    }

    #[test]
    fn spec096_is_extraction_language_clear() {
        assert!(is_extraction_language_clear(""));
        assert!(is_extraction_language_clear("  "));
        assert!(is_extraction_language_clear("none"));
        assert!(is_extraction_language_clear("NONE"));
        assert!(!is_extraction_language_clear("Chinese"));
    }
}
