# Build → deploy → health, so that `terraform apply` alone brings the stack up and
# `terraform destroy` alone takes it down (images and, unless kept, snapshots too).

# Content-addressed tag of this checkout (git sha + hash of uncommitted build sources):
# unchanged sources → same tag → no rebuild, no redeploy.
data "external" "source_tag" {
  count   = var.image_tag == "" ? 1 : 0
  program = ["bash", "${path.module}/../scripts/build-images.sh", "--print-tag-json"]
}

locals {
  image_tag = var.image_tag != "" ? var.image_tag : data.external.source_tag[0].result.tag
}

# Builds and pushes api, frontend and agents images for local.image_tag.
resource "terraform_data" "images" {
  count            = var.build_images ? 1 : 0
  triggers_replace = [local.image_tag, local.image_registry]

  provisioner "local-exec" {
    command = join(" ", compact([
      "bash ${path.module}/../scripts/build-images.sh",
      "--project ${var.project_id} --region ${var.region} --prefix ${var.name_prefix}",
      "--tag ${local.image_tag}",
      var.build_mode == "local" ? "--local" : "",
    ]))
  }

  depends_on = [
    google_artifact_registry_repository_iam_member.build_writer,
    google_project_iam_member.build_logging,
    google_storage_bucket_iam_member.build_source,
    google_storage_bucket_iam_member.build_bucket_reader,
  ]
}

# Re-applies the stack on the VM when what it runs changes. The first boot is done by
# the startup script itself; this covers every later change without SSH by hand.
resource "terraform_data" "redeploy" {
  triggers_replace = {
    image    = "${local.image_registry}:${local.image_tag}"
    files    = sha1(join(",", [for k, o in google_storage_bucket_object.artifacts : "${k}=${o.md5hash}"]))
    secrets  = sha1(join(",", sort([for v in google_secret_manager_secret_version.app : v.id])))
    llm_keys = sha1(join(",", [google_secret_manager_secret_version.openai.id, google_secret_manager_secret_version.mistral.id]))
    instance = google_compute_instance.gce.instance_id
  }

  provisioner "local-exec" {
    command = "bash ${path.module}/../scripts/remote-redeploy.sh ${google_compute_instance.gce.name} ${var.zone} ${var.project_id}"
  }

  depends_on = [
    google_compute_firewall.iap_ssh,
    google_project_iam_member.operators_iap,
    google_project_iam_member.operators_oslogin,
  ]
}

# `apply` returns only once the public site answers over HTTPS.
resource "terraform_data" "health" {
  triggers_replace = [terraform_data.redeploy.id]

  provisioner "local-exec" {
    command = <<-EOT
      for i in $(seq 1 120); do
        curl -fsS --max-time 10 "${local.public_origin}/health" >/dev/null 2>&1 && { echo "healthy: ${local.public_origin}"; exit 0; }
        sleep 10
      done
      echo "not healthy after 20 min; logs: gcloud compute ssh ${google_compute_instance.gce.name} --zone=${var.zone} --project=${var.project_id} --tunnel-through-iap -- sudo tail -n 100 /var/log/edgequake-startup.log" >&2
      exit 1
    EOT
  }
}

# Snapshots outlive their disk (KEEP_AUTO_SNAPSHOTS). Unless kept, destroy removes them.
resource "terraform_data" "snapshot_cleanup" {
  input = {
    project = var.project_id
    keep    = var.keep_snapshots_on_destroy
    disk    = google_compute_disk.data.name
  }

  provisioner "local-exec" {
    when    = destroy
    command = <<-EOT
      if [ "${self.input.keep}" = "true" ]; then
        echo "keeping snapshots of ${self.input.disk} (keep_snapshots_on_destroy = true)"
        exit 0
      fi
      for s in $(gcloud compute snapshots list --project=${self.input.project} --filter="sourceDisk~/disks/${self.input.disk}$" --format="value(name)"); do
        gcloud compute snapshots delete "$s" --project=${self.input.project} --quiet
      done
    EOT
  }
}
