#!/usr/bin/env bash
# Optional convenience wrapper around plain `terraform destroy` (asks for the project id).
#   deploy/gcp/down.sh [--project ID] [--delete-snapshots] [--yes]
# Removes the VM, data disk, IP, secrets, image registry (with its images), bucket, IAM.
# Daily snapshots go too when keep_snapshots_on_destroy = false (tfvars) or with
# --delete-snapshots. Project APIs stay enabled (harmless, free).
set -euo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"
TF=(terraform -chdir="${HERE}/terraform")
YES=0 DELETE_SNAPSHOTS=0 PROJECT_ARG=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --project) PROJECT_ARG="$2"; shift 2 ;;
    --delete-snapshots) DELETE_SNAPSHOTS=1; shift ;;
    --yes|-y) YES=1; shift ;;
    *) echo "down.sh: unknown option: $1" >&2; exit 2 ;;
  esac
done

"${TF[@]}" init -input=false >/dev/null
PROJECT="$("${TF[@]}" output -raw project_id 2>/dev/null || echo "${PROJECT_ARG}")"
[[ -n "${PROJECT}" ]] || { echo "down.sh: nothing deployed (no state) and no --project" >&2; exit 2; }
VARS=(-var "project_id=${PROJECT}")

echo "This deletes the EdgeQuake stack in ${PROJECT}, including the database disk (documents, graph, users)."
if [[ "${YES}" -ne 1 ]]; then
  read -r -p "Type the project id to confirm: " answer
  [[ "${answer}" == "${PROJECT}" ]] || { echo "aborted"; exit 1; }
fi

if [[ "${DELETE_SNAPSHOTS}" -eq 1 ]]; then
  # Destroy-time steps read the value recorded at the last apply: record it first.
  "${TF[@]}" apply -input=false -auto-approve -target=terraform_data.snapshot_cleanup \
    "${VARS[@]}" -var keep_snapshots_on_destroy=false >/dev/null
fi
"${TF[@]}" destroy -input=false -auto-approve "${VARS[@]}"
