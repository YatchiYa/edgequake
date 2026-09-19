terraform {
  required_version = ">= 1.5.0"

  required_providers {
    google = {
      source  = "hashicorp/google"
      version = "~> 6.0"
    }
    random = {
      source  = "hashicorp/random"
      version = "~> 3.6"
    }
  }

  # Existing shared bucket. Prefix stays on the SPEC-148 state path so apply
  # does not orphan live resources. GCP *resource* names are edgequake-* / elitizon-db.
  backend "gcs" {
    bucket = "saas-app-001-tf-state"
    prefix = "eq148"
  }
}

provider "google" {
  project = var.project_id
  region  = var.region
}
