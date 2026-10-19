//! Accept-token generation. Same 256-bit URL-safe code + SHA-256 storage as the SSO handoff.

pub use crate::services::federation::handoff::{hash_code as hash_token, random_code as new_token};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokens_are_unique_urlsafe_and_hash_deterministically() {
        let (a, b) = (new_token(), new_token());
        assert_ne!(a, b);
        assert!(
            a.len() >= 43
                && a.chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        );
        assert_eq!(hash_token(&a), hash_token(&a));
        assert_ne!(hash_token(&a), hash_token(&b));
        assert_ne!(hash_token(&a), a, "the stored value never equals the token");
    }
}
