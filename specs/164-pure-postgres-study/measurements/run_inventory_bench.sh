#!/usr/bin/env bash
# SPEC-164 inventory extension: OP-164-09..14 at SCALE-M only.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
MEAS="$ROOT/measurements"
CONTAINER="${SPEC164_PG_CONTAINER:-eqdoc-scratch}"
PGUSER="${SPEC164_PG_USER:-edgequake}"
PGDB="${SPEC164_PG_DB:-edgequake}"

psqlc() {
  docker exec -i "$CONTAINER" psql -U "$PGUSER" -d "$PGDB" -v ON_ERROR_STOP=1 "$@"
}

echo "== SPEC-164 inventory bench SCALE-M =="
docker cp "$MEAS/harness.sql" "$CONTAINER:/tmp/spec164_harness.sql"
docker exec "$CONTAINER" psql -U "$PGUSER" -d "$PGDB" -v ON_ERROR_STOP=1 \
  -f /tmp/spec164_harness.sql >/tmp/spec164_inv_harness.log 2>&1

# Fresh M load with source_ids
psqlc -c "DELETE FROM spec164.bench_runs WHERE scale_id='m_inv' OR op_id LIKE 'OP-164-1%';"
echo "-- load SCALE-M enriched"
psqlc -c "SELECT spec164.load_scale('m', 200000, 500000, 0);"

suffix=m
scale=m_inv

mapfile -t IDS50 < <(psqlc -At -c "SELECT node_id FROM spec164.nodes_m ORDER BY md5(node_id||'inv50') LIMIT 50")
arr50="$(printf "'%s'," "${IDS50[@]}" | sed 's/,$//')"
start1="$(psqlc -At -c "SELECT node_id FROM spec164.nodes_m ORDER BY md5(node_id||'inv') LIMIT 1")"
# BFS close set: 2-hop neighbors of start1 (cap 100)
set_ids="$(psqlc -At -c "SELECT array_to_string(spec164.plain_bfs('m','$start1',2,100), ''',''')")"
set_arr="'$set_ids'"
# source probes: doc:0 and a few chunks
probes="'doc:0','doc:0_0','doc:0_1','doc:1','doc:1_0'"

time_op() {
  local arm="$1" op="$2" sql_stmt="$3"
  psqlc -c "$sql_stmt" >/dev/null
  local i ms
  for i in 0 1 2 3 4 5; do
    ms="$(psqlc -At -c "SELECT spec164.time_ms(\$bench\$${sql_stmt}\$bench\$)")"
    if [[ "$i" -ge 1 ]]; then
      psqlc -c "SELECT spec164.record_run('$scale','$arm','$op',$i,$ms,NULL,NULL);" >/dev/null
    fi
  done
  local med
  med="$(psqlc -At -c "SELECT round(spec164.median_ms('$scale','$arm','$op')::numeric, 3);")"
  echo "   $arm $op median_ms=$med"
}

# OP-164-09 edges_for_node_set
time_op ARM-A OP-164-09 "SELECT spec164.age_edges_for_node_set(ARRAY[$set_arr])"
time_op ARM-B OP-164-09 "SELECT spec164.plain_edges_for_node_set('$suffix', ARRAY[$set_arr])"

# OP-164-10 search
time_op ARM-A OP-164-10 "SELECT spec164.age_search_nodes('n12', 50)"
time_op ARM-B OP-164-10 "SELECT spec164.plain_search_nodes('$suffix', 'n12', 50)"

# OP-164-11 popular
time_op ARM-A OP-164-11 "SELECT spec164.age_popular_nodes(50)"
time_op ARM-B OP-164-11 "SELECT spec164.plain_popular_nodes('$suffix', 50)"

# OP-164-12 source prefix GIN
time_op ARM-A OP-164-12 "SELECT spec164.age_source_prefix_discover(ARRAY[$probes], 5000)"
time_op ARM-B OP-164-12 "SELECT spec164.plain_source_prefix_discover('$suffix', ARRAY[$probes], 5000)"

# OP-164-13 list OFFSET
time_op ARM-A OP-164-13 "SELECT spec164.age_list_nodes(0, 50)"
time_op ARM-B OP-164-13 "SELECT spec164.plain_list_nodes('$suffix', 0, 50)"

