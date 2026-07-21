# bench001 — SPEC-001 Dual-SUT Harness

EdgeQuake `mix` vs LightRAG `mix` on **GraphRAG-Bench**.

```bash
pip3 install -e .
python3 -m bench001.cli doctor
python3 -m bench001.cli freeze-smoke
python3 -m bench001.cli smoke --dry-run          # plumbing
python3 -m bench001.cli smoke --api http://127.0.0.1:8080
```

Makefile: `make bench001-install|doctor|freeze-smoke|smoke|core`

Spec: [`specs/001-benchmark/`](../../specs/001-benchmark/)
