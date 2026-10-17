resource "google_storage_bucket" "artifacts" {
  name                        = "${var.project_id}-${var.name_prefix}-artifacts"
  location                    = var.region
  project                     = var.project_id
  uniform_bucket_level_access = true
  public_access_prevention    = "enforced"
  force_destroy               = true # deploy overlay + Cloud Build sources only, no user data
  labels                      = local.labels

  depends_on = [google_project_service.required]
}

resource "google_storage_bucket_object" "artifacts" {
  for_each = local.artifact_files
  bucket   = google_storage_bucket.artifacts.name
  name     = each.key
  source   = each.value
}

resource "google_artifact_registry_repository" "images" {
  project       = var.project_id
  location      = var.region
  repository_id = var.name_prefix
  format        = "DOCKER"
  description   = "EdgeQuake api + frontend images built from this checkout"
  labels        = local.labels

  depends_on = [google_project_service.required]
}
