#!/usr/bin/env bash
# CD entrypoint. Copies this overlay into the install root, then LD-15 deploy.sh.
# Invoked as: sudo /tmp/edgequake-overlay/scripts/install-release.sh <image-tag>
# On the VM itself: sudo /opt/edgequake/scripts/install-release.sh <image-tag>
set -euo pipefail

if [[ "${EUID}" -ne 0 ]]; then
  echo "install-release.sh must run as root (sudo)" >&2
  exit 1
fi

VERSION="${1:?usage: install-release.sh <image-tag>}"
if ! [[ "${VERSION}" =~ ^[A-Za-z0-9_][A-Za-z0-9_.-]{0,127}$ ]] || [[ "${VERSION}" == latest ]]; then
  echo "image tag must be a pinned docker tag (not latest)" >&2
  exit 1
fi

SRC="$(cd "$(dirname "$0")/.." && pwd)"
if [[ ! -f "${SRC}/compose/docker-compose.yml" || ! -f "${SRC}/scripts/deploy.sh" ]]; then
  echo "overlay incomplete at ${SRC}" >&2
  exit 1
fi

# shellcheck disable=SC1091
source "${SRC}/scripts/load-metadata.sh"
ROOT="${STACK_INSTALL_ROOT}"

if [[ "${SRC}" != "${ROOT}" ]]; then
  install -d -m 0755 "${ROOT}"
  rm -rf "${ROOT}/compose"
  cp -a "${SRC}/compose" "${ROOT}/compose"

  stage="$(mktemp -d)"
  cp -a "${SRC}/scripts/." "${stage}/"
  chmod 0755 "${stage}/"*.sh
  rm -rf "${ROOT}/scripts"
  mv "${stage}" "${ROOT}/scripts"
fi

export EDGEQUAKE_PIN_VERSION="${VERSION}"
exec "${ROOT}/scripts/deploy.sh"
