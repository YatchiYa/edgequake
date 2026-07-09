#!/usr/bin/env bash
# SPEC-045 — Remove corrupted duplicate document rows from dev workspace.
#
# Deletes pending/failed zombie rows that share a title with a completed document,
# and pending rows whose entity_count was reconciled from another doc's graph.
#
# Usage:
#   ./scripts/spec045_cleanup_corrupted_documents.sh
#   BACKEND_URL=http://localhost:8090 WORKSPACE_ID=... TENANT_ID=... ./scripts/spec045_cleanup_corrupted_documents.sh

set -euo pipefail

BACKEND_URL="${BACKEND_URL:-http://localhost:8090}"
WORKSPACE_ID="${WORKSPACE_ID:-}"
TENANT_ID="${TENANT_ID:-}"

if [[ -z "$WORKSPACE_ID" || -z "$TENANT_ID" ]]; then
  echo "Resolving default workspace/tenant from ${BACKEND_URL}/health ..."
  HEALTH="$(curl -sf "${BACKEND_URL}/health")"
  WORKSPACE_ID="${WORKSPACE_ID:-$(echo "$HEALTH" | python3 -c 'import json,sys; print(json.load(sys.stdin).get("workspace_id","default"))')}"
  TENANT_ID="${TENANT_ID:-default}"
fi

HDR_WS=(-H "X-Workspace-ID: ${WORKSPACE_ID}" -H "X-Tenant-ID: ${TENANT_ID}")

echo "Listing documents (workspace=${WORKSPACE_ID}, tenant=${TENANT_ID}) ..."
DOCS_JSON="$(curl -sf "${BACKEND_URL}/api/v1/documents" "${HDR_WS[@]}")"

export DOCS_JSON
DELETE_IDS="$(python3 <<'PY'
import json, os, sys
from collections import defaultdict

data = json.loads(os.environ["DOCS_JSON"])
docs = data.get("documents") or []

by_title = defaultdict(list)
for d in docs:
    title = (d.get("title") or d.get("file_name") or "").strip()
    if title:
        by_title[title].append(d)

delete = []
for title, group in by_title.items():
    completed = [d for d in group if (d.get("status") or "").lower() in ("completed", "indexed")]
    pending = [d for d in group if (d.get("status") or "").lower() in ("pending", "queued", "failed")]
    if len(group) <= 1:
        continue
    if completed and pending:
        # Keep completed rows; drop pending zombies with same display title.
        delete.extend(d["id"] for d in pending)

# Also drop orphan pending rows with zero chunks and no cost (likely re-upload ghosts).
for d in docs:
    st = (d.get("status") or "").lower()
    if st not in ("pending", "queued", "failed"):
        continue
    chunks = int(d.get("chunk_count") or 0)
    cost = d.get("cost_usd")
    if chunks == 0 and (cost is None or cost == 0):
        if d["id"] not in delete:
            delete.append(d["id"])

for doc_id in delete:
    print(doc_id)
PY
)"

if [[ -z "$DELETE_IDS" ]]; then
  echo "No corrupted duplicate rows to delete."
  exit 0
fi

COUNT="$(echo "$DELETE_IDS" | wc -l | tr -d ' ')"
echo "Deleting ${COUNT} corrupted/zombie document(s) ..."
while IFS= read -r doc_id; do
  [[ -z "$doc_id" ]] && continue
  echo "  → DELETE ${doc_id}"
  curl -sf -X DELETE "${BACKEND_URL}/api/v1/documents/${doc_id}" "${HDR_WS[@]}" >/dev/null
done <<< "$DELETE_IDS"

echo "Done. Remaining documents:"
curl -sf "${BACKEND_URL}/api/v1/documents" "${HDR_WS[@]}" | python3 -c "
import json,sys
d=json.load(sys.stdin)
for row in d.get('documents',[]):
    st=row.get('status','?')
    ent=row.get('entity_count','-')
    title=row.get('title','?')
    print(f'  {st:10} entities={ent!s:>5}  {title}')
"
