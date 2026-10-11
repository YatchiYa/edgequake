//! Path and verb contract for generated OpenAPI operations. No server required.

use std::fs;
use std::path::Path;

#[test]
fn every_http_operation_path_and_generated_verb() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let manifest: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(root.join("sdks/operation-manifest.json")).unwrap())
            .unwrap();
    let generated = fs::read_to_string(
        root.join("sdks/rust/src/resources/operations_generated.rs"),
    )
    .unwrap();
    let mut blob = String::new();
    walk(&root.join("sdks/rust/src"), &mut blob);
    for op in manifest.as_array().unwrap() {
        if op["kind"] == "websocket" {
            let path = op["path"].as_str().unwrap();
            assert!(generated.contains(path), "{path}");
            continue;
        }
        let path = op["path"].as_str().unwrap();
        assert!(blob.contains(path), "{path}");
        if op["missing"]["rust"] == true {
            let needle = format!("{} {}", op["method"].as_str().unwrap(), path);
            assert!(generated.contains(&needle), "{needle}");
        }
    }
}

fn walk(dir: &Path, out: &mut String) {
    for entry in fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            walk(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            out.push_str(&fs::read_to_string(path).unwrap());
        }
    }
}
