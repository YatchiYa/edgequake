#!/usr/bin/env bash
# Re-run the VM startup (render .env from Secret Manager, pull pinned images,
# migrate, compose up) over IAP SSH. Called by terraform_data.redeploy whenever the
# image tag, the deploy files or the secrets change.
#   remote-redeploy.sh VM ZONE PROJECT
set -euo pipefail

VM="$1" ZONE="$2" PROJECT="$3"
SSH=(gcloud compute ssh "${VM}" --zone="${ZONE}" --project="${PROJECT}" --tunnel-through-iap --quiet)

# A fresh VM needs a minute before IAP SSH answers (sshd, OS Login keys).
for attempt in $(seq 1 30); do
  if "${SSH[@]}" --command=true >/dev/null 2>&1; then
    break
  fi
  [[ "${attempt}" -eq 30 ]] && { echo "IAP SSH to ${VM} never became available" >&2; exit 1; }
  sleep 10
done

# Wait out a first-boot startup still in progress, then run it again (idempotent).
# ponytail: on first create this deploys twice (~1-2 min extra); skip it by comparing
# the running tag if that ever matters.
# (Check the boot unit, not pgrep -f: that would match this very SSH command line.)
"${SSH[@]}" --command='while systemctl is-active --quiet google-startup-scripts.service; do sleep 5; done; sudo google_metadata_script_runner startup'
