# 03 — Deployment (Google Cloud)

## What is deployed, where

| Item | Value |
|---|---|
| GCP project | `second-project-511608` (billing on, deployer `noreply.syctra@gmail.com` is Owner) |
| Region / zone | `europe-west1` / `europe-west1-b` |
| EdgeQuake (web UI + API) | **https://35-187-6-117.sslip.io** — Let's Encrypt certificate, HTTP → HTTPS 308 |
| Agents platform (ChatGPT-style, SPEC-165) | **https://35-187-6-117.sslip.io/agents** — same EdgeQuake sign-in |
| Web UI login | user `admin`; password in Secret Manager `edgequake-bootstrap-admin-password` |
| VM | `edgequake-vm`, `e2-standard-2` (2 vCPU, 8 GB), Debian 12, Shielded VM, static IP `35.187.6.117` |
| Data | disk `edgequake-vm-data` (50 GB pd-balanced) mounted at `/mnt/edgequake-vm-data`, daily snapshot 03:00, 14 days retention |
| Images | `europe-west1-docker.pkg.dev/second-project-511608/edgequake/{edgequake,edgequake-frontend,edgequake-agents}:<content tag>` built from this working tree |
| LLM | Mistral (`mistral-small-latest` for extraction/answers, `mistral-embed` 1024-d, copilot `mistral-medium-latest`, agents `mistral/mistral-medium-latest`). OpenAI is recorded as `UNSET` because the local OpenAI key is invalid |
| Demo data | Default workspace seeded with `README.md` and `tutorial/01-what-this-is.md` (public project docs) |
| Multi-user demo | Workspaces **Équipe Alpha** (doc "Projet Phoenix") and **Équipe Beta** (doc "Projet Neptune") in the Default tenant; users `alice` (member of Alpha only) and `bob` (member of Beta only), granted with the members API (`/api/v1/tenants/{id}/members`). Verified: 20/20 API isolation checks and a two-browser UI test (each user sees only their workspace, documents and conversations) |

Credentials are never written in the repository. Read them with:

```bash
gcloud secrets versions access latest --secret=edgequake-bootstrap-admin-password --project=second-project-511608
```

## How it fits together

```mermaid
flowchart TB
  subgraph Internet
    B[Browser]
  end
  subgraph GCP["GCP project second-project-511608 — europe-west1"]
    subgraph VM["edgequake-vm (Docker Compose)"]
      C[caddy :80/:443<br/>Let's Encrypt]
      F[frontend :3000<br/>Next.js + /api/copilot]
      A[api :8080<br/>Rust]
      G[agents :8765<br/>Google ADK]
      M[migrate<br/>one-shot]
      P[(postgres<br/>pgvector + AGE)]
    end
    D[(data disk + daily snapshots)]
    SM[Secret Manager]
    AR[Artifact Registry]
    GCS[bucket: deploy files]
    IAP[IAP TCP forwarding<br/>SSH only]
  end
  B -->|https://IP.sslip.io| C
  C -->|/api/v1 /mcp /ws /health| A
  C -->|/ and /api/copilot| F
  F -->|/api/agents · internal token| G
  F -->|http://api:8080, user token| A
  G -->|MCP + REST, read-only key| A
  A --> P
  M --> P
  P --- D
  VM -. startup: render .env .-> SM
  VM -. pull images .-> AR
  VM -. copy compose/scripts .-> GCS
```

Network: firewall opens 80 and 443 only; SSH is reachable only through IAP (`35.235.240.0/20`). Service accounts: the VM reads secrets, images and the deploy bucket; a separate build account writes images and build logs.

## How it is built and deployed (pure Terraform)

```bash
cd deploy/gcp/terraform
cp terraform.tfvars.example terraform.tfvars   # project_id, keys, build_mode
terraform init
terraform apply      # build images (if sources changed) → infra → redeploy → wait for HTTPS
terraform destroy    # everything, including images and (keep_snapshots_on_destroy = false) snapshots
```

`terraform apply` runs, in dependency order:

