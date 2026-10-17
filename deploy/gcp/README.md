# EdgeQuake on your own GCP project — plain Terraform

One GCE Debian VM + persistent data disk (daily snapshots, 14 days) running Docker Compose:
Postgres (Apache AGE + pgvector — Cloud SQL cannot host AGE) → migrate → api → frontend (with the
copilot) → ADK agents → Caddy. Caddy is the only public listener (:80 redirects to HTTPS). Secrets live
in Secret Manager. SSH is IAP-only.

```bash
cd deploy/gcp/terraform
cp terraform.tfvars.example terraform.tfvars   # set project_id and mistral_api_key (or openai_api_key)
terraform init
terraform apply      # builds images, creates everything, deploys, waits until https://…/health answers
terraform output     # URLs + the commands that print the passwords
terraform destroy    # removes everything
```

## From a fresh clone, in your own GCP project (exact commands)

```bash
# 0. Tools: gcloud (https://cloud.google.com/sdk/docs/install), terraform >= 1.5, git, curl, bash.
#    Docker too if you choose build_mode = "local".
git clone <this repository> edgequake && cd edgequake

# 1. Sign in twice: gcloud for the provisioners, Application Default Credentials for Terraform.
gcloud auth login
gcloud auth application-default login

# 2. A project with billing (skip if you already have one where you are Owner).
gcloud projects create my-edgequake-123 --name="EdgeQuake"
gcloud billing accounts list
gcloud billing projects link my-edgequake-123 --billing-account=XXXXXX-XXXXXX-XXXXXX

# 3. Settings (git-ignored file: it holds your LLM key).
cd deploy/gcp/terraform
cp terraform.tfvars.example terraform.tfvars
#    edit terraform.tfvars:  project_id = "my-edgequake-123"
#                            mistral_api_key = "..."     (or openai_api_key)
#    optional:               build_mode = "local"        (fast on a strong machine)

# 4. Up (first run: builds the images, ~20–60 min; later runs only rebuild what changed).
terraform init
terraform apply

# 5. Use it.
terraform output public_origin                    # https://<ip>.sslip.io  — user: admin
terraform output agents_url                       # …/agents (same sign-in)
$(terraform output -raw admin_password_command)   # prints the admin password

# 6. Down (everything; snapshots too if keep_snapshots_on_destroy = false was applied).
terraform destroy
```

Nothing in the repository is tied to a specific project: the project, keys and options live
only in your `terraform.tfvars`, and the state in your local `terraform.tfstate` (keep both
safe and out of git; they are git-ignored). Working from an archive instead of a git clone
also works (images are then tagged by a hash of the sources).

## Prerequisites

- A GCP project with **billing enabled**; your account needs **Owner**, or **Editor + Security Admin +
  Service Usage Admin** (it creates service accounts, IAM bindings, secrets and enables APIs).
- `gcloud auth login` **and** `gcloud auth application-default login` (Terraform uses Application
  Default Credentials; provisioners use gcloud), Terraform >= 1.5, `git`, `curl`, `bash`.
  Docker only for `build_mode = "local"`.
- A working LLM key. Test it first, e.g. `curl -s -o /dev/null -w '%{http_code}' https://api.mistral.ai/v1/models -H "Authorization: Bearer $KEY"` → `200`.

## What `terraform apply` does

1. APIs, VPC/subnet/firewalls, static IP, Artifact Registry, deploy bucket, service accounts + IAM,
   secrets (random Postgres/JWT/admin/master-key passwords, agents internal token, your LLM keys).
2. `terraform_data.images` — builds `edgequake`, `edgequake-frontend`, `edgequake-agents` from **this
   checkout** for a content tag (git sha + hash of uncommitted build sources). Same sources → same tag →
   nothing to do; existing images are never rebuilt (`FORCE_BUILD=1` to force).
   - `build_mode = "cloud"` (default): Cloud Build. New projects often have no quota for big build
     machines; the script falls back to `e2-highcpu-8`, then the default machine, and retries while new
     IAM grants propagate. The Rust build can take 30–60 min on small machines.
   - `build_mode = "local"`: `docker buildx` on your machine (much faster on a strong workstation;
     needs ~30 GB free disk).
