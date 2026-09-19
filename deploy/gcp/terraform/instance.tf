resource "google_compute_disk" "data" {
  name    = "${var.instance_name}-data"
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
  name         = var.instance_name
  machine_type = var.machine_type
  zone         = var.zone
  project      = var.project_id
  labels       = local.labels
  tags         = [var.name_prefix, var.instance_name]

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
    device_name = "${var.instance_name}-data"
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
    edgequake-version       = var.edgequake_version
    edgequake-hostname      = var.hostname
    edgequake-project       = var.project_id
    edgequake-prefix        = var.name_prefix
    edgequake-public-origin = local.public_origin
    edgequake-acme-email    = var.caddy_acme_email
    edgequake-install-root  = "/opt/${var.name_prefix}"
    edgequake-data-disk     = google_compute_disk.data.name
    startup-script          = file("${path.module}/../scripts/startup.sh")
  }

  lifecycle {
    ignore_changes = [
      metadata["startup-script"],
    ]
  }

  depends_on = [
    google_storage_bucket_object.artifacts,
    google_secret_manager_secret_version.app,
    google_secret_manager_secret_iam_member.gce_accessor,
    google_compute_disk_resource_policy_attachment.data,
    google_project_iam_member.gce_logging,
  ]
}
