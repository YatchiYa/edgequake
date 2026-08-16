//! LLM temperature gating — re-exports edgequake-llm SSOT (SPEC-131).
//!
//! Product call sites may import from this module for historical paths;
//! the implementation lives in `edgequake_llm::temperature`.

pub use edgequake_llm::{effective_temperature_for_model, resolve_effective_temperature};

#[cfg(test)]
mod tests {
    use super::*;
    use edgequake_llm::ENV_OMIT_TEMPERATURE;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn clear_omit() {
        std::env::remove_var(ENV_OMIT_TEMPERATURE);
    }

    #[test]
    fn u131_01_omit_env_forces_none_for_gemma() {
        let _g = ENV_LOCK.lock().unwrap();
        clear_omit();
        std::env::set_var(ENV_OMIT_TEMPERATURE, "true");
        assert_eq!(
            resolve_effective_temperature("google.gemma-4-31b", 0.0),
            None
        );
        clear_omit();
    }

    #[test]
    fn u131_02_gemma_without_omit_sends_preferred() {
        let _g = ENV_LOCK.lock().unwrap();
        clear_omit();
        assert_eq!(
            resolve_effective_temperature("google.gemma-4-31b", 0.0),
            Some(0.0)
        );
        assert_eq!(
            resolve_effective_temperature("xai.grok-4.3", 0.0),
            Some(0.0)
        );
    }

    #[test]
    fn u131_03_gpt5_gate_still_omits_without_env() {
        let _g = ENV_LOCK.lock().unwrap();
        clear_omit();
        assert_eq!(resolve_effective_temperature("gpt-5-nano", 0.0), None);
    }

    #[test]
    fn u131_04_gpt4o_keeps_override_without_env() {
        let _g = ENV_LOCK.lock().unwrap();
        clear_omit();
        assert_eq!(resolve_effective_temperature("gpt-4o", 0.0), Some(0.0));
    }
}
