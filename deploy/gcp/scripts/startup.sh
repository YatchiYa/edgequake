#!/usr/bin/env bash
# GCE startup: Docker CE, data disk, artifacts from GCS, first deploy.
# Docker CE: https://docs.docker.com/engine/install/debian/
set -euo pipefail

LOG=/var/log/edgequake-startup.log
exec > >(tee -a "${LOG}") 2>&1
echo "[$(date -Is)] edgequake startup begin"

export DEBIAN_FRONTEND=noninteractive
apt-get update
apt-get install -y ca-certificates curl gnupg python3 apt-transport-https openssl

if ! command -v gcloud >/dev/null 2>&1; then
  curl -fsSL https://packages.cloud.google.com/apt/doc/apt-key.gpg \
    | gpg --dearmor -o /usr/share/keyrings/cloud.google.gpg
  echo "deb [signed-by=/usr/share/keyrings/cloud.google.gpg] https://packages.cloud.google.com/apt cloud-sdk main" \
    > /etc/apt/sources.list.d/google-cloud-sdk.list
  apt-get update
  apt-get install -y google-cloud-cli
fi

if ! command -v docker >/dev/null 2>&1; then
  install -m 0755 -d /etc/apt/keyrings
  curl -fsSL https://download.docker.com/linux/debian/gpg -o /etc/apt/keyrings/docker.asc
  chmod a+r /etc/apt/keyrings/docker.asc
  echo "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.asc] https://download.docker.com/linux/debian $(. /etc/os-release && echo "$VERSION_CODENAME") stable" \
    > /etc/apt/sources.list.d/docker.list
  apt-get update
  apt-get install -y docker-ce docker-ce-cli containerd.io docker-compose-plugin
fi
systemctl enable --now docker

meta() {
  curl -sf -H "Metadata-Flavor: Google" \
    "http://metadata.google.internal/computeMetadata/v1/instance/attributes/${1}" || true
}

DATA_DISK="$(meta edgequake-data-disk)"
if [[ -z "${DATA_DISK}" ]]; then
  DATA_DISK="elitizon-db-data"
fi
DATA_DEV="/dev/disk/by-id/google-${DATA_DISK}"
MOUNT="/mnt/${DATA_DISK}"
mkdir -p "${MOUNT}"
if [[ -b "${DATA_DEV}" ]]; then
  if ! blkid "${DATA_DEV}" >/dev/null 2>&1; then
    mkfs.ext4 -F "${DATA_DEV}"
  fi
  if ! mountpoint -q "${MOUNT}"; then
    mount "${DATA_DEV}" "${MOUNT}"
  fi
  if ! grep -q "${DATA_DEV}" /etc/fstab; then
    echo "${DATA_DEV} ${MOUNT} ext4 defaults,nofail 0 2" >> /etc/fstab
  fi
else
  echo "WARNING: ${DATA_DEV} missing; using ${MOUNT} on boot disk"
fi
mkdir -p "${MOUNT}/pgdata"
chown 999:999 "${MOUNT}/pgdata" || true

export STACK_BUCKET="$(meta edgequake-bucket)"
export STACK_VERSION="$(meta edgequake-version)"
export STACK_HOSTNAME="$(meta edgequake-hostname)"
export STACK_PROJECT="$(meta edgequake-project)"
export STACK_PREFIX="$(meta edgequake-prefix)"
export STACK_PUBLIC_ORIGIN="$(meta edgequake-public-origin)"
export STACK_ACME_EMAIL="$(meta edgequake-acme-email)"
export STACK_INSTALL_ROOT="$(meta edgequake-install-root)"
export STACK_DATA_DISK="${DATA_DISK}"
if [[ -z "${STACK_PREFIX}" ]]; then
  STACK_PREFIX="edgequake"
fi
if [[ -z "${STACK_INSTALL_ROOT}" ]]; then
  STACK_INSTALL_ROOT="/opt/${STACK_PREFIX}"
fi

install -d -m 0755 "${STACK_INSTALL_ROOT}"
gsutil -m cp -r "gs://${STACK_BUCKET}/compose" "gs://${STACK_BUCKET}/scripts" "${STACK_INSTALL_ROOT}/"
chmod 0755 "${STACK_INSTALL_ROOT}/scripts/"*.sh

cat >/etc/sudoers.d/edgequake <<SUDO
# CD and operators: one release entrypoint. Do not grant sudo bash.
%google-sudoers ALL=(root) NOPASSWD: ${STACK_INSTALL_ROOT}/scripts/install-release.sh, ${STACK_INSTALL_ROOT}/scripts/deploy.sh, /tmp/edgequake-overlay/scripts/install-release.sh, /usr/bin/docker
SUDO
chmod 440 /etc/sudoers.d/edgequake

STACK_ENV_FILE="${STACK_INSTALL_ROOT}/.env" "${STACK_INSTALL_ROOT}/scripts/render-env.sh"
"${STACK_INSTALL_ROOT}/scripts/deploy.sh"

echo "[$(date -Is)] edgequake startup complete"
