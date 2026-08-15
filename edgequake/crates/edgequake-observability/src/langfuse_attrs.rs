//! Langfuse + OTEL GenAI trace-identity attributes (SPEC-124 sessions).
//!
//! DRY SSOT for attribute keys. Never synthesize session ids — callers must
//! pass a durable product id (`conversation_id`) or leave session unset.

use crate::langfuse::unquote_env_value;

/// Langfuse session id (preferred by Langfuse OTEL mapper).
pub const LANGFUSE_SESSION_ID: &str = "langfuse.session.id";
/// OpenInference / generic session alias (Langfuse also accepts).
pub const SESSION_ID: &str = "session.id";
/// OTEL GenAI v1.37 conversation/thread id (conditionally required when available).
pub const GEN_AI_CONVERSATION_ID: &str = "gen_ai.conversation.id";

pub const LANGFUSE_USER_ID: &str = "langfuse.user.id";
pub const USER_ID: &str = "user.id";

pub const LANGFUSE_META_TENANT_ID: &str = "langfuse.trace.metadata.tenant_id";
pub const LANGFUSE_META_WORKSPACE_ID: &str = "langfuse.trace.metadata.workspace_id";
/// LAW-124-19: human slug alongside UUID — never replace `*_id`.
pub const LANGFUSE_META_TENANT_SLUG: &str = "langfuse.trace.metadata.tenant_slug";
pub const LANGFUSE_META_WORKSPACE_SLUG: &str = "langfuse.trace.metadata.workspace_slug";

/// Prefix for filterable trace metadata (LAW-124-20).
pub const LANGFUSE_TRACE_METADATA_PREFIX: &str = "langfuse.trace.metadata.";
/// Prefix for filterable observation metadata (LAW-124-20).
pub const LANGFUSE_OBSERVATION_METADATA_PREFIX: &str = "langfuse.observation.metadata.";
/// Langfuse propagate/filter value cap.
pub const LANGFUSE_METADATA_VALUE_MAX_CHARS: usize = 200;

/// OTEL GenAI usage (integers). LAW-124-12: emit tokens; never cost.
pub const GEN_AI_USAGE_INPUT_TOKENS: &str = "gen_ai.usage.input_tokens";
pub const GEN_AI_USAGE_OUTPUT_TOKENS: &str = "gen_ai.usage.output_tokens";

pub const LANGFUSE_OBSERVATION_TYPE: &str = "langfuse.observation.type";
pub const LANGFUSE_OBSERVATION_INPUT: &str = "langfuse.observation.input";
pub const LANGFUSE_OBSERVATION_OUTPUT: &str = "langfuse.observation.output";
pub const LANGFUSE_TRACE_TAGS: &str = "langfuse.trace.tags";

/// GenAI ecosystem aliases (Langfuse also maps these to observation I/O).
pub const GEN_AI_PROMPT: &str = "gen_ai.prompt";
pub const GEN_AI_COMPLETION: &str = "gen_ai.completion";

pub const OBSERVATION_TYPE_GENERATION: &str = "generation";
pub const OBSERVATION_TYPE_RETRIEVER: &str = "retriever";
pub const OBSERVATION_TYPE_EMBEDDING: &str = "embedding";
pub const OBSERVATION_TYPE_CHAIN: &str = "chain";
pub const OBSERVATION_TYPE_SPAN: &str = "span";

/// LAW-124-12: never emit these attribute keys (Langfuse cost ingestion).
pub const COST_ATTR_DENYLIST: &[&str] = &[
    "gen_ai.usage.cost",
    "langfuse.observation.cost_details",
    "langfuse.observation.cost",
];

pub fn is_forbidden_cost_attr(key: &str) -> bool {
    COST_ATTR_DENYLIST.contains(&key)
}

/// Allowlisted baggage keys copied onto every span (security: no arbitrary baggage).
pub const LANGFUSE_BAGGAGE_ALLOWLIST: &[&str] = &[
    LANGFUSE_SESSION_ID,
    SESSION_ID,
    GEN_AI_CONVERSATION_ID,
    LANGFUSE_USER_ID,
    USER_ID,
    LANGFUSE_META_TENANT_ID,
    LANGFUSE_META_WORKSPACE_ID,
    LANGFUSE_META_TENANT_SLUG,
    LANGFUSE_META_WORKSPACE_SLUG,
    LANGFUSE_TRACE_TAGS,
];

