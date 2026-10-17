# Optional keyless CD from GitHub Actions (.github/workflows/deploy-gcp-edgequake.yml):
# WIF → deploy SA → IAP SSH → install-release.sh. Off unless enable_github_deploy = true.

locals {
  github = var.enable_github_deploy ? toset(["enabled"]) : toset([])
}

resource "google_service_account" "github" {
  for_each     = local.github
  account_id   = "${var.name_prefix}-github"
  display_name = "EdgeQuake GitHub Actions deploy (WIF)"
  project      = var.project_id

  depends_on = [google_project_service.required]
}

resource "google_iam_workload_identity_pool" "github" {
  for_each                  = local.github
  project                   = var.project_id
  workload_identity_pool_id = "${var.name_prefix}-github"
  display_name              = "EdgeQuake GitHub Actions"
  description               = "Keyless deploy to ${local.instance_name} via IAP SSH"

  depends_on = [google_project_service.required]
}

resource "google_iam_workload_identity_pool_provider" "github" {
  for_each                           = local.github
  project                            = var.project_id
  workload_identity_pool_id          = google_iam_workload_identity_pool.github[each.key].workload_identity_pool_id
  workload_identity_pool_provider_id = "github-oidc"
  display_name                       = "GitHub OIDC"
  attribute_mapping = {
    "google.subject"             = "assertion.sub"
    "attribute.actor"            = "assertion.actor"
    "attribute.repository"       = "assertion.repository"
    "attribute.repository_owner" = "assertion.repository_owner"
    "attribute.ref"              = "assertion.ref"
  }
  # Only var.github_allowed_ref of var.github_repository may mint tokens. Forks and other refs cannot.
  attribute_condition = "assertion.repository == \"${var.github_repository}\" && assertion.ref == \"${var.github_allowed_ref}\""
  oidc {
    issuer_uri = "https://token.actions.githubusercontent.com"
  }

  lifecycle {
    precondition {
      condition     = var.github_repository != ""
      error_message = "enable_github_deploy = true requires github_repository (org/name)."
    }
  }
}

resource "google_service_account_iam_member" "github_wif" {
  for_each           = local.github
  service_account_id = google_service_account.github[each.key].name
  role               = "roles/iam.workloadIdentityUser"
  member             = "principalSet://iam.googleapis.com/${google_iam_workload_identity_pool.github[each.key].name}/attribute.repository/${var.github_repository}"
}

resource "google_iap_tunnel_instance_iam_member" "github_iap" {
  for_each = local.github
  project  = var.project_id
  zone     = var.zone
  instance = google_compute_instance.gce.name
  role     = "roles/iap.tunnelResourceAccessor"
  member   = "serviceAccount:${google_service_account.github[each.key].email}"

  condition {
    title       = "ssh-port-22"
    description = "IAP TCP forwarding only to SSH on ${local.instance_name}"
    expression  = "destination.port == 22"
  }
}

resource "google_compute_instance_iam_member" "github_oslogin" {
  for_each      = local.github
  project       = var.project_id
  zone          = var.zone
  instance_name = google_compute_instance.gce.name
  role          = "roles/compute.osAdminLogin"
  member        = "serviceAccount:${google_service_account.github[each.key].email}"
}

resource "google_service_account_iam_member" "github_use_gce" {
  for_each           = local.github
  service_account_id = google_service_account.gce.name
  role               = "roles/iam.serviceAccountUser"
  member             = "serviceAccount:${google_service_account.github[each.key].email}"
}

resource "google_compute_instance_iam_member" "github_viewer" {
  for_each      = local.github
  project       = var.project_id
  zone          = var.zone
  instance_name = google_compute_instance.gce.name
  role          = "roles/compute.viewer"
  member        = "serviceAccount:${google_service_account.github[each.key].email}"
}

# gcloud compute ssh resolves the project before the instance. Instance-scoped
# compute.viewer does not include compute.projects.get — without this binding
# CD fails with: Required 'compute.projects.get' permission for 'projects/…'
resource "google_project_iam_member" "github_compute_viewer" {
  for_each = local.github
  project  = var.project_id
  role     = "roles/compute.viewer"
  member   = "serviceAccount:${google_service_account.github[each.key].email}"
}
