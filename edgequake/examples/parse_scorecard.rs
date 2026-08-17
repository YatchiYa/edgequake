//! SPEC-094 parse scorecard harness.
//!
//! ```bash
//! cargo run -p edgequake --example parse_scorecard -- \
//!   http://127.0.0.1:8080 \
//!   ../legacy/edgequake-pdf/test-data \
//!   edgeparse \
//!   10 \
//!   /tmp/parse-scorecard.json
//! ```
//!
//! Args: `<base_url> <golden_dir> [backend=edgeparse] [limit=50] [out=parse-scorecard.json]`

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct FileResult {
    file: String,
    backend: String,
    ok: bool,
    total_ms: Option<u64>,
    page_count: Option<u32>,
    pages_per_second: Option<f64>,
    fallback_applied: Option<bool>,
    error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Scorecard {
    base_url: String,
    backend: String,
    files: Vec<FileResult>,
    summary: Summary,
}

#[derive(Debug, Serialize, Deserialize)]
struct Summary {
    total: usize,
    ok: usize,
    failed: usize,
    fallback_rate: f64,
    mean_total_ms: f64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let base_url = args
        .next()
        .unwrap_or_else(|| "http://127.0.0.1:8080".into());
    let golden_dir = PathBuf::from(
        args.next()
            .unwrap_or_else(|| "../legacy/edgequake-pdf/test-data".into()),
    );
    let backend = args.next().unwrap_or_else(|| "edgeparse".into());
    let limit: usize = args.next().and_then(|s| s.parse().ok()).unwrap_or(50);
    let out = PathBuf::from(args.next().unwrap_or_else(|| "parse-scorecard.json".into()));

    let before_temp = list_temp_names();
    let backends = if backend == "all" {
        vec!["edgeparse".to_string(), "vision".to_string()]
    } else {
        vec![backend.clone()]
    };

    let mut pdfs: Vec<PathBuf> = fs::read_dir(&golden_dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.extension()
                .and_then(|e| e.to_str())
                .map(|e| e.eq_ignore_ascii_case("pdf"))
                .unwrap_or(false)
        })
        .collect();
    pdfs.sort();
    if limit > 0 {
        pdfs.truncate(limit);
    }

    let client = reqwest::Client::new();
    let mut files = Vec::new();

    for b in &backends {
        for pdf in &pdfs {
            let result = parse_one(&client, &base_url, pdf, b).await;
            eprintln!(
                "{} [{}] ok={} ms={:?}",
                pdf.file_name().unwrap_or_default().to_string_lossy(),
                b,
                result.ok,
                result.total_ms
            );
            files.push(result);
        }
    }

    let ok = files.iter().filter(|f| f.ok).count();
    let failed = files.len() - ok;
    let fallbacks = files
        .iter()
        .filter(|f| f.fallback_applied.unwrap_or(false))
        .count();
    let mean_total_ms = if ok == 0 {
        0.0
    } else {
        files.iter().filter_map(|f| f.total_ms).sum::<u64>() as f64 / ok as f64
    };

    let scorecard = Scorecard {
        base_url: base_url.clone(),
        backend,
        files,
        summary: Summary {
            total: ok + failed,
            ok,
            failed,
            fallback_rate: if ok + failed == 0 {
                0.0
            } else {
                fallbacks as f64 / (ok + failed) as f64
            },
            mean_total_ms,
        },
    };

    fs::write(&out, serde_json::to_string_pretty(&scorecard)?)?;
    eprintln!("Wrote scorecard to {}", out.display());

    let after_temp = list_temp_names();
    for name in after_temp.difference(&before_temp) {
        if name.contains("edgequake-parse") {
            return Err(format!("temp residue left behind: {name}").into());
        }
    }
    eprintln!("temp residue check: ok");

    if scorecard.summary.failed > 0 {
        std::process::exit(1);
    }
    Ok(())
}

async fn parse_one(
    client: &reqwest::Client,
    base_url: &str,
    pdf: &Path,
    backend: &str,
) -> FileResult {
    let started = Instant::now();
    let bytes = match fs::read(pdf) {
        Ok(b) => b,
        Err(e) => {
            return FileResult {
                file: pdf.display().to_string(),
                backend: backend.to_string(),
                ok: false,
                total_ms: None,
                page_count: None,
                pages_per_second: None,
                fallback_applied: None,
                error: Some(e.to_string()),
            };
        }
    };

    let url = format!("{}/api/v1/parse", base_url.trim_end_matches('/'));
    let options = serde_json::json!({ "backend": backend });
    let part = match reqwest::multipart::Part::bytes(bytes)
        .file_name(
            pdf.file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("document.pdf")
                .to_string(),
        )
        .mime_str("application/pdf")
    {
        Ok(p) => p,
        Err(e) => {
            return FileResult {
                file: pdf.display().to_string(),
                backend: backend.to_string(),
                ok: false,
                total_ms: Some(started.elapsed().as_millis() as u64),
                page_count: None,
                pages_per_second: None,
                fallback_applied: None,
                error: Some(e.to_string()),
            };
        }
    };

    let form = reqwest::multipart::Form::new()
        .part("file", part)
        .text("options", options.to_string());

    match client.post(&url).multipart(form).send().await {
        Ok(resp) => {
            let status = resp.status();
            let body: serde_json::Value = resp.json().await.unwrap_or(serde_json::json!({}));
            if !status.is_success() {
                return FileResult {
                    file: pdf.display().to_string(),
                    backend: backend.to_string(),
                    ok: false,
                    total_ms: Some(started.elapsed().as_millis() as u64),
                    page_count: None,
                    pages_per_second: None,
                    fallback_applied: None,
                    error: Some(format!("HTTP {status}: {body}")),
                };
            }
            FileResult {
                file: pdf.display().to_string(),
                backend: backend.to_string(),
                ok: true,
                total_ms: body["metrics"]["total_ms"].as_u64(),
                page_count: body["page_count"].as_u64().map(|v| v as u32),
                pages_per_second: body["metrics"]["pages_per_second"].as_f64(),
                fallback_applied: body["fallback_applied"].as_bool(),
                error: None,
            }
        }
        Err(e) => FileResult {
            file: pdf.display().to_string(),
            backend: backend.to_string(),
            ok: false,
            total_ms: Some(started.elapsed().as_millis() as u64),
            page_count: None,
            pages_per_second: None,
            fallback_applied: None,
            error: Some(e.to_string()),
        },
    }
}

fn list_temp_names() -> HashSet<String> {
    std::fs::read_dir(std::env::temp_dir())
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect()
}
