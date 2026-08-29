//! Distroless-safe container helpers (no shell, no curl).
//!
//! Docker HEALTHCHECK and Kubernetes preStop must run as the API binary:
//! `gcr.io/distroless/cc` has neither `curl` nor `sh`.

use anyhow::{bail, Context, Result};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

const DEFAULT_PORT: u16 = 8080;
const DEFAULT_PRESTOP_SECS: u64 = 15;
const MIN_PRESTOP_SECS: u64 = 1;
const MAX_PRESTOP_SECS: u64 = 300;
const PROBE_TIMEOUT: Duration = Duration::from_secs(5);
const MAX_LIVE_BYTES: u64 = 4096;

const LIVE_REQUEST: &[u8] = b"GET /live HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n";

/// `edgequake healthcheck` — GET `/live` on localhost (liveness, not deep `/health`).
pub fn run_healthcheck() -> Result<()> {
    let port = probe_port();
    let addr = format!("127.0.0.1:{port}");
    let response = get_live(&addr)?;
    if !live_response_is_ok(&response) {
        let first = response.lines().next().unwrap_or("(empty)");
        bail!("healthcheck: GET /live on {addr} was not HTTP 200 ({first})");
    }
    Ok(())
}

/// `edgequake pre-stop [seconds]` — drain delay before SIGTERM (Helm preStop).
pub fn run_pre_stop(arg: Option<String>) -> Result<()> {
    let secs = resolve_pre_stop_seconds(
        arg.as_deref(),
        std::env::var("EDGEQUAKE_PRESTOP_SECONDS").ok().as_deref(),
    )?;
    std::thread::sleep(Duration::from_secs(secs));
    Ok(())
}

fn probe_port() -> u16 {
    std::env::var("EDGEQUAKE_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_PORT)
}

fn get_live(addr: &str) -> Result<String> {
    let sock_addr: std::net::SocketAddr = addr
        .parse()
        .with_context(|| format!("healthcheck: parse {addr}"))?;
    let mut stream = TcpStream::connect_timeout(&sock_addr, PROBE_TIMEOUT)
        .with_context(|| format!("healthcheck: connect {addr}"))?;
    stream
        .set_read_timeout(Some(PROBE_TIMEOUT))
        .context("healthcheck: set read timeout")?;
    stream
        .set_write_timeout(Some(PROBE_TIMEOUT))
        .context("healthcheck: set write timeout")?;
    stream
        .write_all(LIVE_REQUEST)
        .context("healthcheck: write GET /live")?;
    let mut limited = stream.take(MAX_LIVE_BYTES);
    let mut buf = Vec::new();
    limited
        .read_to_end(&mut buf)
        .context("healthcheck: read /live response")?;
    Ok(String::from_utf8_lossy(&buf).into_owned())
}

fn live_response_is_ok(response: &str) -> bool {
    response
        .split(['\r', '\n'])
        .next()
        .is_some_and(status_line_is_http_200)
}

/// True for `HTTP/1.0 200` / `HTTP/1.1 200` with a space (or EOL) after 200,
/// so `2000` is not treated as success.
fn status_line_is_http_200(line: &str) -> bool {
    for prefix in ["HTTP/1.0 200", "HTTP/1.1 200"] {
        if let Some(rest) = line.strip_prefix(prefix) {
            return rest.is_empty() || rest.starts_with(' ') || rest.starts_with('\t');
        }
    }
    false
}

fn resolve_pre_stop_seconds(arg: Option<&str>, env: Option<&str>) -> Result<u64> {
    let parsed = if let Some(raw) = arg.filter(|s| !s.is_empty()) {
        raw.parse()
            .with_context(|| format!("pre-stop: invalid seconds '{raw}'"))?
    } else if let Some(raw) = env.filter(|s| !s.is_empty()) {
        raw.parse()
            .with_context(|| format!("pre-stop: invalid EDGEQUAKE_PRESTOP_SECONDS '{raw}'"))?
    } else {
        DEFAULT_PRESTOP_SECS
    };
    Ok(parsed.clamp(MIN_PRESTOP_SECS, MAX_PRESTOP_SECS))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::thread;

    #[test]
    fn live_ok_http11() {
        assert!(live_response_is_ok(
            "HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nOK"
        ));
    }

    #[test]
    fn live_ok_http10() {
        assert!(live_response_is_ok("HTTP/1.0 200 OK\r\n\r\nOK"));
    }

    #[test]
    fn live_rejects_non_200() {
        assert!(!live_response_is_ok(
            "HTTP/1.1 503 Service Unavailable\r\n\r\n"
        ));
        assert!(!live_response_is_ok(""));
        assert!(!live_response_is_ok("connection refused"));
        assert!(!live_response_is_ok("HTTP/1.1 2000 OK\r\n\r\n"));
        assert!(!live_response_is_ok("HTTP/1.1 200OK\r\n\r\n"));
    }

    #[test]
    fn live_accepts_bare_200() {
        assert!(live_response_is_ok("HTTP/1.1 200"));
        assert!(live_response_is_ok("HTTP/1.0 200\r\n"));
    }

    #[test]
    fn pre_stop_prefers_argv() {
        assert_eq!(resolve_pre_stop_seconds(Some("20"), Some("8")).unwrap(), 20);
    }

    #[test]
    fn pre_stop_uses_env_then_default() {
        assert_eq!(resolve_pre_stop_seconds(None, Some("8")).unwrap(), 8);
        assert_eq!(resolve_pre_stop_seconds(None, None).unwrap(), 15);
        assert_eq!(resolve_pre_stop_seconds(Some(""), Some("9")).unwrap(), 9);
    }

    #[test]
    fn pre_stop_clamps_to_1_through_300() {
        assert_eq!(resolve_pre_stop_seconds(Some("0"), None).unwrap(), 1);
        assert_eq!(resolve_pre_stop_seconds(Some("300"), None).unwrap(), 300);
        assert_eq!(resolve_pre_stop_seconds(Some("301"), None).unwrap(), 300);
        assert_eq!(resolve_pre_stop_seconds(None, Some("99999")).unwrap(), 300);
    }

    #[test]
    fn pre_stop_rejects_non_integer() {
        assert!(resolve_pre_stop_seconds(Some("x"), None).is_err());
    }

    #[test]
    fn get_live_fails_when_nothing_listens() {
        let port = {
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            listener.local_addr().unwrap().port()
        };
        let err = get_live(&format!("127.0.0.1:{port}")).unwrap_err();
        assert!(
            err.to_string().contains("connect"),
            "expected connect error, got {err}"
        );
    }

    #[test]
    fn get_live_reads_200_from_local_listener() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let handle = thread::spawn(move || {
            let (mut sock, _) = listener.accept().unwrap();
            let mut buf = [0u8; 128];
            let _ = sock.read(&mut buf);
            sock.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nOK")
                .unwrap();
        });
        let body = get_live(&addr.to_string()).unwrap();
        assert!(live_response_is_ok(&body));
        handle.join().unwrap();
    }

    #[test]
    fn get_live_caps_oversized_response() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let handle = thread::spawn(move || {
            let (mut sock, _) = listener.accept().unwrap();
            let mut buf = [0u8; 128];
            let _ = sock.read(&mut buf);
            let mut body = b"HTTP/1.1 200 OK\r\n\r\n".to_vec();
            body.extend(vec![b'A'; 8192]);
            sock.write_all(&body).unwrap();
        });
        let body = get_live(&addr.to_string()).unwrap();
        assert!(body.len() <= MAX_LIVE_BYTES as usize);
        handle.join().unwrap();
    }
}
