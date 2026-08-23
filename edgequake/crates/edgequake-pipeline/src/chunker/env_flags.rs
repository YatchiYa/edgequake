//! Env kill-switches that default **on** when unset (SPEC-125 / SPEC-135).

/// `0` / `false` / `off` / `no` (any case) → false. Unset or any other value → true.
pub fn env_flag_default_on(raw: Option<&str>) -> bool {
    match raw {
        Some(v) => !matches!(
            v.trim().to_ascii_lowercase().as_str(),
            "0" | "false" | "off" | "no"
        ),
        None => true,
    }
}

/// Read `name` from the process environment (default ON).
pub fn env_flag_default_on_var(name: &str) -> bool {
    env_flag_default_on(std::env::var(name).ok().as_deref())
}

#[cfg(test)]
mod tests {
    use super::env_flag_default_on;

    #[test]
    fn default_on_unset_and_truthy() {
        assert!(env_flag_default_on(None));
        assert!(env_flag_default_on(Some("1")));
        assert!(env_flag_default_on(Some("true")));
        assert!(env_flag_default_on(Some("")));
    }

    #[test]
    fn default_on_falsy_tokens() {
        assert!(!env_flag_default_on(Some("0")));
        assert!(!env_flag_default_on(Some("false")));
        assert!(!env_flag_default_on(Some("OFF")));
        assert!(!env_flag_default_on(Some("no")));
    }
}
