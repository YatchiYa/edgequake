resource "google_compute_network" "vpc" {
  # Leftover Option B already owns `edgequake-vpc` / `edgequake-subnet`. Do not import.
  name                    = "${var.name_prefix}-host-vpc"
  auto_create_subnetworks = false
  routing_mode            = "REGIONAL"
  project                 = var.project_id

  depends_on = [google_project_service.required]
}

resource "google_compute_subnetwork" "subnet" {
  name                     = "${var.name_prefix}-host-subnet"
  ip_cidr_range            = "10.48.0.0/24"
  region                   = var.region
  network                  = google_compute_network.vpc.id
  project                  = var.project_id
  private_ip_google_access = true
}

# IAP TCP forwarding source range — https://docs.cloud.google.com/iap/docs/using-tcp-forwarding
resource "google_compute_firewall" "iap_ssh" {
  name    = "${var.name_prefix}-allow-iap-ssh"
  network = google_compute_network.vpc.name
  project = var.project_id

  direction = "INGRESS"
  allow {
    protocol = "tcp"
    ports    = ["22"]
  }
  source_ranges = ["35.235.240.0/20"]
  target_tags   = [var.name_prefix]
}

resource "google_compute_firewall" "http" {
  count   = var.enable_public_http_https ? 1 : 0
  name    = "${var.name_prefix}-allow-http"
  network = google_compute_network.vpc.name
  project = var.project_id

  direction = "INGRESS"
  allow {
    protocol = "tcp"
    ports    = ["80"]
  }
  source_ranges = ["0.0.0.0/0"]
  target_tags   = [var.name_prefix]
}

resource "google_compute_firewall" "https" {
  count   = var.enable_public_http_https ? 1 : 0
  name    = "${var.name_prefix}-allow-https"
  network = google_compute_network.vpc.name
  project = var.project_id

  direction = "INGRESS"
  allow {
    protocol = "tcp"
    ports    = ["443"]
  }
  source_ranges = ["0.0.0.0/0"]
  target_tags   = [var.name_prefix]
}

resource "google_compute_address" "ip" {
  name         = "${var.name_prefix}-ip"
  region       = var.region
  project      = var.project_id
  address_type = "EXTERNAL"
}
