//! SPEC-111 / LAW-MIG — checksum repair wiring must stay coherent.
//!
//! Catches the class of failure where:
//! - migrate CLI needs repair authorization but make_dev only set DEV_MODE on the server
//! - repair modules invent private allow helpers that ignore the scoped allowlist
//! - Makefile allowlist drifts from Rust `KNOWN_CHECKSUM_REPAIR_VERSIONS`

#[test]
fn makefile_migrate_passes_scoped_checksum_repair_allowlist() {
    let makefile = std::fs::read_to_string("../../../Makefile").expect("Makefile");
    assert!(
        makefile.contains("KNOWN_CHECKSUM_REPAIR_VERSIONS"),
        "Makefile must define KNOWN_CHECKSUM_REPAIR_VERSIONS"
    );
    assert!(
        makefile.contains("EDGEQUAKE_ALLOW_CHECKSUM_REPAIR"),
        "VISIBLE_MIGRATE_STEP must pass EDGEQUAKE_ALLOW_CHECKSUM_REPAIR"
    );
    assert!(
        makefile.contains("EDGEQUAKE_DEV_MODE=\"$(DEV_EDGEQUAKE_DEV_MODE)\""),
        "VISIBLE_MIGRATE_STEP must also pass EDGEQUAKE_DEV_MODE"
    );
    for v in ["71", "78", "118", "121", "125", "131"] {
        assert!(
            makefile.contains(v),
            "Makefile allowlist must include repair version {v}"
        );
    }
}

#[test]
fn rust_known_versions_match_makefile_list() {
    // Parse from source (no `postgres` feature required for this contract binary).
    let rust_src = include_str!("../src/state/migration_bootstrap/checksum_repair.rs");
    let rust_line = rust_src
        .lines()
        .find(|l| l.contains("KNOWN_CHECKSUM_REPAIR_VERSIONS: &[i64]"))
        .expect("Rust KNOWN_CHECKSUM_REPAIR_VERSIONS");
    let rust_list = rust_line
        .split('=')
        .nth(1)
        .and_then(|rhs| {
            let start = rhs.find('[')?;
            let end = rhs.find(']')?;
            Some(rhs[start + 1..end].to_string())
        })
        .expect("Rust array literal");
    let from_rust: Vec<i64> = rust_list
        .split(',')
        .filter_map(|p| p.trim().parse().ok())
        .collect();

    let makefile = std::fs::read_to_string("../../../Makefile").expect("Makefile");
    let make_line = makefile
        .lines()
        .find(|l| l.starts_with("KNOWN_CHECKSUM_REPAIR_VERSIONS"))
        .expect("KNOWN_CHECKSUM_REPAIR_VERSIONS assignment");
    let from_make: Vec<i64> = make_line
        .split('=')
        .nth(1)
        .expect("rhs")
        .split(',')
        .filter_map(|p| p.trim().parse().ok())
        .collect();

    assert_eq!(
        from_make, from_rust,
        "Makefile and Rust KNOWN_CHECKSUM_REPAIR_VERSIONS must be identical"
    );
    assert!(
        !from_rust.is_empty(),
        "KNOWN_CHECKSUM_REPAIR_VERSIONS must not be empty"
    );
}

#[test]
fn all_repair_modules_call_shared_allow_helper() {
    let modules = [
        ("m071.rs", "MIGRATION_071_VERSION"),
        ("m078.rs", "MIGRATION_078_VERSION"),
        ("m118.rs", "MIGRATION_118_VERSION"),
        ("m121.rs", "MIGRATION_121_VERSION"),
        ("m125.rs", "MIGRATION_125_VERSION"),
        ("m131.rs", "MIGRATION_131_VERSION"),
    ];
    for (file, version_const) in modules {
        let path = format!("src/state/migration_bootstrap/reconcile/{file}");
        let src = std::fs::read_to_string(&path).unwrap_or_else(|_| panic!("read {path}"));
        assert!(
            src.contains(&format!("allow_checksum_repair({version_const})")),
            "{file} must call allow_checksum_repair({version_const})"
        );
        assert!(
            src.contains("refuse_silent_repair_message"),
            "{file} must use refuse_silent_repair_message"
        );
        assert!(
            !src.contains("fn allow_checksum_repair()"),
            "{file} must not define a private allow_checksum_repair()"
        );
    }
}

#[test]
fn immutability_spec_exists() {
    assert!(
        std::path::Path::new("../../../specs/111-issues/10-migration-immutability.md").exists(),
        "LAW-MIG doc required"
    );
}