3. The VM. Its startup script installs Docker, mounts the data disk, copies compose files and scripts
   from the bucket, renders `/opt/edgequake/.env` from Secret Manager (models matched to the provider,
   shared agents internal token) and runs `deploy.sh`: pull → migrate dry-run → migrate →
   up → HTTPS / routing / `\dx` gates.
4. `terraform_data.redeploy` — re-runs that startup over IAP SSH whenever the image tag, the deploy
   files or a secret changes (`scripts/remote-redeploy.sh`).
5. `terraform_data.health` — `apply` returns only when the public `/health` answers.

## After apply

```bash
terraform output public_origin                     # https://<ip-with-dashes>.sslip.io  (user: admin)
$(terraform output -raw admin_password_command)    # web UI admin password
terraform output agents_url                        # https://<ip-with-dashes>.sslip.io/agents (same sign-in)
$(terraform output -raw ssh_iap_command)           # shell on the VM
```

**HTTPS without a domain**: with `hostname` empty the site is `https://<ip-with-dashes>.sslip.io`
(sslip.io resolves to the static IP, Caddy gets a real Let's Encrypt certificate). Own domain: point an
A record at `terraform output external_ip`, set `hostname`.

## Update, roll back, change a key

| Task | How |
|---|---|
| Ship a code change | `terraform apply` (new content tag → build → redeploy; data disk untouched) |
| Roll back | `terraform apply -var image_tag=<pushed tag> -var build_images=false` |
| Change an LLM key | edit `terraform.tfvars`, `terraform apply` (new secret version → redeploy) |
| Logs | on the VM: `sudo tail -f /var/log/edgequake-startup.log /var/log/edgequake-deploy.log`, `sudo docker logs -f edgequake-api` |

## What gets created (names `edgequake-*`)

- VM `edgequake-vm` (e2-standard-2, Debian 12, Shielded VM), 50 GB pd-balanced data disk + daily snapshot policy
- VPC/subnet, firewalls (IAP SSH from 35.235.240.0/20, :80/:443 public), static external IP
- Artifact Registry repo `edgequake`, GCS bucket `<project>-edgequake-artifacts` (deploy overlay, build sources/logs)
- Secret Manager: postgres password, JWT secret, bootstrap admin password, agents internal token, master API key, OpenAI/Mistral keys (`UNSET` when not given)
- Service accounts `edgequake-gce` (runtime: logs, metrics, secrets, registry read) and `edgequake-build` (image push)
- For you (+ `operator_members`): IAP tunnel, OS Login admin, secret read
- Optional (`enable_github_deploy = true`): GitHub WIF pool + deploy SA

State is **local** (`terraform.tfstate`, git-ignored, contains secrets — keep it safe). Shared state:
add `backend.tf` with `terraform { backend "gcs" {} }` and
`terraform init -backend-config=bucket=<bucket> -backend-config=prefix=edgequake`.

## Destroy

```bash
terraform destroy
```

Deletes everything above **including the data disk** (documents, graph, users) and the images. Daily
snapshots are deleted too when `keep_snapshots_on_destroy = false` was applied; with `true` (default,
safety net) they stay and bill until deleted. The setting is read from the last `apply` (Terraform
destroy-time rule): change it, `terraform apply`, then `terraform destroy`. Project APIs stay enabled.

## Cost (europe-west1, list prices, approximate)

| Item | e2-standard-2 | e2-medium |
| --- | --- | --- |
| VM, 24/7 | ~$54/mo | ~$27/mo |
| 50 GB data + 20 GB boot pd-balanced | ~$8/mo | ~$8/mo |
| Static IP (in use) | ~$4/mo | ~$4/mo |
| Snapshots, registry, secrets | ~$1–3/mo | ~$1–3/mo |
| **Total** | **~$65–70/mo** | **~$40/mo** |

Plus Cloud Build minutes (cloud mode), egress and LLM usage. e2-standard-2 (8 GB) is the default:
Postgres + AGE/pgvector, the Rust API, Next.js and the agents exceed e2-medium's 4 GB under ingestion.

## Optional wrappers

`deploy/gcp/up.sh` / `down.sh` call the same Terraform, adding an access check, keys from the repo
`.env` when `terraform.tfvars` has none, a confirmation prompt and the summary of URLs
(`up.sh --project ID [--build cloud|local] [--tag TAG] [--plan-only] [--yes]`,
`down.sh [--delete-snapshots] [--yes]`).
