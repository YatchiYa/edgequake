#[cfg(feature = "postgres")]
mod tests {
    use crate::type_list::normalize_type_list;

    #[test]
    fn normalize_type_list_trims_dedupes_and_caps() {
        let input = vec![
            " person ".to_string(),
            "PERSON".to_string(),
            "org-unit".to_string(),
            "".to_string(),
            "   ".to_string(),
        ];
        let out = normalize_type_list(&input);
        assert_eq!(out, vec!["PERSON".to_string(), "ORG_UNIT".to_string()]);
    }

    #[test]
    fn normalize_type_list_respects_max_fifty() {
        let input: Vec<String> = (0..60).map(|i| format!("type_{i}")).collect();
        assert_eq!(normalize_type_list(&input).len(), 50);
    }
}
