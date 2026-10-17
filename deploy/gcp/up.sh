#!/usr/bin/env bash
# Optional convenience wrapper. The deploy itself is plain Terraform:
#   cd deploy/gcp/terraform && terraform init && terraform apply
# This adds: access checks, LLM keys from <repo>/.env when terraform.tfvars has none,
# a confirmation, and the URLs + credential commands at the end.
#   deploy/gcp/up.sh [--project ID] [--build cloud|local] [--tag TAG] [--plan-only] [--yes]
set -euo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"
TF=(terraform -chdir="${HERE}/terraform")
REPO_ROOT="$(cd "${HERE}/../.." && pwd)"
PLAN_ONLY=0 YES=0 PROJECT_ARG=""
VARS=() # -var flags: beat terraform.tfvars (TF_VAR_* env would not)

die() { echo "up.sh: $*" >&2; exit 1; }

while [[ $# -gt 0 ]]; do
  case "$1" in
    --project) PROJECT_ARG="$2"; shift 2 ;;
    --build) VARS+=(-var "build_mode=$2"); shift 2 ;;
    --tag) VARS+=(-var "image_tag=$2" -var "build_images=false"); shift 2 ;;
    --plan-only) PLAN_ONLY=1; shift ;;
    --yes|-y) YES=1; shift ;;
    -h|--help) sed -n '2,6p' "$0"; exit 0 ;;
    *) die "unknown option: $1" ;;
  esac
done

command -v gcloud >/dev/null || die "gcloud not found (https://cloud.google.com/sdk/docs/install)"
command -v terraform >/dev/null || die "terraform >= 1.5 not found"
ACCOUNT="$(gcloud auth list --filter=status:ACTIVE --format='value(account)' 2>/dev/null | head -1)"
[[ -n "${ACCOUNT}" ]] || die "no active gcloud account; run: gcloud auth login"
gcloud auth application-default print-access-token >/dev/null 2>&1 \
  || die "Terraform needs Application Default Credentials; run: gcloud auth application-default login"

TFVARS_PROJECT="$(sed -n 's/^project_id *= *"\(.*\)"/\1/p' "${HERE}/terraform/terraform.tfvars" 2>/dev/null || true)"
PROJECT="${PROJECT_ARG:-${TFVARS_PROJECT:-$(gcloud config get-value project 2>/dev/null || true)}}"
[[ -n "${PROJECT}" ]] || die "no project; pass --project ID or set project_id in terraform/terraform.tfvars"
VARS+=(-var "project_id=${PROJECT}")
gcloud projects describe "${PROJECT}" --format='value(projectId)' >/dev/null 2>&1 \
  || die "${ACCOUNT} cannot access project '${PROJECT}' (wrong id, or missing IAM: needs Owner, or Editor + Security Admin + Service Usage Admin)"
echo "account: ${ACCOUNT}  project: ${PROJECT}"

# LLM keys from the repo .env, only when not already given (tfvars wins over TF_VAR_*).
env_val() { [[ -f "${REPO_ROOT}/.env" ]] && sed -n "s/^${1}=//p" "${REPO_ROOT}/.env" | tail -1 | sed -e 's/^["'\'']//' -e 's/["'\'']$//'; }
for key in MISTRAL_API_KEY OPENAI_API_KEY; do
  var="TF_VAR_$(echo "${key}" | tr '[:upper:]' '[:lower:]')"
  if [[ -z "${!var:-}" ]] && val="$(env_val "${key}")" && [[ -n "${val}" ]]; then
    export "${var}=${val}"
  fi
done

"${TF[@]}" init -input=false >/dev/null
"${TF[@]}" plan -input=false "${VARS[@]}" -out=tfplan
[[ "${PLAN_ONLY}" -eq 1 ]] && { rm -f "${HERE}/terraform/tfplan"; exit 0; }
if [[ "${YES}" -ne 1 ]]; then
  read -r -p "Apply this plan to ${PROJECT}? Type yes: " answer
  [[ "${answer}" == "yes" ]] || { echo "aborted"; rm -f "${HERE}/terraform/tfplan"; exit 1; }
fi
# Builds images if the sources changed, creates/updates the stack, redeploys, waits for HTTPS.
"${TF[@]}" apply -input=false tfplan
rm -f "${HERE}/terraform/tfplan"

out() { "${TF[@]}" output -raw "$1"; }
cat <<EOF

EdgeQuake:   $(out public_origin)        (user: $(out admin_username))
  password:  $(out admin_password_command)
Agents:      $(out agents_url)        (same sign-in)
SSH:         $(out ssh_iap_command)
EOF
