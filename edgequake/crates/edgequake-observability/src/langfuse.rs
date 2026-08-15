//! Langfuse configuration (SPEC-124) — env-only secrets, pure URL/auth helpers.
//!
//! LAW-124-2: never expose secret values via API. This module only stores
//! presence flags and non-secret base URL for status DTOs.

use std::collections::HashMap;

/// Default Langfuse EU cloud host (no trailing slash).
pub const DEFAULT_LANGFUSE_BASE_URL: &str = "https://cloud.langfuse.com";

/// Resolved Langfuse observability settings (no secret material).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LangfuseConfig {
    /// Export should be attempted (keys present + not force-disabled).
    pub enabled: bool,
    /// Non-secret UI / OTLP base URL (no trailing slash).
    pub base_url: String,
    pub public_key_configured: bool,
    pub secret_key_configured: bool,
    /// Convenience: same as `base_url` for Settings "Open in Langfuse".
    pub ui_url: String,
}

impl Default for LangfuseConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            base_url: DEFAULT_LANGFUSE_BASE_URL.to_string(),
            public_key_configured: false,
            secret_key_configured: false,
            ui_url: DEFAULT_LANGFUSE_BASE_URL.to_string(),
        }
    }
}

impl LangfuseConfig {
    /// Load from environment. Secrets are read only to decide presence / enablement
    /// and (behind `otel`) to build the Authorization header — they are not stored.
    pub fn from_env() -> Self {
        let public = std::env::var("LANGFUSE_PUBLIC_KEY")
            .ok()
            .map(|v| unquote_env_value(&v))
            .filter(|v| !v.is_empty());
        let secret = std::env::var("LANGFUSE_SECRET_KEY")
            .ok()
            .map(|v| unquote_env_value(&v))
            .filter(|v| !v.is_empty());

        let public_key_configured = public.is_some();
        let secret_key_configured = secret.is_some();

        let base_url = normalize_base_url(
            std::env::var("LANGFUSE_BASE_URL")
                .ok()
                .map(|v| unquote_env_value(&v))
                .filter(|v| !v.is_empty())
                .or_else(|| {
                    std::env::var("LANGFUSE_HOST")
                        .ok()
                        .map(|v| unquote_env_value(&v))
                        .filter(|v| !v.is_empty())
                })
                .as_deref()
                .unwrap_or(DEFAULT_LANGFUSE_BASE_URL),
        );

        let force = std::env::var("EDGEQUAKE_LANGFUSE_ENABLED")
            .ok()
            .map(|v| unquote_env_value(&v));
        let force_off = force
            .as_deref()
            .map(|v| {
                let t = v.trim();
                t == "0" || t.eq_ignore_ascii_case("false") || t.eq_ignore_ascii_case("off")
            })
            .unwrap_or(false);
        let force_on = force
            .as_deref()
            .map(|v| {
                let t = v.trim();
                t == "1" || t.eq_ignore_ascii_case("true") || t.eq_ignore_ascii_case("on")
            })
            .unwrap_or(false);

        let keys_ok = public_key_configured && secret_key_configured;
        let enabled = if force_off {
            false
        } else if force_on {
            keys_ok
        } else {
            keys_ok
        };

        Self {
            enabled,
            ui_url: base_url.clone(),
            base_url,
            public_key_configured,
            secret_key_configured,
        }
    }

    /// Full OTLP/HTTP traces URL for programmatic `with_endpoint`.
    ///
    /// opentelemetry-otlp 0.32 does **not** append `/v1/traces` when the endpoint is set
    /// via the builder (unlike `OTEL_EXPORTER_OTLP_ENDPOINT` env). Langfuse only accepts
    /// spans at `/api/public/otel/v1/traces` (root `/api/public/otel` returns 404).
    pub fn otlp_endpoint(&self) -> String {
        format!(
            "{}/api/public/otel/v1/traces",
            self.base_url.trim_end_matches('/')
        )
    }

    /// Deep link to a single trace in the Langfuse UI.
    ///
    /// Prefer [`session_ui_url`] for operator deep-links until API `trace_id`
    /// is unified with the OTEL TraceId (SPEC-124 residual).
    pub fn trace_ui_url(&self, trace_id: &str) -> String {
        format!("{}/trace/{}", self.base_url.trim_end_matches('/'), trace_id)
    }

