#!/usr/bin/env bash
# Render <install-root>/.env from instance metadata + Secret Manager.
# Secrets are never logged.
set -euo pipefail

PREFIX="${STACK_PREFIX:-edgequake}"
PROJECT="${STACK_PROJECT:-}"
ORIGIN="${STACK_PUBLIC_ORIGIN:-}"
VERSION="${STACK_VERSION:-0.34.0}"
REGISTRY="${STACK_REGISTRY:-ghcr.io/raphaelmansuy}"
POSTGRES_TAG="${STACK_POSTGRES_TAG:-0.34.0}"
COPILOT_URL="${STACK_COPILOT_URL:-https://api.mistral.ai/v1}"
COPILOT_MODEL="${STACK_COPILOT_MODEL:-mistral-medium-latest}"
HOSTNAME="${STACK_HOSTNAME:-}"
ACME_EMAIL="${STACK_ACME_EMAIL:-}"
INSTALL_ROOT="${STACK_INSTALL_ROOT:-/opt/${PREFIX}}"
DATA_DISK="${STACK_DATA_DISK:-edgequake-vm-data}"

if [[ -z "${PROJECT}" || -z "${ORIGIN}" ]]; then
  if [[ -f "${INSTALL_ROOT}/scripts/load-metadata.sh" ]]; then
    # shellcheck disable=SC1091
    source "${INSTALL_ROOT}/scripts/load-metadata.sh"
    PROJECT="${STACK_PROJECT:-${PROJECT}}"
    ORIGIN="${STACK_PUBLIC_ORIGIN:-${ORIGIN}}"
    VERSION="${STACK_VERSION:-${VERSION}}"
    REGISTRY="${STACK_REGISTRY:-${REGISTRY}}"
    POSTGRES_TAG="${STACK_POSTGRES_TAG:-${POSTGRES_TAG}}"
    COPILOT_URL="${STACK_COPILOT_URL:-${COPILOT_URL}}"
    COPILOT_MODEL="${STACK_COPILOT_MODEL:-${COPILOT_MODEL}}"
    HOSTNAME="${STACK_HOSTNAME:-${HOSTNAME}}"
    ACME_EMAIL="${STACK_ACME_EMAIL:-${ACME_EMAIL}}"
    PREFIX="${STACK_PREFIX:-${PREFIX}}"
    INSTALL_ROOT="${STACK_INSTALL_ROOT:-${INSTALL_ROOT}}"
    DATA_DISK="${STACK_DATA_DISK:-${DATA_DISK}}"
  fi
fi
if [[ -z "${PROJECT}" ]]; then
  PROJECT="$(curl -sf -H "Metadata-Flavor: Google" \
    http://metadata.google.internal/computeMetadata/v1/project/project-id)"
fi
if [[ -z "${ORIGIN}" ]]; then
  echo "render-env.sh: STACK_PUBLIC_ORIGIN is empty" >&2
  exit 1
fi

# File cert is for the public IP (empty SNI). The hostname uses Let's Encrypt.
EXTERNAL_IP="$(curl -sf -H "Metadata-Flavor: Google" \
  "http://metadata.google.internal/computeMetadata/v1/instance/network-interfaces/0/access-configs/0/external-ip" || true)"
TLS_HOST="${EXTERNAL_IP:-${ORIGIN#https://}}"

access_secret() {
  local name="$1"
  gcloud secrets versions access latest --secret="${name}" --project="${PROJECT}"
}

POSTGRES_PASSWORD="$(access_secret "${PREFIX}-postgres-password")"
JWT_SECRET="$(access_secret "${PREFIX}-jwt")"
BOOTSTRAP_ADMIN_PASSWORD="$(access_secret "${PREFIX}-bootstrap-admin-password")"
MASTER_API_KEY="$(access_secret "${PREFIX}-master-api-key")"
OPENAI_API_KEY="$(access_secret "${PREFIX}-openai-api-key" || true)"
MISTRAL_API_KEY="$(access_secret "${PREFIX}-mistral-api-key" || true)"
# Shared secret: only the web app (after checking the EdgeQuake session) may call the agents /v1 API.
AGENTS_INTERNAL_TOKEN="$(access_secret "${PREFIX}-agents-internal-token")"
# Placeholder version is "UNSET" (Terraform). The server forbids mock as the
# default unless EDGEQUAKE_ALLOW_MOCK_PROVIDER=1 (smoke-test hatch, no LLM key).
LLM_PROVIDER="mock"
ALLOW_MOCK="1"
if [[ -z "${OPENAI_API_KEY}" || "${OPENAI_API_KEY}" == "UNSET" ]]; then
  OPENAI_API_KEY=""