1. **APIs**, VPC/subnet/firewalls, static IP, Artifact Registry, bucket, service accounts and IAM, secrets (random passwords + LLM keys).
2. **`terraform_data.images`** — `scripts/build-images.sh` builds `api`, `frontend` and `agents` for the content tag (git sha + hash of uncommitted build sources). An unchanged tree gives the same tag; if the three images exist the step is skipped. `build_mode = "cloud"` uses Cloud Build (falls back to smaller machines when the project has no quota for `e2-highcpu-32`; retries while new IAM grants propagate); `"local"` uses `docker buildx` on the workstation.
3. **VM** with the startup script: installs Docker CE, mounts the data disk, copies compose files and scripts from the bucket, renders `/opt/edgequake/.env` from Secret Manager (provider-matched models, shared agents internal token), runs `deploy.sh` (pull → `migrate dry-run` → `migrate` → `up` → HTTPS and extension gates).
4. **`terraform_data.redeploy`** — re-runs the startup over IAP SSH whenever the image tag, the deploy files or any secret changes (`scripts/remote-redeploy.sh`).
5. **`terraform_data.health`** — `apply` returns only when `https://…/health` answers.

`deploy/gcp/up.sh` and `down.sh` remain as optional wrappers (access checks, keys from the repo `.env`, confirmation, printing the URLs).

## Operate

| Task | Command |
|---|---|
| Shell on the VM | `gcloud compute ssh edgequake-vm --zone=europe-west1-b --project=second-project-511608 --tunnel-through-iap` |
| Containers | `sudo docker ps` · `sudo docker logs -f edgequake-api` |
| Startup / deploy logs | `sudo tail -f /var/log/edgequake-startup.log /var/log/edgequake-deploy.log` |
| Ship a code change | `terraform apply` (new content tag → build → redeploy) |
| Roll back | `terraform apply -var image_tag=<old tag> -var build_images=false` |
| Change an LLM key | edit `terraform.tfvars`, `terraform apply` (new secret version → redeploy) |

## Problems met during the first deployment (and the permanent fix)

| Symptom | Root cause | Fix in the module |
|---|---|---|
| Account could not see `second-pr-511015` | The gcloud config pointed at a project this account has no role on | Deployed to `second-project-511608` (Owner + billing); wrapper checks access first |
| Cloud Build: *service account … does not have access to the bucket* | `roles/storage.objectAdmin` lacks `storage.buckets.get` | Added `roles/storage.legacyBucketReader` for the build account |
| Cloud Build: *quota restrictions … this machine type* | New project, no quota for `e2-highcpu-32` / `e2-highcpu-8` | Automatic fallback to smaller machines; `build_mode = "local"` |
| Local build: disk full | Workstation root disk at 100 % | Freed space (unused images, build cache, a temporary worktree); README notes ~30 GB needed |
| Documents failed: *Mistral embeddings 400* | Compose defaulted the embedding model to `text-embedding-3-small` for every provider | Provider-matched models in `render-env.sh`; no OpenAI default in compose |
| Copilot would 404 in production | Caddy sent all `/api/*` to the Rust API | `/api/copilot*` routed to the frontend, unbuffered |
| Terraform-only apply failed on caller email | ADC tokens lack the email scope | Fallback to the active gcloud account |
| Redeploy step hung | `pgrep -f` matched its own SSH command line | Wait on `google-startup-scripts.service` instead |
| API crash loop after a redeploy (site down ~5 min, 2026-10-16 21:51 UTC) | A startup error (PDFium prime) made `async_main` return early, dropping the ingest Tokio runtime inside an async context ("Cannot drop a runtime…" panic, which also hid the real error); the restarted container reused its `/tmp`, so it failed every time | Emergency: re-pointed the image tag to the last good API and redeployed. Fix: `BackgroundRuntime` guard in `edgequake/src/main.rs` (non-blocking shutdown on any exit path, regression test), and `tmpfs: /tmp` for the API container so every restart starts clean |

## Cost

About 65–70 USD/month in europe-west1 (e2-standard-2 + 50 GB pd-balanced + static IP + snapshots), plus build costs when `build_mode = "cloud"`. `terraform destroy` stops all of it.