/// Normalized identity for one request / chat turn.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LangfuseTraceIdentity {
    pub session_id: Option<String>,
    pub user_id: Option<String>,
    pub tenant_id: Option<String>,
    pub workspace_id: Option<String>,
    pub tenant_slug: Option<String>,
    pub workspace_slug: Option<String>,
}

impl LangfuseTraceIdentity {
    /// Build from optional raw strings. Empty / whitespace-only values are dropped
    /// (OTEL GenAI: do not invent conversation ids).
    pub fn from_parts(
        session_id: Option<&str>,
        user_id: Option<&str>,
        tenant_id: Option<&str>,
        workspace_id: Option<&str>,
    ) -> Self {
        Self {
            session_id: normalize_id(session_id),
            user_id: normalize_id(user_id),
            tenant_id: normalize_id(tenant_id),
            workspace_id: normalize_id(workspace_id),
            tenant_slug: None,
            workspace_slug: None,
        }
    }

    /// Additive slugs (LAW-124-19). Blank values omitted; never copied into `*_id`.
    pub fn with_slugs(mut self, tenant_slug: Option<&str>, workspace_slug: Option<&str>) -> Self {
        self.tenant_slug = normalize_id(tenant_slug);
        self.workspace_slug = normalize_id(workspace_slug);
        self
    }

    pub fn is_empty(&self) -> bool {
        self.session_id.is_none()
            && self.user_id.is_none()
            && self.tenant_id.is_none()
            && self.workspace_id.is_none()
            && self.tenant_slug.is_none()
            && self.workspace_slug.is_none()
    }

    /// Flat `(key, value)` pairs for OTEL attributes / baggage.
    pub fn key_values(&self) -> Vec<(&'static str, String)> {
        let mut out = Vec::with_capacity(8);
        if let Some(ref sid) = self.session_id {
            out.push((LANGFUSE_SESSION_ID, sid.clone()));
            out.push((SESSION_ID, sid.clone()));
            out.push((GEN_AI_CONVERSATION_ID, sid.clone()));
        }
        if let Some(ref uid) = self.user_id {
            out.push((LANGFUSE_USER_ID, uid.clone()));
            out.push((USER_ID, uid.clone()));
        }
        if let Some(ref tid) = self.tenant_id {
            out.push((LANGFUSE_META_TENANT_ID, tid.clone()));
        }
        if let Some(ref wid) = self.workspace_id {
            out.push((LANGFUSE_META_WORKSPACE_ID, wid.clone()));
        }
        if let Some(ref ts) = self.tenant_slug {
            out.push((LANGFUSE_META_TENANT_SLUG, ts.clone()));
        }
        if let Some(ref ws) = self.workspace_slug {
            out.push((LANGFUSE_META_WORKSPACE_SLUG, ws.clone()));
        }
        out
    }
}

