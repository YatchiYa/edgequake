#!/usr/bin/env bash
# Load edgequake-* instance metadata into the environment.
set -euo pipefail

meta() {
  curl -sf -H "Metadata-Flavor: Google" \
    "http://metadata.google.internal/computeMetadata/v1/instance/attributes/${1}" || true
}

export STACK_BUCKET="$(meta edgequake-bucket)"
export STACK_VERSION="$(meta edgequake-version)"
export STACK_HOSTNAME="$(meta edgequake-hostname)"
export STACK_PROJECT="$(meta edgequake-project)"
export STACK_PREFIX="$(meta edgequake-prefix)"
export STACK_PUBLIC_ORIGIN="$(meta edgequake-public-origin)"
export STACK_ACME_EMAIL="$(meta edgequake-acme-email)"
export STACK_INSTALL_ROOT="$(meta edgequake-install-root)"
export STACK_DATA_DISK="$(meta edgequake-data-disk)"

if [[ -z "${STACK_PREFIX}" ]]; then
  STACK_PREFIX="edgequake"
fi
if [[ -z "${STACK_INSTALL_ROOT}" ]]; then
  STACK_INSTALL_ROOT="/opt/${STACK_PREFIX}"
fi
