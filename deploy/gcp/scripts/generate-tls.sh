#!/usr/bin/env bash
# Self-signed cert with IP SAN for Caddy until DNS/Let's Encrypt (LAW-148-13).
# Docker userland proxy + empty SNI on IP HTTPS make `tls internal` fail
# (caddyserver/caddy#6344, #6364): Caddy sees the container IP, not EDGEQUAKE_TLS_HOST.
set -euo pipefail

HOST="${EDGEQUAKE_TLS_HOST:?EDGEQUAKE_TLS_HOST required}"
DIR="${STACK_CERT_DIR:-/opt/edgequake/compose/certs}"
mkdir -p "${DIR}"
umask 077

if [[ -f "${DIR}/tls.crt" && -f "${DIR}/tls.key" ]]; then
  if openssl x509 -in "${DIR}/tls.crt" -noout -checkend 86400 >/dev/null 2>&1 \
    && openssl x509 -in "${DIR}/tls.crt" -noout -text | grep -q "${HOST}"; then
    exit 0
  fi
fi

SAN="IP:${HOST},IP:127.0.0.1,DNS:localhost"
openssl req -x509 -newkey rsa:2048 -sha256 -days 825 -nodes \
  -keyout "${DIR}/tls.key" -out "${DIR}/tls.crt" \
  -subj "/CN=${HOST}" \
  -addext "subjectAltName=${SAN}"
chmod 600 "${DIR}/tls.key"
chmod 644 "${DIR}/tls.crt"
chown root:root "${DIR}/tls.key" "${DIR}/tls.crt"
