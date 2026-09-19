#!/usr/bin/env bash
# CD entrypoint. Copies this overlay into /opt/edgequake, then LD-15 deploy.sh.
# Invoked as: sudo /tmp/edgequake-overlay/scripts/install-release.sh X.Y.Z
set -euo pipefail

if [[ "${EUID}" -ne 0 ]]; then
  echo "install-release.sh must run as root (sudo)" >&2
  exit 1
fi

VERSION="${1:?usage: install-release.sh X.Y.Z}"
if ! [[ "${VERSION}" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "version must be X.Y.Z" >&2
  exit 1
fi

SRC="$(cd "$(dirname "$0")/.." && pwd)"
if [[ ! -f "${SRC}/compose/docker-compose.yml" || ! -f "${SRC}/scripts/deploy.sh" ]]; then
  echo "overlay incomplete at ${SRC}" >&2
  exit 1
fi

install -d -m 0755 /opt/edgequake
rm -rf /opt/edgequake/compose
cp -a "${SRC}/compose" /opt/edgequake/compose

stage="$(mktemp -d)"
cp -a "${SRC}/scripts/." "${stage}/"
chmod 0755 "${stage}/"*.sh
rm -rf /opt/edgequake/scripts
mv "${stage}" /opt/edgequake/scripts

export EDGEQUAKE_PIN_VERSION="${VERSION}"
exec /opt/edgequake/scripts/deploy.sh
