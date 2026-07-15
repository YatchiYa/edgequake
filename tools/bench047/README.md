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

### Stronger vision ablation (025 / W1)

Keep Small for **query** LLM; pin **Pass A/B** to `mistral-medium-3-5`:

```bash
make bench047-smoke-vision-medium
# or
python3 -m bench047.cli doctor --profile P0_mm_ite_vision_medium
python3 -m bench047.cli smoke --profile P0_mm_ite_vision_medium --document-scope --no-resume --i-accept-cost
```

Gate Acc claims on Chart **`answer_in_evidence_rate_long` ≥ 0.50**
(`bench047 fidelity` — full answerable audit, never `--max-samples` for gates).
Raw `a_in_e` is diagnostic only (short needles inflate). Also report **Chart exclusive Acc**
(`len(evidence_sources)==1`) alongside multi-label Chart Acc. Locked Acc chain remains
`P0_mm_ite` (Small+Small).

```bash
# Full-n fidelity (gateable)
python3 -m bench047.cli fidelity smoke

# Compare Acc + exclusive Chart + paired attribution + fidelity gates
python3 -m bench047.cli report path/to/run_a --compare path/to/baseline
```
