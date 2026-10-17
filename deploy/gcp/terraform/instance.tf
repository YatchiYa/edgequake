resource "google_compute_disk" "data" {
  name    = "${local.instance_name}-data"
  type    = "pd-balanced"
  zone    = var.zone
  size    = var.data_disk_gb
  labels  = local.labels
  project = var.project_id

  lifecycle {
    prevent_destroy = false
  }

  depends_on = [google_project_service.required]
}

resource "google_compute_resource_policy" "daily_snapshot" {
  name    = "${var.name_prefix}-daily-snap"
  region  = var.region
  project = var.project_id

  snapshot_schedule_policy {
    schedule {
      daily_schedule {
        days_in_cycle = 1
        start_time    = "03:00"
      }
    }
    retention_policy {
      max_retention_days    = 14
      on_source_disk_delete = "KEEP_AUTO_SNAPSHOTS"
    }
    snapshot_properties {
      labels            = local.labels
      storage_locations = [var.region]
      guest_flush       = false
    }
  }
}

resource "google_compute_disk_resource_policy_attachment" "data" {
  name    = google_compute_resource_policy.daily_snapshot.name
  disk    = google_compute_disk.data.name
  zone    = var.zone
  project = var.project_id
}

resource "google_compute_instance" "gce" {
  name         = local.instance_name
  machine_type = var.machine_type
  zone         = var.zone
  project      = var.project_id
  labels       = local.labels
  tags         = [var.name_prefix, local.instance_name]

  allow_stopping_for_update = true

  boot_disk {
    initialize_params {
      image  = "debian-cloud/debian-12"
      size   = var.boot_disk_gb
      type   = "pd-balanced"
      labels = local.labels
    }
    auto_delete = true
  }

  attached_disk {
    source      = google_compute_disk.data.id
    device_name = "${local.instance_name}-data"
    mode        = "READ_WRITE"
  }

  network_interface {
    subnetwork = google_compute_subnetwork.subnet.id
    access_config {
      nat_ip = google_compute_address.ip.address
    }
  }

  scheduling {
    preemptible         = false
    automatic_restart   = true
    on_host_maintenance = "MIGRATE"
    provisioning_model  = "STANDARD"
  }

  shielded_instance_config {
    enable_secure_boot          = true
    enable_vtpm                 = true
    enable_integrity_monitoring = true
  }

  service_account {
    email  = google_service_account.gce.email
    scopes = ["cloud-platform"]
  }

  metadata = {
    enable-oslogin          = "TRUE"
    block-project-ssh-keys  = "TRUE"
    edgequake-bucket        = google_storage_bucket.artifacts.name
    edgequake-version       = local.image_tag
    edgequake-registry      = local.image_registry
    edgequake-postgres-tag  = var.postgres_image_tag
    edgequake-hostname      = local.hostname
    edgequake-project       = var.project_id
    edgequake-prefix        = var.name_prefix
    edgequake-public-origin = local.public_origin
    edgequake-acme-email    = local.acme_email
    edgequake-install-root  = "/opt/${var.name_prefix}"
    edgequake-data-disk     = google_compute_disk.data.name
    edgequake-copilot-url   = var.copilot_llm_base_url
    edgequake-copilot-model = var.copilot_llm_model
    startup-script          = file("${path.module}/../scripts/startup.sh")
  }

  depends_on = [
    terraform_data.images,
    google_storage_bucket_object.artifacts,
    google_secret_manager_secret_version.app,
    google_secret_manager_secret_version.openai,
    google_secret_manager_secret_version.mistral,
    google_artifact_registry_repository_iam_member.gce_reader,
    google_secret_manager_secret_iam_member.gce_accessor,
    google_compute_disk_resource_policy_attachment.data,
    google_project_iam_member.gce_logging,
  ]
}
