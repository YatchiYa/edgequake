//! Compile-time OpenAPI path registry SSOT (SPEC-027 phase 15).
//!
//! `build.rs` scans handler annotations and validates they match `openapi.rs` paths().

include!(concat!(env!("OUT_DIR"), "/openapi_path_count.rs"));

/// Handler function names registered in `openapi.rs` `paths()` (last path segment).
<<<<<<< HEAD
pub const REGISTERED_HANDLER_COUNT: usize = 186;
=======
pub const REGISTERED_HANDLER_COUNT: usize = 197;
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042

const _: () = assert!(
    OPENAPI_GENERATED_HANDLER_COUNT == REGISTERED_HANDLER_COUNT,
    "openapi.rs paths() count must match build.rs scan — run build and update REGISTERED_HANDLER_COUNT"
);
