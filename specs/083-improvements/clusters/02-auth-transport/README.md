# Cluster 02 — Auth & transport hardening

> **Sprint**: 1  
> **Laws**: LAW-4, LAW-3  
> **Defects**: S-07…S-13, D-50

---

## WHY

Sprint-1 auth/transport defects are largely **FIXED** per register (S-07…S-09, S-11…S-13, D-50). Residual: **S-10 PARTIAL** (CORS/WS Origin). Historical WHY (pre-fix): stolen access tokens lived 24h; unknown JWT roles became User; default JWT secret and open CORS shipped in prod; rate limits spoofable; uploads trusted filenames; benchmark `eval()` executed dataset code; `.env.example` hardcoded vision openai (D-50 — now FIXED).

## ROOT CAUSE

Fail-open defaults across auth/transport. Auth runs before rate limit but key ignores Claims. Security checks are warn-only unless strict flag.

```
  mint JWT (jti unused, iss/aud None)
  Role::parse --> User on garbage
  JWT_SECRET default --> warn --> serve
  CORS None --> Any/Any/Any
  rate_limit(x-tenant-id) --> spoof fresh bucket
  filename/MIME extension-only
  eval(dataset)
  VISION_PROVIDER=openai in example   [FIXED D-50 — example no longer hardcodes]
```

## SOLUTION

| ID | Fix |
|----|-----|
| S-07 | Require iss/aud; jti denylist on logout; shorter access TTL |
| S-08 | `Role::try_parse` → Err → 401 |
| S-09 | Fatal on default/short secret unless DEV_MODE + banner; ≥32 bytes |
| S-10 | Prod CORS allow-list required; WS Origin required in prod |
| S-11 | Rate key from Claims; schedule `cleanup_stale_buckets`; Redis for multi-replica |
| S-12 | `sanitize_filename` + magic-byte MIME |
| S-13 | `ast.literal_eval` / JSON only |
| D-50 | `.env.example` vision empty or ollama |

DRY: `StartupSecurityPolicy`, `AuthenticatedRateKey`, `FileIngressGuard`.

## EDGE CASES

Native clients without Origin; anonymous routes; multi-instance jti store; double extensions.

## E2E

`e2e_logout_rejects_access_jti`, `contract_unknown_role_rejected`, `contract_startup_rejects_default_secret`, `e2e_rate_limit_ignores_spoofed_header`, `contract_no_eval_in_bench047`, `contract_env_example_vision_not_openai_by_default`
