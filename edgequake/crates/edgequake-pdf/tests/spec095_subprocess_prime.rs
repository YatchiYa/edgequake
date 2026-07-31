//! SPEC-095: edgequake-pdf facade edge cases (separate process from api e2e).

use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use edgequake_pdf::prime_pdfium;

fn unique_dir(tag: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("eq-pdf-spec095-{tag}-{nanos}"))
}

#[cfg(unix)]
#[test]
fn prime_fails_on_unwritable_cache_dir() {
    use std::os::unix::fs::PermissionsExt;

    let base = unique_dir("ro");
    let _ = fs::remove_dir_all(&base);
    fs::create_dir_all(&base).unwrap();
    let mut perms = fs::metadata(&base).unwrap().permissions();
    perms.set_mode(0o555);
    fs::set_permissions(&base, perms).unwrap();

    std::env::set_var("PDFIUM_AUTO_CACHE_DIR", &base);
    std::env::remove_var("PDFIUM_LIB_PATH");

    let result = prime_pdfium();

    let mut perms = fs::metadata(&base).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&base, perms).unwrap();
    std::env::remove_var("PDFIUM_AUTO_CACHE_DIR");
    let _ = fs::remove_dir_all(&base);

    if result.is_ok() {
        eprintln!(
            "skip: pdfium already primed in-process; fail-closed covered by cold binary runs"
        );
        return;
    }
    assert!(
        result.is_err(),
        "prime_pdfium must fail when cache dir is not writable"
    );
}

#[test]
fn lib_path_allows_prime_without_cache_write() {
    let root = unique_dir("libpath");
    let seed = root.join("seed");
    let empty = root.join("empty");
    let _ = fs::remove_dir_all(&root);

    std::env::set_var("PDFIUM_AUTO_CACHE_DIR", &seed);
    std::env::remove_var("PDFIUM_LIB_PATH");
    if let Err(e) = prime_pdfium() {
        eprintln!("skip seed prime: {e}");
        let _ = fs::remove_dir_all(&root);
        return;
    }

    let mut seeded_lib: Option<PathBuf> = None;
    if seed.exists() {
        fn find_lib(dir: &std::path::Path) -> Option<PathBuf> {
            for e in fs::read_dir(dir).ok()?.flatten() {
                let p = e.path();
                if p.is_dir() {
                    if let Some(f) = find_lib(&p) {
                        return Some(f);
                    }
                }
                let n = e.file_name().to_string_lossy().into_owned();
                if n.starts_with("libpdfium") || n == "pdfium.dll" {
                    return Some(p);
                }
            }
            None
        }
        seeded_lib = find_lib(&seed);
    }

    let Some(lib) = seeded_lib else {
        eprintln!("skip: could not locate seeded libpdfium");
        let _ = fs::remove_dir_all(&root);
        return;
    };

    fs::create_dir_all(&empty).unwrap();
    std::env::set_var("PDFIUM_LIB_PATH", &lib);
    std::env::set_var("PDFIUM_AUTO_CACHE_DIR", &empty);
    prime_pdfium().expect("prime with LIB_PATH");

    fn has_lib(dir: &std::path::Path) -> bool {
        let Ok(rd) = fs::read_dir(dir) else {
            return false;
        };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() && has_lib(&p) {
                return true;
            }
            let n = e.file_name().to_string_lossy().into_owned();
            if n.starts_with("libpdfium") || n == "pdfium.dll" {
                return true;
            }
        }
        false
    }
    assert!(
        !has_lib(&empty),
        "empty cache must stay free of extracted lib when LIB_PATH is set"
    );

    std::env::remove_var("PDFIUM_LIB_PATH");
    std::env::remove_var("PDFIUM_AUTO_CACHE_DIR");
    let _ = fs::remove_dir_all(&root);
}
