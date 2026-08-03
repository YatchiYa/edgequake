# SPEC-095 — PDFium Cache Poison (Challenged RCA + Fix)

Captured: July 31, 2026
Updated: July 31, 2026 (first-principles challenge)
Core Concept: Truncated PDFium extraction poisons a shared cache via non-atomic write + existence-only hit check; EdgeQuake's 5s page-count timeout is not the truncating mechanism.
Domain: Architecture
Priority: 5
Stage: Refined → Implementation
Tags: High Impact, Technical, Root Cause

## Challenge matrix (field report vs source)

| Finding | Verdict | Why |
| --- | --- | --- |
| F1: Extraction lives in upstream `pdfium-auto`, not EdgeQuake | **Keep** | Workspace pins `edgequake-pdf2md = 0.9.8` with `bundled`; no EdgeQuake Rust writes the cache |
| F2: `PDFIUM_LIB_PATH` vs `PDFIUM_DYNAMIC_LIB_PATH` confusion | **Revise** | Bundled `ensure_pdfium_bundled` never read `PDFIUM_LIB_PATH`. Cache still populated because bundled extract was unconditional — not a post-failure fallback |
| F3: 5s `tokio::time::timeout` cancels mid-write and truncates `libpdfium.so` | **Reject** | Upstream `inspect` → `extract_metadata` uses `spawn_blocking`. Dropping the timed-out future does **not** abort the blocking write |
| F4: World-writable `/tmp/edgequake-pdfium-cache` is a planting surface | **Keep** | Sticky bit ≠ integrity; `dlopen` of a writable path remains a risk |

## Corrected root cause

`pdfium-auto` `ensure_pdfium_bundled` wrote embedded bytes directly to the final shared path and treated any existing file as a cache hit:

1. Concurrent cold starts race on `exists()` → both write the final path.
2. SIGKILL / ENOSPC leave a short file that forever satisfies `exists()`.
3. Later callers `dlopen` → `file too short`.

Related hang (SPEC-091 R-17): `get_pdfium()` check-then-act on `OnceCell` can deadlock concurrent first binds.

EdgeQuake's `COUNT_PAGES_TIMEOUT` (5s) remains a valid soft ceiling for *inspect after bind*. It is a symptom amplifier under load (timeout logs while writers race), **not** the poison mechanism.

## Goals

- Atomic extract (temp + `rename`) with size integrity and advisory file lock in `pdfium-auto`.
- Honour `PDFIUM_LIB_PATH` in bundled mode before any cache write.
- Serialize `get_pdfium` first-bind; expose `prime_pdfium()` for EdgeQuake.
- Prime PDFium once at EdgeQuake startup before accepting traffic.
- E2E proof: concurrent cold cache, poison heal, startup prime.

## Non-goals

- Removing `COUNT_PAGES_TIMEOUT`.
- Making the Docker cache non-writable without a pre-extracted image asset.
- Changing vision convert stall-watchdog behaviour.

## Acceptance criteria

1. Concurrent cold-cache `ensure_pdfium_bundled` never leaves a short library file; final size equals embedded byte length.
2. A pre-seeded truncated cache file is deleted and re-extracted on next ensure.
3. When `PDFIUM_LIB_PATH` points at a valid library, bundled mode skips extract (cache dir untouched).
4. Concurrent `get_pdfium` first calls return without hanging.
5. EdgeQuake calls `prime_pdfium` before listen (fail-closed boot). Readiness is boot refusal, not a `/ready` JSON field.
6. Tests: `pdfium-auto` integration (`cold_cache_concurrent`, `poison_heal_ensure`, `lib_path_skips_extract`) + `e2e_spec095_*` / `edgequake-pdf` facade tests.

## Owners

| Layer | Owner | Deliverable |
| --- | --- | --- |
| Root cause | `pdfium-auto` 0.3.1 | Atomic write, integrity, lock, LIB_PATH |
| Init race + prime API | `edgequake-pdf2md` 0.9.9 | `get_or_try_init`, `prime_pdfium` |
| Startup + facade | EdgeQuake | `edgequake-pdf::prime_pdfium`, main.rs prime |
| Proof | EdgeQuake tests | `e2e_spec095_*` |

## Env var SSOT

| Variable | Role |
| --- | --- |
| `PDFIUM_AUTO_CACHE_DIR` | Override extract cache root |
| `PDFIUM_LIB_PATH` | Skip extract/download; bind this path |
| `PDFIUM_BUNDLE_LIB` | Build-time embed path only |
| `EDGEQUAKE_SKIP_PDFIUM_PRIME` | Opt out of startup prime (dev only) |
| `PDFIUM_DYNAMIC_LIB_PATH` | **Legacy** — do not use |

## PDFium binary pin

`pdfium-auto` / build embed: **chromium/7961** (PDFium 152.0.7961.0, latest stable as of 2026-07-20).

## Traceability

- `pdfium-auto` integration: `cold_cache_concurrent`, `poison_heal_ensure`, `lib_path_skips_extract`, `download_min_size_rejects_short`
- `edgequake-pdf2md`: `get_pdfium_serializes_first_bind`
- `edgequake-pdf`: `spec095_subprocess_prime` (unwritable fail-closed, LIB_PATH skip)
- `edgequake-api`: `e2e_spec095_*`, `contract_spec095_pdfium` (prime before `Server::new`)
