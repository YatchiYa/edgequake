//! SPEC-131 E2E-131-07: no bare `temperature: Some(` LLM CompletionOptions in pipeline src.

#[test]
fn e2e_131_07_no_bare_temperature_some_in_pipeline() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/src");
    let deny_substr = "temperature: Some(";
    let mut offenders = Vec::new();
    walk_rs(root, &mut |path, content| {
        for (i, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("//") {
                continue;
            }
            if trimmed.contains(deny_substr) {
                offenders.push(format!("{}:{}: {}", path, i + 1, trimmed));
            }
        }
    });
    assert!(
        offenders.is_empty(),
        "E2E-131-07: use resolve_effective_temperature instead of bare temperature: Some(\n{}",
        offenders.join("\n")
    );
}

fn walk_rs(dir: &str, f: &mut dyn FnMut(String, String)) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if let Some(s) = path.to_str() {
                walk_rs(s, f);
            }
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            if let (Some(p), Ok(content)) = (path.to_str(), std::fs::read_to_string(&path)) {
                f(p.to_string(), content);
            }
        }
    }
}
