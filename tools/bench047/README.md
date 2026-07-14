# SPEC-047 bench047 harness

Python tool to evaluate EdgeQuake (Mistral Small + mistral-embed, hybrid) on
[MMLongBench-Doc](https://github.com/mayubo2333/MMLongBench-Doc).

**License:** dataset is CC BY-NC 4.0 (research only).

```bash
cd tools/bench047
pip install -e .
export MISTRAL_API_KEY=... OPENAI_API_KEY=...
export EDGEQUAKE_API_URL=http://localhost:8090

bench047 download-qa
bench047 freeze-smoke
bench047 download-pdfs
bench047 doctor
bench047 smoke
cat ../../specs/047-rag-evaluation/e2e/artifacts/smoke/SUMMARY.md
```

Or from repo root: `make bench047-smoke`.