    /// Deep link to a Langfuse Session (honest operator link for SPEC-124).
    pub fn session_ui_url(&self, session_id: &str) -> String {
        format!(
            "{}/sessions/{}",
            self.base_url.trim_end_matches('/'),
            percent_encode_path_segment(session_id)
        )
    }

    /// Operator env snippet (placeholders only — never real secrets).
    pub fn env_snippet(&self) -> String {
        format!(
            "# Recommended: add to repo-root .env (make dev sources it)\n\
             LANGFUSE_PUBLIC_KEY=pk-lf-...\n\
             LANGFUSE_SECRET_KEY=sk-lf-...\n\
             LANGFUSE_BASE_URL={}\n\
             # Or export in the same shell, then restart make dev\n\
             # (otel is on by default since SPEC-124)",
            self.base_url
        )
    }

    /// Config requirement rows for Settings UI (no secret values).
    pub fn config_requirements(&self) -> Vec<LangfuseConfigRequirement> {
        vec![
            LangfuseConfigRequirement {
                name: "LANGFUSE_PUBLIC_KEY".into(),
                satisfied: self.public_key_configured,
            },
            LangfuseConfigRequirement {
                name: "LANGFUSE_SECRET_KEY".into(),
                satisfied: self.secret_key_configured,
            },
            LangfuseConfigRequirement {
                name: "LANGFUSE_BASE_URL".into(),
                satisfied: true, // always has default
            },
        ]
    }
}

/// One env requirement for Settings / OpenAPI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LangfuseConfigRequirement {
    pub name: String,
    pub satisfied: bool,
}

/// Strip surrounding quotes that GNU Make leaves when `-include`ing a quoted `.env`.
///
/// Bash `. .env` strips quotes; Make does not. Without this, OTLP Basic auth
/// embeds `"` in the key material and Langfuse returns HTTP 401 while Settings
/// still reports keys as configured.
pub fn unquote_env_value(raw: &str) -> String {
    let t = raw.trim();
    let bytes = t.as_bytes();
    if bytes.len() >= 2 {
        let first = bytes[0];
        let last = bytes[bytes.len() - 1];
        if (first == b'"' && last == b'"') || (first == b'\'' && last == b'\'') {
            return t[1..t.len() - 1].to_string();
        }
    }
    t.to_string()
}

/// Strip trailing slashes (and Make/dotenv quotes) from a Langfuse base URL.
pub fn normalize_base_url(raw: &str) -> String {
    unquote_env_value(raw).trim_end_matches('/').to_string()
}

/// Build Basic auth + Langfuse v4 ingestion headers from live env keys.
///
/// Returns `None` if either key is missing. Does not log key material.
pub fn langfuse_otlp_headers_from_env() -> Option<HashMap<String, String>> {
    let public = unquote_env_value(&std::env::var("LANGFUSE_PUBLIC_KEY").ok()?);
    let secret = unquote_env_value(&std::env::var("LANGFUSE_SECRET_KEY").ok()?);
    if public.is_empty() || secret.is_empty() {
        return None;
    }
    Some(langfuse_otlp_headers(&public, &secret))
}

/// Pure header builder (testable).
pub fn langfuse_otlp_headers(public_key: &str, secret_key: &str) -> HashMap<String, String> {
    let token = base64_encode(format!("{public_key}:{secret_key}").as_bytes());
    let mut headers = HashMap::new();
    headers.insert("Authorization".into(), format!("Basic {token}"));
    headers.insert("x-langfuse-ingestion-version".into(), "4".into());
    headers
}

/// Minimal Base64 (RFC 4648) encoder — avoids a new workspace dep for one header.
fn base64_encode(input: &[u8]) -> String {
    const TABLE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(((input.len() + 2) / 3) * 4);
    let mut i = 0;
    while i + 3 <= input.len() {
        let n = ((input[i] as u32) << 16) | ((input[i + 1] as u32) << 8) | (input[i + 2] as u32);
        out.push(TABLE[((n >> 18) & 0x3F) as usize] as char);
        out.push(TABLE[((n >> 12) & 0x3F) as usize] as char);
        out.push(TABLE[((n >> 6) & 0x3F) as usize] as char);
        out.push(TABLE[(n & 0x3F) as usize] as char);
        i += 3;
    }
    match input.len() - i {
        1 => {
            let n = (input[i] as u32) << 16;
            out.push(TABLE[((n >> 18) & 0x3F) as usize] as char);
            out.push(TABLE[((n >> 12) & 0x3F) as usize] as char);
            out.push('=');
            out.push('=');
        }
        2 => {
            let n = ((input[i] as u32) << 16) | ((input[i + 1] as u32) << 8);
            out.push(TABLE[((n >> 18) & 0x3F) as usize] as char);
            out.push(TABLE[((n >> 12) & 0x3F) as usize] as char);
            out.push(TABLE[((n >> 6) & 0x3F) as usize] as char);
            out.push('=');
        }
        _ => {}
    }
    out
}

