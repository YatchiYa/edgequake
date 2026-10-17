#!/usr/bin/env bash
# Build + push this checkout's api and frontend images (linux/amd64) to the stack's
# Artifact Registry repo. Dockerfiles/contexts/build args mirror release-docker.yml.
#   build-images.sh --project ID [--region R] [--prefix P] [--tag T] [--local]
#   build-images.sh --print-tag      # tag a build of this checkout would get
#   build-images.sh --print-tag-json # same, as {"tag": "..."} for Terraform's external data source
# Normally run by Terraform (terraform_data.images) after the registry, build SA and
# bucket exist. --local uses your docker daemon instead of Cloud Build.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
PROJECT="" REGION="europe-west1" PREFIX="edgequake" TAG="" MODE="cloud"
MACHINE="${BUILD_MACHINE_TYPE:-e2-highcpu-32}"

BUILD_DIRS=(edgequake edgequake_webui agents .dockerignore)
# Without git (repo received as an archive): local build artefacts never enter a build.
NON_SOURCE=(-name node_modules -o -name target -o -name .next -o -name .venv -o -name .data -o -name __pycache__ -o -name .env -o -name '.env.*')

is_git() { git -C "${REPO_ROOT}" rev-parse --is-inside-work-tree >/dev/null 2>&1; }

# NUL-separated list of the files that make up a build.
source_files() {
  cd "${REPO_ROOT}"
  if is_git; then
    git ls-files -z -co --exclude-standard -- "${BUILD_DIRS[@]}" \
      | while IFS= read -r -d '' f; do if [[ -e "${f}" ]]; then printf '%s\0' "${f}"; fi; done
  else
    find "${BUILD_DIRS[@]}" \( "${NON_SOURCE[@]}" \) -prune -o -type f -print0 2>/dev/null | sort -z
  fi
}

# Content-addressed: same sources → same tag → Terraform skips the rebuild.
# Uncommitted changes in the build dirs add a hash of the diff + untracked files.
default_tag() {
  local tag dirty
  if ! is_git; then
    echo "src-$(cd "${REPO_ROOT}" && source_files | xargs -0 -r sha1sum | sha1sum | cut -c1-12)"
    return
  fi
  tag="$(git -C "${REPO_ROOT}" rev-parse --short=12 HEAD)"
  dirty="$(
    cd "${REPO_ROOT}"
    {
      git diff HEAD --binary -- "${BUILD_DIRS[@]}"
      git ls-files -z -o --exclude-standard -- "${BUILD_DIRS[@]}" | xargs -0 -r sha1sum
    } | sha1sum | cut -c1-10
  )"
  if [[ -n "$(git -C "${REPO_ROOT}" status --porcelain -- "${BUILD_DIRS[@]}")" ]]; then
    tag+="-dirty-${dirty}"
  fi
  echo "${tag}"
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --project) PROJECT="$2"; shift 2 ;;
    --region) REGION="$2"; shift 2 ;;
    --prefix) PREFIX="$2"; shift 2 ;;
    --tag) TAG="$2"; shift 2 ;;
    --local) MODE="local"; shift ;;
    --print-tag) default_tag; exit 0 ;;
    --print-tag-json) printf '{"tag":"%s"}\n' "$(default_tag)"; exit 0 ;;
    *) echo "unknown option: $1" >&2; exit 2 ;;
  esac
done

