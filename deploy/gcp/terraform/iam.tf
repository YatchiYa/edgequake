resource "google_service_account" "gce" {
  account_id   = "${var.name_prefix}-gce"
  display_name = "EdgeQuake runtime on ${local.instance_name}"
  project      = var.project_id

  depends_on = [google_project_service.required]
}

resource "google_project_iam_member" "gce_logging" {
  project = var.project_id
  role    = "roles/logging.logWriter"
  member  = "serviceAccount:${google_service_account.gce.email}"
}

resource "google_project_iam_member" "gce_monitoring" {
  project = var.project_id
  role    = "roles/monitoring.metricWriter"
  member  = "serviceAccount:${google_service_account.gce.email}"
}

resource "google_storage_bucket_iam_member" "gce_artifacts" {
  bucket = google_storage_bucket.artifacts.name
  role   = "roles/storage.objectViewer"
  member = "serviceAccount:${google_service_account.gce.email}"
}

resource "google_artifact_registry_repository_iam_member" "gce_reader" {
  project    = var.project_id
  location   = google_artifact_registry_repository.images.location
  repository = google_artifact_registry_repository.images.name
  role       = "roles/artifactregistry.reader"
  member     = "serviceAccount:${google_service_account.gce.email}"
}

# ── Cloud Build (scripts/build-images.sh) ────────────────────────────────────
resource "google_service_account" "build" {
  account_id   = "${var.name_prefix}-build"
  display_name = "EdgeQuake image builds (Cloud Build)"
  project      = var.project_id

  depends_on = [google_project_service.required]
}

resource "google_artifact_registry_repository_iam_member" "build_writer" {
  project    = var.project_id
  location   = google_artifact_registry_repository.images.location
  repository = google_artifact_registry_repository.images.name
  role       = "roles/artifactregistry.writer"
  member     = "serviceAccount:${google_service_account.build.email}"
}

resource "google_project_iam_member" "build_logging" {
  project = var.project_id
  role    = "roles/logging.logWriter"
  member  = "serviceAccount:${google_service_account.build.email}"
}

# Source tarball + logs live under gs://<artifacts>/cloudbuild-{source,logs}/.
resource "google_storage_bucket_iam_member" "build_source" {
  bucket = google_storage_bucket.artifacts.name
  role   = "roles/storage.objectAdmin"
  member = "serviceAccount:${google_service_account.build.email}"
}

# Cloud Build with a user-managed SA + custom staging/log bucket also needs
# storage.buckets.get, which objectAdmin lacks.
resource "google_storage_bucket_iam_member" "build_bucket_reader" {
  bucket = google_storage_bucket.artifacts.name
  role   = "roles/storage.legacyBucketReader"
  member = "serviceAccount:${google_service_account.build.email}"
}

# ── Operators (the caller + var.operator_members) ────────────────────────────
resource "google_project_iam_member" "operators_iap" {
  for_each = local.operators
  project  = var.project_id
  role     = "roles/iap.tunnelResourceAccessor"
  member   = each.value
}

resource "google_project_iam_member" "operators_oslogin" {
  for_each = local.operators
  project  = var.project_id
  role     = "roles/compute.osAdminLogin"
  member   = each.value
}

resource "google_service_account_iam_member" "operators_use_gce" {
  for_each           = local.operators
  service_account_id = google_service_account.gce.name
  role               = "roles/iam.serviceAccountUser"
  member             = each.value
}

resource "google_service_account_iam_member" "operators_use_build" {
  for_each           = local.operators
  service_account_id = google_service_account.build.name
  role               = "roles/iam.serviceAccountUser"
  member             = each.value
}
