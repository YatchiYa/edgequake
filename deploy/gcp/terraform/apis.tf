# disable_on_destroy=false: this is a shared project (LAW-148-2).
resource "google_project_service" "required" {
  for_each = local.required_services

  project            = var.project_id
  service            = each.value
  disable_on_destroy = false
}