/// Minimal path-segment percent-encoding (RFC 3986 unreserved left alone).
fn percent_encode_path_segment(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                out.push(b as char);
            }
            _ => {
                out.push('%');
                out.push(char::from(b"0123456789ABCDEF"[(b >> 4) as usize]));
                out.push(char::from(b"0123456789ABCDEF"[(b & 0x0F) as usize]));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    /// Env mutation is process-global — serialize Langfuse config tests.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn with_env_vars(pairs: &[(&str, Option<&str>)], f: impl FnOnce()) {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let previous: Vec<(String, Option<String>)> = pairs
            .iter()
            .map(|(k, _)| ((*k).to_string(), std::env::var(k).ok()))
            .collect();
        for (k, v) in pairs {
            match v {
                Some(val) => std::env::set_var(k, val),
                None => std::env::remove_var(k),
            }
        }
        f();
        for (k, prev) in previous {
            match prev {
                Some(val) => std::env::set_var(&k, val),
                None => std::env::remove_var(&k),
            }
        }
    }

    #[test]
    fn unquote_strips_matching_double_or_single_quotes() {
        assert_eq!(unquote_env_value("  \"pk-lf-x\"  "), "pk-lf-x");
        assert_eq!(unquote_env_value("'sk-lf-y'"), "sk-lf-y");
        assert_eq!(unquote_env_value("pk-lf-plain"), "pk-lf-plain");
        // Mismatched quotes are left alone.
        assert_eq!(unquote_env_value("\"pk-lf-x'"), "\"pk-lf-x'");
    }

    #[test]
    fn normalize_strips_trailing_slash() {
        assert_eq!(
            normalize_base_url("https://cloud.langfuse.com/"),
            "https://cloud.langfuse.com"
        );
    }

    #[test]
    fn normalize_strips_make_quotes_and_slash() {
        assert_eq!(
            normalize_base_url("\"https://us.cloud.langfuse.com/\""),
            "https://us.cloud.langfuse.com"
        );
    }

    #[test]
    fn otlp_endpoint_joins_path() {
        let cfg = LangfuseConfig {
            enabled: true,
            base_url: "https://cloud.langfuse.com".into(),
            public_key_configured: true,
            secret_key_configured: true,
            ui_url: "https://cloud.langfuse.com".into(),
        };
        assert_eq!(
            cfg.otlp_endpoint(),
            "https://cloud.langfuse.com/api/public/otel/v1/traces"
        );
        assert_eq!(
            cfg.trace_ui_url("abc123"),
            "https://cloud.langfuse.com/trace/abc123"
        );
        assert_eq!(
            cfg.session_ui_url("conv-1"),
            "https://cloud.langfuse.com/sessions/conv-1"
        );
        assert_eq!(
            cfg.session_ui_url("a/b"),
            "https://cloud.langfuse.com/sessions/a%2Fb"
        );
    }

    #[test]
    fn base64_known_vector() {
        // echo -n "pk:sk" | base64
        assert_eq!(base64_encode(b"pk:sk"), "cGs6c2s=");
    }

    #[test]
    fn headers_include_basic_and_v4() {
        let h = langfuse_otlp_headers("pk", "sk");
        assert_eq!(
            h.get("Authorization").map(String::as_str),
            Some("Basic cGs6c2s=")
        );
        assert_eq!(
            h.get("x-langfuse-ingestion-version").map(String::as_str),
            Some("4")
        );
    }

    #[test]
    fn from_env_disabled_without_keys() {
        with_env_vars(
            &[
                ("LANGFUSE_PUBLIC_KEY", None),
                ("LANGFUSE_SECRET_KEY", None),
                ("LANGFUSE_BASE_URL", None),
                ("LANGFUSE_HOST", None),
                ("EDGEQUAKE_LANGFUSE_ENABLED", None),
            ],
            || {
                let cfg = LangfuseConfig::from_env();
                assert!(!cfg.enabled);
                assert!(!cfg.public_key_configured);
                assert_eq!(cfg.base_url, DEFAULT_LANGFUSE_BASE_URL);
            },
        );
    }

    #[test]
    fn from_env_partial_keys_disabled() {
        with_env_vars(
            &[
                ("LANGFUSE_PUBLIC_KEY", Some("pk-lf-x")),
                ("LANGFUSE_SECRET_KEY", None),
                ("EDGEQUAKE_LANGFUSE_ENABLED", None),
            ],
            || {
                let cfg = LangfuseConfig::from_env();
                assert!(!cfg.enabled);
                assert!(cfg.public_key_configured);
                assert!(!cfg.secret_key_configured);
            },
        );
    }

    #[test]
    fn from_env_both_keys_enabled() {
        with_env_vars(
            &[
                ("LANGFUSE_PUBLIC_KEY", Some("pk-lf-x")),
                ("LANGFUSE_SECRET_KEY", Some("sk-lf-y")),
                ("LANGFUSE_BASE_URL", Some("https://us.cloud.langfuse.com/")),
                ("EDGEQUAKE_LANGFUSE_ENABLED", None),
            ],
            || {
                let cfg = LangfuseConfig::from_env();
                assert!(cfg.enabled);
                assert_eq!(cfg.base_url, "https://us.cloud.langfuse.com");
                assert_eq!(cfg.ui_url, cfg.base_url);
            },
        );
    }

    #[test]
    fn from_env_make_quoted_keys_enable_and_auth_matches_unquoted() {
        // GNU Make `-include .env` keeps surrounding quotes in the value.
        with_env_vars(
            &[
                ("LANGFUSE_PUBLIC_KEY", Some("\"pk-lf-x\"")),
                ("LANGFUSE_SECRET_KEY", Some("\"sk-lf-y\"")),
                (
                    "LANGFUSE_BASE_URL",
                    Some("\"https://us.cloud.langfuse.com\""),
                ),
                ("EDGEQUAKE_LANGFUSE_ENABLED", None),
            ],
            || {
                let cfg = LangfuseConfig::from_env();
                assert!(cfg.enabled);
                assert!(cfg.public_key_configured);
                assert!(cfg.secret_key_configured);
                assert_eq!(cfg.base_url, "https://us.cloud.langfuse.com");
                assert_eq!(
                    cfg.otlp_endpoint(),
                    "https://us.cloud.langfuse.com/api/public/otel/v1/traces"
                );
                let h = langfuse_otlp_headers_from_env().expect("headers");
                assert_eq!(
                    h.get("Authorization").map(String::as_str),
                    Some(langfuse_otlp_headers("pk-lf-x", "sk-lf-y")["Authorization"].as_str())
                );
            },
        );
    }

    #[test]
    fn from_env_host_alias() {
        with_env_vars(
            &[
                ("LANGFUSE_PUBLIC_KEY", Some("pk")),
                ("LANGFUSE_SECRET_KEY", Some("sk")),
                ("LANGFUSE_BASE_URL", None),
                ("LANGFUSE_HOST", Some("http://localhost:3000")),
                ("EDGEQUAKE_LANGFUSE_ENABLED", None),
            ],
            || {
                let cfg = LangfuseConfig::from_env();
                assert_eq!(cfg.base_url, "http://localhost:3000");
            },
        );
    }

    #[test]
    fn force_off_disables_with_keys() {
        with_env_vars(
            &[
                ("LANGFUSE_PUBLIC_KEY", Some("pk")),
                ("LANGFUSE_SECRET_KEY", Some("sk")),
                ("EDGEQUAKE_LANGFUSE_ENABLED", Some("0")),
            ],
            || {
                let cfg = LangfuseConfig::from_env();
                assert!(!cfg.enabled);
            },
        );
    }

    #[test]
    fn env_snippet_has_no_real_secrets() {
        let cfg = LangfuseConfig::from_env();
        let s = cfg.env_snippet();
        assert!(s.contains("pk-lf-..."));
        assert!(!s.contains("sk-lf-y"));
    }
}
