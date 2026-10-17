output "project_id" {
  value = var.project_id
}

output "external_ip" {
  value = google_compute_address.ip.address
}

output "public_origin" {
  value = local.public_origin
}

output "instance_name" {
  value = google_compute_instance.gce.name
}

output "zone" {
  value = var.zone
}

output "ssh_iap_command" {
  value = "gcloud compute ssh ${google_compute_instance.gce.name} --zone=${var.zone} --project=${var.project_id} --tunnel-through-iap"
}

output "health_url_https" {
  value = "${local.public_origin}/health"
}

output "artifacts_bucket" {
  value = google_storage_bucket.artifacts.name
}

output "image_registry" {
  value = local.image_registry
}

output "image_tag" {
  value = local.image_tag
}

output "build_service_account" {
  value = google_service_account.build.email
}

output "admin_username" {
  value = "admin"
}

output "admin_password_command" {
  value = "gcloud secrets versions access latest --secret=${google_secret_manager_secret.app["bootstrap-admin-password"].secret_id} --project=${var.project_id}"
}

output "workload_identity_provider" {
  value = var.enable_github_deploy ? google_iam_workload_identity_pool_provider.github["enabled"].name : null
}

output "github_service_account" {
  value = var.enable_github_deploy ? google_service_account.github["enabled"].email : null
}

output "gce_service_account" {
  value = google_service_account.gce.email
}

output "secret_ids" {
  value = {
    postgres                 = google_secret_manager_secret.app["postgres-password"].secret_id
    jwt                      = google_secret_manager_secret.app["jwt"].secret_id
    bootstrap_admin_password = google_secret_manager_secret.app["bootstrap-admin-password"].secret_id
    master_api_key           = google_secret_manager_secret.app["master-api-key"].secret_id
    openai_api_key           = google_secret_manager_secret.openai.secret_id
    mistral_api_key          = google_secret_manager_secret.mistral.secret_id
  }
}

output "agents_url" {
  description = "Agents platform (ChatGPT-style), same EdgeQuake sign-in"
  value       = "${local.public_origin}/agents"
}
