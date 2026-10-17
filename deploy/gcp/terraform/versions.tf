terraform {
  required_version = ">= 1.5.0"

  required_providers {
    google = {
      source  = "hashicorp/google"
      version = "~> 6.0"
    }
    external = {
      source  = "hashicorp/external"
      version = "~> 2.3"
    }
    random = {
      source  = "hashicorp/random"
      version = "~> 3.6"
    }
  }

  # State is local (terraform.tfstate here). For shared GCS state add a backend.tf with
  # `terraform { backend "gcs" {} }` and run:
  #   terraform init -backend-config=bucket=<bucket> -backend-config=prefix=edgequake
}

provider "google" {
  project = var.project_id
  region  = var.region
  # User ADC credentials need a quota project; bill API calls to the target project.
  billing_project       = var.project_id
  user_project_override = true
}
