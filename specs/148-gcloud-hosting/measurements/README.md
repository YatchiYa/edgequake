# Measurements

Post-apply evidence lives here. **Never commit secrets**, `.env`, or API keys.

Applied **2026-09-19** (rename from `eq148-*`): VM **`elitizon-db`**, public IP **`34.135.165.171`**, bucket **`saas-app-001-edgequake-artifacts`**, VPC **`edgequake-host-vpc`** (leftover Option B already owns `edgequake-vpc`).

GitHub Actions variables: `EDGEQUAKE_GCP_WIF_PROVIDER`, `EDGEQUAKE_GCP_DEPLOY_SA`, `EDGEQUAKE_GCP_PROJECT`, `EDGEQUAKE_GCP_ZONE`, `EDGEQUAKE_GCP_INSTANCE`.

Expected artifacts after apply:

| File | Contents |
|------|----------|
| `apply.txt` | `terraform output` (IPs, WIF provider, image pin) — redact emails if needed |
| `health.json` | `curl -sk https://$IP/health` (status/version/components only) |
| `dx.txt` | `\dx` showing `vector` and `age` |
| `redirect.txt` | `curl -sI http://$IP/health` headers (301/308) |
