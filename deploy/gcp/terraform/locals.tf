data "google_client_openid_userinfo" "me" {}

# ADC tokens often lack the email scope (userinfo email = null): fall back to the
# active gcloud account so `terraform apply` works without any wrapper script.
data "external" "gcloud_account" {
  program = ["bash", "-c", "printf '{\"email\":\"%s\"}' \"$(gcloud config get-value account 2>/dev/null)\""]
}

locals {
  labels = {
    component                  = "edgequake"
    goog-terraform-provisioned = "true"
  }

  instance_name = var.instance_name != "" ? var.instance_name : "${var.name_prefix}-vm"

  # sslip.io resolves 1-2-3-4.sslip.io to 1.2.3.4, so Caddy gets a real Let's Encrypt cert without a domain.
  hostname      = var.hostname != "" ? var.hostname : "${replace(google_compute_address.ip.address, ".", "-")}.sslip.io"
  public_origin = "https://${local.hostname}"
  acme_email    = var.caddy_acme_email != "" ? var.caddy_acme_email : local.caller_email

  ar_registry    = "${var.region}-docker.pkg.dev/${var.project_id}/${google_artifact_registry_repository.images.repository_id}"
  image_registry = var.image_registry != "" ? var.image_registry : local.ar_registry

  caller_email  = coalesce(data.google_client_openid_userinfo.me.email, data.external.gcloud_account.result.email)
  caller_member = endswith(local.caller_email, ".gserviceaccount.com") ? "serviceAccount:${local.caller_email}" : "user:${local.caller_email}"
  operators     = toset(distinct(concat([local.caller_member], var.operator_members)))

  required_services = toset([
    "artifactregistry.googleapis.com",
    "cloudbuild.googleapis.com",
    "compute.googleapis.com",
    "iam.googleapis.com",
    "iamcredentials.googleapis.com",
    "iap.googleapis.com",
    "secretmanager.googleapis.com",
    "storage.googleapis.com",
    "sts.googleapis.com",
    "logging.googleapis.com",
    "monitoring.googleapis.com",
    "oslogin.googleapis.com",
  ])

  artifact_files = {
    "compose/docker-compose.yml" = "${path.module}/../compose/docker-compose.yml"
    "compose/Caddyfile"          = "${path.module}/../compose/Caddyfile"
    "compose/Caddyfile.hostname" = "${path.module}/../compose/Caddyfile.hostname"
    "compose/snippets.caddy"     = "${path.module}/../compose/snippets.caddy"
    "compose/.env.example"       = "${path.module}/../compose/.env.example"
    "scripts/deploy.sh"          = "${path.module}/../scripts/deploy.sh"
    "scripts/install-release.sh" = "${path.module}/../scripts/install-release.sh"
    "scripts/render-env.sh"      = "${path.module}/../scripts/render-env.sh"
    "scripts/load-metadata.sh"   = "${path.module}/../scripts/load-metadata.sh"
    "scripts/generate-tls.sh"    = "${path.module}/../scripts/generate-tls.sh"
  }
}