fi
if [[ -z "${MISTRAL_API_KEY}" || "${MISTRAL_API_KEY}" == "UNSET" ]]; then
  MISTRAL_API_KEY=""
fi
# Models must match the provider: an OpenAI model name sent to Mistral fails every
# embedding call (and the Default workspace records whatever is set at first boot).
LLM_MODEL=""
EMBEDDING_MODEL=""
if [[ -n "${OPENAI_API_KEY}" ]]; then
  LLM_PROVIDER="openai"
  ALLOW_MOCK=""
  LLM_MODEL="gpt-4o-mini"
  EMBEDDING_MODEL="text-embedding-3-small"
elif [[ -n "${MISTRAL_API_KEY}" ]]; then
  LLM_PROVIDER="mistral"
  ALLOW_MOCK=""
  LLM_MODEL="mistral-small-latest"
  EMBEDDING_MODEL="mistral-embed"
fi

# WebUI copilot: Mistral key for the Mistral endpoint, OpenAI key otherwise.
COPILOT_KEY="${OPENAI_API_KEY}"
if [[ "${COPILOT_URL}" == *api.mistral.ai* ]]; then
  COPILOT_KEY="${MISTRAL_API_KEY}"
fi

ENV_FILE="${STACK_ENV_FILE:-${INSTALL_ROOT}/.env}"
umask 077
cat >"${ENV_FILE}" <<EOF
EDGEQUAKE_IMAGE_REGISTRY=${REGISTRY}
EDGEQUAKE_VERSION=${VERSION}
EDGEQUAKE_POSTGRES_TAG=${POSTGRES_TAG}
POSTGRES_PASSWORD=${POSTGRES_PASSWORD}
JWT_SECRET=${JWT_SECRET}
EDGEQUAKE_BOOTSTRAP_ADMIN_USERNAME=admin
EDGEQUAKE_BOOTSTRAP_ADMIN_PASSWORD=${BOOTSTRAP_ADMIN_PASSWORD}
EDGEQUAKE_BOOTSTRAP_ADMIN_EMAIL=admin@example.com
EDGEQUAKE_MASTER_API_KEY=${MASTER_API_KEY}
EDGEQUAKE_API_URL=${ORIGIN}
EDGEQUAKE_PUBLIC_URL=${ORIGIN}
EDGEQUAKE_CORS_ORIGINS=${ORIGIN}
EDGEQUAKE_TLS_HOST=${TLS_HOST}
EDGEQUAKE_DATA_MOUNT=/mnt/${DATA_DISK}
EDGEQUAKE_DEV_MODE=false
EDGEQUAKE_AUTH_ENABLED=true
EDGEQUAKE_LLM_PROVIDER=${LLM_PROVIDER}
EDGEQUAKE_LLM_MODEL=${LLM_MODEL}
EDGEQUAKE_EMBEDDING_PROVIDER=${LLM_PROVIDER}
EDGEQUAKE_EMBEDDING_MODEL=${EMBEDDING_MODEL}
EDGEQUAKE_VISION_PROVIDER=${LLM_PROVIDER}
EDGEQUAKE_ALLOW_MOCK_PROVIDER=${ALLOW_MOCK}
OPENAI_API_KEY=${OPENAI_API_KEY}
MISTRAL_API_KEY=${MISTRAL_API_KEY}
LLM_API_KEY=${MISTRAL_API_KEY}
COPILOT_LLM_BASE_URL=${COPILOT_URL}
COPILOT_LLM_API_KEY=${COPILOT_KEY}
COPILOT_LLM_MODEL=${COPILOT_MODEL}
EDGEQUAKE_HOSTNAME=${HOSTNAME}
CADDY_ACME_EMAIL=${ACME_EMAIL}
AGENTS_INTERNAL_TOKEN=${AGENTS_INTERNAL_TOKEN}
EOF
chmod 600 "${ENV_FILE}"
chown root:root "${ENV_FILE}"
