variable "project_id" {
  type        = string
  description = "GCP project to deploy into (billing enabled)."
}

variable "region" {
  type    = string
  default = "europe-west1"
}

variable "zone" {
  type    = string
  default = "europe-west1-b"
}

variable "name_prefix" {
  type        = string
  description = "Prefix for every GCP resource name (bucket, secrets, SAs, firewalls, IP, AR repo)."
  default     = "edgequake"
}

variable "instance_name" {
  type        = string
  description = "GCE VM name. Empty = \"<name_prefix>-vm\"."
  default     = ""
}

variable "machine_type" {
  type = string
  # 2 vCPU / 8 GB: Postgres (AGE + pgvector) + Rust API + Next.js + Caddy.
  # e2-medium (4 GB, shared vCPU) works for a demo but swaps under ingestion.
  default = "e2-standard-2"
}

variable "boot_disk_gb" {
  type    = number
  default = 20
}

variable "data_disk_gb" {
  type    = number
  default = 50
}

variable "image_registry" {
  type        = string
  description = "Registry path for edgequake + edgequake-frontend images. Empty = this stack's Artifact Registry repo. \"ghcr.io/raphaelmansuy\" = upstream images (without this fork's changes)."
  default     = ""
}

variable "image_tag" {
  type        = string
  description = "Image tag to run. Empty (default) = content tag of this checkout, built by Terraform. Set it to pin/roll back to an already pushed tag (with build_images = false)."
  default     = ""
}

variable "build_images" {
  type        = bool
  description = "Build and push api/frontend/agents images for image_tag during apply (skipped when the tag is unchanged)."
  default     = true
}

variable "build_mode" {
  type        = string
  description = "cloud = Google Cloud Build (no local Docker needed); local = docker buildx on this machine (faster on a strong workstation, needs ~30 GB free disk)."
  default     = "cloud"
  validation {
    condition     = contains(["cloud", "local"], var.build_mode)
    error_message = "build_mode must be cloud or local."
  }
}

variable "keep_snapshots_on_destroy" {
  type        = bool
  description = "Keep the data disk's daily snapshots after terraform destroy (true = safety net, false = remove everything)."
  default     = true
}

variable "postgres_image_tag" {
  type        = string
  description = "Tag of ghcr.io/raphaelmansuy/edgequake-postgres (PG18 + AGE + pgvector). Unchanged in this fork."
  default     = "0.34.0"
}

variable "hostname" {
  type        = string
  description = "Public DNS name pointing at the static IP. Empty = <ip-with-dashes>.sslip.io (real Let's Encrypt cert, no domain needed)."
  default     = ""
}

variable "caddy_acme_email" {
  type        = string
  description = "Let's Encrypt contact. Empty = the account running Terraform."
  default     = ""
}

variable "operator_members" {
  type        = list(string)
  description = "Extra IAM members (user:… / group:…) granted IAP SSH, OS Login admin and secret read. The caller is always added."
  default     = []
}

variable "openai_api_key" {
  type      = string
  sensitive = true
  default   = ""
}

variable "mistral_api_key" {
  type      = string
  sensitive = true
  default   = ""
}

variable "copilot_llm_base_url" {
  type        = string
  description = "OpenAI-compatible endpoint for the WebUI copilot (/api/copilot)."
  default     = "https://api.mistral.ai/v1"
}

variable "copilot_llm_model" {
  type    = string
  default = "mistral-medium-latest"
}

variable "enable_github_deploy" {
  type        = bool
  description = "Create GitHub Actions WIF pool + deploy SA (keyless CD over IAP SSH)."
  default     = false
}

variable "github_repository" {
  type        = string
  description = "GitHub repo (org/name) allowed to impersonate the deploy SA via WIF."
  default     = ""
}

variable "github_allowed_ref" {
  type        = string
  description = "Only workflows on this ref may mint tokens."
  default     = "refs/heads/edgequake-main"
}

variable "enable_public_http_https" {
  type        = bool
  description = "Open :80 (redirect-only) and :443."
  default     = true
}