fn normalize_id(raw: Option<&str>) -> Option<String> {
    raw.map(unquote_env_value)
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

pub fn is_allowlisted_baggage_key(key: &str) -> bool {
    LANGFUSE_BAGGAGE_ALLOWLIST.contains(&key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_session_emits_no_session_keys() {
        let id = LangfuseTraceIdentity::from_parts(None, Some("u1"), None, None);
        let keys: Vec<_> = id.key_values().into_iter().map(|(k, _)| k).collect();
        assert!(!keys.contains(&LANGFUSE_SESSION_ID));
        assert!(!keys.contains(&GEN_AI_CONVERSATION_ID));
        assert!(keys.contains(&LANGFUSE_USER_ID));
    }

    #[test]
    fn blank_session_not_synthesized() {
        let id = LangfuseTraceIdentity::from_parts(Some("  "), Some("u"), None, None);
        assert!(id.session_id.is_none());
    }

    #[test]
    fn session_co_emits_langfuse_and_genai() {
        let id = LangfuseTraceIdentity::from_parts(
            Some("\"550e8400-e29b-41d4-a716-446655440000\""),
            Some("user-1"),
            Some("tenant-1"),
            Some("ws-1"),
        );
        assert_eq!(
            id.session_id.as_deref(),
            Some("550e8400-e29b-41d4-a716-446655440000")
        );
        let map: std::collections::HashMap<_, _> = id.key_values().into_iter().collect();
        assert_eq!(
            map.get(LANGFUSE_SESSION_ID).map(String::as_str),
            Some("550e8400-e29b-41d4-a716-446655440000")
        );
        assert_eq!(
            map.get(SESSION_ID).map(String::as_str),
            Some("550e8400-e29b-41d4-a716-446655440000")
        );
        assert_eq!(
            map.get(GEN_AI_CONVERSATION_ID).map(String::as_str),
            Some("550e8400-e29b-41d4-a716-446655440000")
        );
        assert_eq!(map.get(USER_ID).map(String::as_str), Some("user-1"));
        assert_eq!(
            map.get(LANGFUSE_META_TENANT_ID).map(String::as_str),
            Some("tenant-1")
        );
        assert_eq!(
            map.get(LANGFUSE_META_WORKSPACE_ID).map(String::as_str),
            Some("ws-1")
        );
        assert!(!map.contains_key(LANGFUSE_META_TENANT_SLUG));
        assert!(!map.contains_key(LANGFUSE_META_WORKSPACE_SLUG));
    }

    #[test]
    fn slugs_are_additive_never_replace_ids() {
        let id = LangfuseTraceIdentity::from_parts(
            None,
            None,
            Some("aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee"),
            Some("11111111-2222-3333-4444-555555555555"),
        )
        .with_slugs(Some("acme"), Some("kb-prod"));
        let map: std::collections::HashMap<_, _> = id.key_values().into_iter().collect();
        assert_eq!(
            map.get(LANGFUSE_META_TENANT_ID).map(String::as_str),
            Some("aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee")
        );
        assert_eq!(
            map.get(LANGFUSE_META_WORKSPACE_ID).map(String::as_str),
            Some("11111111-2222-3333-4444-555555555555")
        );
        assert_eq!(
            map.get(LANGFUSE_META_TENANT_SLUG).map(String::as_str),
            Some("acme")
        );
        assert_eq!(
            map.get(LANGFUSE_META_WORKSPACE_SLUG).map(String::as_str),
            Some("kb-prod")
        );
        assert_ne!(
            map.get(LANGFUSE_META_TENANT_ID),
            map.get(LANGFUSE_META_TENANT_SLUG)
        );
    }

    #[test]
    fn blank_slug_omitted() {
        let id = LangfuseTraceIdentity::from_parts(None, None, Some("tid"), Some("wid"))
            .with_slugs(Some("  "), None);
        let keys: Vec<_> = id.key_values().into_iter().map(|(k, _)| k).collect();
        assert!(keys.contains(&LANGFUSE_META_TENANT_ID));
        assert!(!keys.contains(&LANGFUSE_META_TENANT_SLUG));
        assert!(!keys.contains(&LANGFUSE_META_WORKSPACE_SLUG));
    }

    #[test]
    fn allowlist_is_closed() {
        assert!(is_allowlisted_baggage_key(LANGFUSE_SESSION_ID));
        assert!(!is_allowlisted_baggage_key("authorization"));
        assert!(!is_allowlisted_baggage_key("secret"));
    }

    #[test]
    fn cost_attrs_are_forbidden() {
        for key in COST_ATTR_DENYLIST {
            assert!(is_forbidden_cost_attr(key));
            assert!(!is_allowlisted_baggage_key(key));
        }
        assert!(!is_forbidden_cost_attr(GEN_AI_USAGE_INPUT_TOKENS));
        assert!(!is_forbidden_cost_attr(GEN_AI_USAGE_OUTPUT_TOKENS));
    }

    #[test]
    fn identity_key_values_never_include_cost() {
        let id =
            LangfuseTraceIdentity::from_parts(Some("sess"), Some("user"), Some("t"), Some("w"));
        for (k, _) in id.key_values() {
            assert!(
                !is_forbidden_cost_attr(k),
                "identity must not emit cost key {k}"
            );
        }
    }
}
