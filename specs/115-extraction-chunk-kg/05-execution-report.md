# 05 — Execution Report (Live Mistral Small)

> Filled 2026-08-10. Artifacts under [measurements/](measurements/).

## What ran

| Step | Result |
|------|--------|
| `geometry_probe.py` (real LR F) | N=13/20/26 @1200/800/600 |
| LightRAG `run_lightrag_mistral.py` | **Arm C OK** — 13 chunks, U=367, E=318, 92.9 s |
| EQ `spec115_mistral_ingest` Arm B | **OK** — adaptive 800/66, N=16, M=584/322, U≈491/305 |
| EQ Arm A (fair) | **OK** after Mistral 503 retry — N=12, M=439/325, U≈375/320 |
| Postgres HTTP path | **Blocked** — Docker/OrbStack unavailable; used library pipeline path |

## Pins

```text
LLM:   mistral-small-latest
Embed: mistral-embed (1024)
Glean: 1
Caps:  40 ents / 100 records (both SUTs)
Text:  gold MD twin of papers/light_rag_2410.05779v3.pdf
```

## Scoreboard

```ascii
                    N     M_ent   M_rel    U_node   U_edge
  LR  F@1200 (C)   13     ~425    ~342      367      318
  EQ  fair  (A)    12      439     325      375      320
  EQ  prod  (B)    16      584     322      491      305
```

## Hypothesis outcomes

| ID | Verdict | Evidence |
|----|---------|----------|
| H-C1 | **Confirmed** | Product N=16 vs fair 12 (1.33×); F geometry 20 vs 13 (1.54×) |
| H-C2 | **Confirmed** | M_B/M_A = 1.33 = N_B/N_A |
| H-C3 | **Confirmed** | U_A/U_C = 1.02 — fair EQ ≈ LR |
| H-C4 | Deferred | Library path used Recursive (gold MD), not Pdf; F vs R N differs (13 vs 12) |
| H-C5 | **Rejected** on this sample | Fair pins do **not** show order-of-magnitude over-extract |

## Mode honesty

- Extract text = **pymupdf gold MD**, not live PDF vision parse (isolates chunk/extract).
- EQ U = name-normalized unique from pipeline extractions (approx merger), not AGE (Postgres down).
- LR U = NetworkX/graphml unique nodes after merge.
- EQ document-card vanity M is the mention column (SPEC-108 LAW-X1).

## Reproduce

```bash
python3 specs/115-extraction-chunk-kg/experiments/geometry_probe.py
python3 specs/115-extraction-chunk-kg/experiments/run_lightrag_mistral.py
# both arms:
cd edgequake && cargo run --example spec115_mistral_ingest --release
# fair only:
SPEC115_ONLY_ARM=A cargo run --example spec115_mistral_ingest --release
```
