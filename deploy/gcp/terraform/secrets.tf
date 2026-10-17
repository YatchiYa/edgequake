resource "random_password" "postgres" {
  length  = 32
  special = false
}

resource "random_password" "jwt" {
  length  = 48
  special = false
}

resource "random_password" "bootstrap_admin" {
  length  = 24
  special = false
}

resource "random_password" "agents_internal" {
  length  = 48
  special = false
}

resource "random_password" "master_api_key" {
  length  = 40
  special = false
}

locals {
  secret_ids = {
    postgres-password        = random_password.postgres.result
    jwt                      = random_password.jwt.result
    bootstrap-admin-password = random_password.bootstrap_admin.result
    master-api-key           = random_password.master_api_key.result
    agents-internal-token    = random_password.agents_internal.result
  }
}

resource "google_secret_manager_secret" "app" {
  for_each  = local.secret_ids
  secret_id = "${var.name_prefix}-${each.key}"
  project   = var.project_id
  labels    = local.labels

  replication {
    auto {}
  }

  depends_on = [google_project_service.required]
}

resource "google_secret_manager_secret_version" "app" {
  for_each    = local.secret_ids
  secret      = google_secret_manager_secret.app[each.key].id
  secret_data = each.value
}

resource "google_secret_manager_secret" "openai" {
  secret_id = "${var.name_prefix}-openai-api-key"
  project   = var.project_id
  labels    = local.labels

  replication {
    auto {}
  }

  depends_on = [google_project_service.required]
}

# "UNSET" placeholder until a key is passed (render-env.sh treats it as empty).
resource "google_secret_manager_secret_version" "openai" {
  secret      = google_secret_manager_secret.openai.id
  secret_data = var.openai_api_key != "" ? var.openai_api_key : "UNSET"

  lifecycle {
    ignore_changes = [enabled]
  }
}

resource "google_secret_manager_secret_iam_member" "gce_accessor" {
  for_each  = google_secret_manager_secret.app
  project   = var.project_id
  secret_id = each.value.secret_id
  role      = "roles/secretmanager.secretAccessor"
  member    = "serviceAccount:${google_service_account.gce.email}"
}

resource "google_secret_manager_secret_iam_member" "gce_openai" {
  project   = var.project_id
  secret_id = google_secret_manager_secret.openai.secret_id
  role      = "roles/secretmanager.secretAccessor"
  member    = "serviceAccount:${google_service_account.gce.email}"
}

resource "google_secret_manager_secret" "mistral" {
  secret_id = "${var.name_prefix}-mistral-api-key"
  project   = var.project_id
  labels    = local.labels

  replication {
    auto {}
  }

  depends_on = [google_project_service.required]
}

# "UNSET" placeholder until a key is passed (render-env.sh treats it as empty).
resource "google_secret_manager_secret_version" "mistral" {
  secret      = google_secret_manager_secret.mistral.id
  secret_data = var.mistral_api_key != "" ? var.mistral_api_key : "UNSET"

  lifecycle {
    ignore_changes = [enabled]
  }
}

resource "google_secret_manager_secret_iam_member" "gce_mistral" {
  project   = var.project_id
  secret_id = google_secret_manager_secret.mistral.secret_id
  role      = "roles/secretmanager.secretAccessor"
  member    = "serviceAccount:${google_service_account.gce.email}"
}

resource "google_secret_manager_secret_iam_member" "operators_secret_accessor" {
  for_each = {
    for pair in setproduct(keys(google_secret_manager_secret.app), local.operators) :
    "${pair[0]}-${pair[1]}" => {
      secret_key = pair[0]
      member     = pair[1]
    }
  }
  project   = var.project_id
  secret_id = google_secret_manager_secret.app[each.value.secret_key].secret_id
  role      = "roles/secretmanager.secretAccessor"
  member    = each.value.member
}
