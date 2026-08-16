//! SPEC-131 E2E-131-07 for PDF figure filter call sites.

#[test]
fn e2e_131_07_no_bare_temperature_some_in_pdf() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/src");
    let deny = "temperature: Some(";
    let mut offenders = Vec::new();
    walk_rs(root, &mut |path, content| {
        for (i, line) in content.lines().enumerate() {
            let t = line.trim();
            if t.starts_with("//") {
                continue;
            }
            if t.contains(deny) {
                offenders.push(format!("{}:{}: {t}", path, i + 1));
            }
        }
    });
    assert!(
        offenders.is_empty(),
        "figure_filter must use resolve_effective_temperature:\n{}",
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
            if let (Some(p), Ok(c)) = (path.to_str(), std::fs::read_to_string(&path)) {
                f(p.to_string(), c);
            }
        }
    }
}