PROJECT="${PROJECT:-$(gcloud config get-value project 2>/dev/null)}"
[[ -n "${PROJECT}" ]] || { echo "--project is required" >&2; exit 2; }
TAG="${TAG:-$(default_tag)}"
REGISTRY="${REGION}-docker.pkg.dev/${PROJECT}/${PREFIX}"
API="${REGISTRY}/edgequake:${TAG}"
WEB="${REGISTRY}/edgequake-frontend:${TAG}"
AGENTS="${REGISTRY}/edgequake-agents:${TAG}"
FRONTEND_ARGS=(--build-arg NEXT_PUBLIC_API_URL=http://localhost:8080 --build-arg "NEXT_PUBLIC_APP_VERSION=${TAG}")

# Idempotent: a tag is content-addressed, so if all three images exist it is done.
if [[ -z "${FORCE_BUILD:-}" ]] \
  && gcloud artifacts docker images describe "${API}" >/dev/null 2>&1 \
  && gcloud artifacts docker images describe "${WEB}" >/dev/null 2>&1 \
  && gcloud artifacts docker images describe "${AGENTS}" >/dev/null 2>&1; then
  echo "images for ${TAG} already in ${REGISTRY}; skipping build (FORCE_BUILD=1 to rebuild)"
  exit 0
fi

echo "building ${API}, ${WEB} and ${AGENTS} (${MODE})"

if [[ "${MODE}" == "local" ]]; then
  gcloud auth configure-docker "${REGION}-docker.pkg.dev" --quiet
  docker buildx build --platform linux/amd64 --push -f "${REPO_ROOT}/edgequake/docker/Dockerfile" -t "${API}" "${REPO_ROOT}"
  docker buildx build --platform linux/amd64 --push -f "${REPO_ROOT}/edgequake_webui/Dockerfile" \
    "${FRONTEND_ARGS[@]}" -t "${WEB}" "${REPO_ROOT}"
  docker buildx build --platform linux/amd64 --push -t "${AGENTS}" "${REPO_ROOT}/agents"
  exit 0
fi

BUCKET="gs://${PROJECT}-${PREFIX}-artifacts"
WORK="$(mktemp -d)"
trap 'rm -rf "${WORK}"' EXIT

# Source = tracked + untracked-not-ignored files (includes uncommitted changes,
# never gitignored .env / target / node_modules). Skips tracked files deleted locally.
(cd "${REPO_ROOT}" && source_files | tar --null -T - -czf "${WORK}/source.tgz")

cat >"${WORK}/cloudbuild.yaml" <<EOF
steps:
  - id: api
    name: gcr.io/cloud-builders/docker
    waitFor: ["-"]
    env: ["DOCKER_BUILDKIT=1"]
    args: [build, --platform=linux/amd64, -f, edgequake/docker/Dockerfile, -t, "${API}", .]
  - id: frontend
    name: gcr.io/cloud-builders/docker
    waitFor: ["-"]
    env: ["DOCKER_BUILDKIT=1"]
    args: [build, --platform=linux/amd64, -f, edgequake_webui/Dockerfile,
           --build-arg, NEXT_PUBLIC_API_URL=http://localhost:8080,
           --build-arg, "NEXT_PUBLIC_APP_VERSION=${TAG}", -t, "${WEB}", .]
  - id: agents
    name: gcr.io/cloud-builders/docker
    waitFor: ["-"]
    dir: agents
    args: [build, --platform=linux/amd64, -t, "${AGENTS}", .]
images: ["${API}", "${WEB}", "${AGENTS}"]
timeout: 7200s
options:
  diskSizeGb: 200
EOF

submit() { # $1 = machine type ("" = Cloud Build default)
  gcloud builds submit "${WORK}/source.tgz" \
    --project="${PROJECT}" --region="${REGION}" \
    --config="${WORK}/cloudbuild.yaml" \
    ${1:+--machine-type="$1"} \
    --service-account="projects/${PROJECT}/serviceAccounts/${PREFIX}-build@${PROJECT}.iam.gserviceaccount.com" \
    --gcs-source-staging-dir="${BUCKET}/cloudbuild-source" \
    --gcs-log-dir="${BUCKET}/cloudbuild-logs" 2>&1 | tee "${WORK}/submit.log"
}

# New projects often lack quota for large build machines: fall back to smaller ones
# ("" = Cloud Build default). Fresh IAM grants can take a minute: retry those.
for machine in "${MACHINE}" e2-highcpu-8 ""; do
  for attempt in 1 2 3 4 5; do
    submit "${machine}" && exit 0
    grep -q "does not have access to the bucket" "${WORK}/submit.log" || break
    echo "IAM grant still propagating; retrying in 30s (${attempt}/5)"
    sleep 30
  done
  grep -q "quota restrictions" "${WORK}/submit.log" || exit 1
  echo "no Cloud Build quota for '${machine:-default}' in ${REGION}; trying a smaller machine"
done
exit 1
