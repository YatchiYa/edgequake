# 07 — Ops runbook (unblock before release)

> Partners can act **today** on EdgeQuake + Ollama without waiting for SPEC-113 code.

## Symptom

Chat / query / PDF vision fails against an Ollama model whose **name** contains a thinking-family substring (`qwen3`, `deepseek-r1`, `qwq`, …) with errors like:

```text
does not support thinking
```

Health may still show `llm_provider: true`. Failure is per-request on chat.

## Confirm root class (2 minutes)

```bash
# 1) What does Ollama say the model can do?
curl -s http://localhost:11434/api/show -d '{"model":"YOUR_MODEL"}' | jq '.capabilities'

# 2) Does Auto think break chat?
curl -s http://localhost:11434/api/chat -d '{
  "model": "YOUR_MODEL",
  "stream": false,
  "think": true,
  "messages": [{"role":"user","content":"ping"}]
}' | jq .

# 3) Does omit work?
curl -s http://localhost:11434/api/chat -d '{
  "model": "YOUR_MODEL",
  "stream": false,
  "messages": [{"role":"user","content":"ping"}]
}' | jq .
```

| Observation | Meaning |
|-------------|---------|
| caps **lack** `thinking`, `think:true` errors, omit works | #369 class — client must not send `think` |
| caps **include** `thinking`, both work | Different bug — collect EdgeQuake logs |

## Immediate workarounds (pick one)

### W1 — Alias rename (reporter-proven)

```bash
ollama cp YOUR_QWen3_VL_MODEL  vl-instruct-8b
```

Point EdgeQuake `OLLAMA_MODEL` / workspace model at `vl-instruct-8b`.

### W2 — Use a non-matching family name model

Prefer an instruct / non-thinking tag that Ollama documents for your task (for Qwen3-VL see upstream notes on instruct vs thinking variants).

### W3 — Set explicit low effort only if model supports think

If caps include `thinking` and you want control, set role/env effort per SPEC-109. If caps **lack** thinking, setting high effort may still be harmful on unfixed crates — prefer W1/W2 until Wave A ships.

### W4 — Switch provider temporarily

OpenAI / other provider for query while local VL stays on a safely named Ollama model for vision-only paths (hybrid mode).

## After SPEC-113 ships

1. Upgrade `edgequake-llm` (EdgeQuake release notes will pin version).
2. Remove alias workaround if desired — capability gate makes original names safe.
3. Verify:

```bash
curl -s http://localhost:11434/api/show -d '{"model":"qwen3-vl:…"}' | jq '.capabilities'
# chat via EdgeQuake UI /api/v1/query should succeed without rename
```

## Do / Don’t

| Do | Don’t |
|----|-------|
| Trust `capabilities` from `/api/show` | Assume every `qwen3*` id supports `think` |
| Prefer omit when unsure | Force `think:true` in proxies “to be safe” |
| File logs with model id + caps JSON | Open tickets as “Ollama broken” without show output |

## Related

- Issue: https://github.com/raphaelmansuy/edgequake/issues/369  
- Laws: [01-first-principles.md](01-first-principles.md)  
- Fix train: [04-fix-plan.md](04-fix-plan.md)