# OP-164-14 keyset scan
time_op ARM-A OP-164-14 "SELECT spec164.age_scan_after('n100000', 500)"
time_op ARM-B OP-164-14 "SELECT spec164.plain_scan_after('$suffix', 'n100000', 500)"

# EXPLAIN samples
psqlc -c "EXPLAIN (ANALYZE, BUFFERS) SELECT spec164.age_edges_for_node_set(ARRAY[$set_arr]);" \
  >"$MEAS/explains/m_inv_ARM-A_edges_set.txt" 2>&1 || true
psqlc -c "EXPLAIN (ANALYZE, BUFFERS) SELECT spec164.plain_edges_for_node_set('m', ARRAY[$set_arr]);" \
  >"$MEAS/explains/m_inv_ARM-B_edges_set.txt" 2>&1 || true
psqlc -c "EXPLAIN (ANALYZE, BUFFERS) SELECT spec164.age_source_prefix_discover(ARRAY[$probes], 5000);" \
  >"$MEAS/explains/m_inv_ARM-A_lineage.txt" 2>&1 || true
psqlc -c "EXPLAIN (ANALYZE, BUFFERS) SELECT spec164.plain_source_prefix_discover('m', ARRAY[$probes], 5000);" \
  >"$MEAS/explains/m_inv_ARM-B_lineage.txt" 2>&1 || true

python3 - "$MEAS/inventory_results.json" <<'PY'
import json, sys, subprocess, os
out = sys.argv[1]
container = os.environ.get("SPEC164_PG_CONTAINER", "eqdoc-scratch")
user = os.environ.get("SPEC164_PG_USER", "edgequake")
db = os.environ.get("SPEC164_PG_DB", "edgequake")

def q(sql):
    return subprocess.check_output(
        ["docker", "exec", "-i", container, "psql", "-U", user, "-d", db, "-At", "-c", sql],
        text=True,
    ).strip()

rows = q("""
SELECT json_agg(row_to_json(t)) FROM (
  SELECT scale_id, arm_id, op_id,
         round(percentile_cont(0.5) WITHIN GROUP (ORDER BY elapsed_ms)::numeric, 3) AS median_ms,
         count(*) AS n
  FROM spec164.bench_runs
  WHERE scale_id = 'm_inv' AND run_ix >= 1
  GROUP BY 1,2,3 ORDER BY 3,2
) t;
""")
medians = json.loads(rows) if rows else []
by = {(r["op_id"], r["arm_id"]): float(r["median_ms"]) for r in medians}
ratios = []
all_ok = True
for op in [f"OP-164-{i:02d}" for i in range(9, 15)]:
    a, b = by.get((op, "ARM-A")), by.get((op, "ARM-B"))
    if a and b and a > 0:
        ratio = round(b / a, 3)
        ok = ratio <= 2.0
        if not ok:
            all_ok = False
        ratios.append({"op_id": op, "arm_a_ms": a, "arm_b_ms": b, "ratio_b_over_a": ratio, "within_2x": ok})
    else:
        all_ok = False
        ratios.append({"op_id": op, "error": "missing", "within_2x": False})

# H-164-07: 09 plain faster expected
r09 = next((r for r in ratios if r.get("op_id") == "OP-164-09"), {})
h07 = bool(r09.get("arm_b_ms") and r09.get("arm_a_ms") and r09["arm_b_ms"] < r09["arm_a_ms"])
r12 = next((r for r in ratios if r.get("op_id") == "OP-164-12"), {})
h08 = bool(r12.get("within_2x"))

doc = {
    "spec": "164-inventory",
    "scale": "m_inv (200k/500k)",
    "medians": medians,
    "ratios_b_over_a": ratios,
    "hypotheses": {
        "H-164-06": {"pass": all_ok, "detail": "OP-164-09..14 B<=2x A at M"},
        "H-164-07": {"pass": h07, "detail": "edges_for_node_set plain faster than AGE parent agtype"},
        "H-164-08": {"pass": h08, "detail": "lineage GIN plain within 2x AGE"},
    },
    "decision_amendment": "strengthen_migrate_feasible" if all_ok else "migrate_with_caveat",
}
with open(out, "w") as f:
    json.dump(doc, f, indent=2)
    f.write("\n")
print(json.dumps(doc, indent=2))
PY

echo "== wrote $MEAS/inventory_results.json =="
