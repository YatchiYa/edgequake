# 00 — Issue data (#369)

> **Source:** [github.com/raphaelmansuy/edgequake/issues/369](https://github.com/raphaelmansuy/edgequake/issues/369)  
> **Fetched:** 2026-08-08 · **State:** open · **Label:** `bug` · **Author:** @ravimohta

## Bug title (reporter)

Edgequake invokes ollama with thinking parameters if the llm name has the word "qwen3" in it.

## Reproduction (reporter)

1. Pull any Qwen3-VL variant into Ollama (`ollama pull <qwen3-vl-model>`).
2. Configure EdgeQuake / `edgequake-llm` Ollama provider to use that model as chat, with **no** explicit `reasoning_effort`.
3. Send a chat request.
4. Ollama rejects: model does not support thinking.

## Expected

Any Ollama model configured with EdgeQuake should be allowed to work (no false-positive `think` injection).

## Environment (reporter)

| Item | Value |
|------|-------|
| Ollama | Recent (supports `/api/show` `capabilities`) |
| Crate | `edgequake-llm` **v0.10.4**, `OllamaProvider` |

## Confirming workaround (reporter)

```bash
ollama cp <original-qwen3-vl-model-name> vl-instruct-8b
```

Same weights / same Ollama behavior — only the **string** passed to `is_thinking_model()` changes. Requests succeed. This isolates the defect to the **name substring heuristic**, not the model blob.

## Reporter-suggested fix

Replace (or gate) substring `is_thinking_model()` with a real capability lookup:

- Prefer `POST /api/show` → `capabilities[]` contains `"thinking"`
- Cache per model
- Fall back to name heuristic only when capabilities unavailable (very old Ollama)

## Code cited in issue (matches local `edgequake-llm` HEAD + crates.io 0.10.4)

```rust
fn is_thinking_model(model: &str) -> bool {
    let model_lower = model.to_lowercase();
    model_lower.contains("deepseek-r1")
        || model_lower.contains("qwen3")
        || model_lower.contains("qwq")
        || model_lower.contains("openthinker")
        || model_lower.contains("phi4-reasoning")
        || model_lower.contains("magistral")
        || model_lower.contains("cogito")
        || model_lower.contains("gpt-oss")
}

// Auto path in resolve_think:
if desired.is_none() && Self::is_thinking_model(model) {
    Some(serde_json::Value::Bool(true))
} else {
    None
}
```

## Related upstream facts (research)

| Fact | Citation |
|------|----------|
| `/api/show` returns `capabilities` including `thinking` | [Ollama PR #10066](https://github.com/ollama/ollama/pull/10066), [Show model details](https://docs.ollama.com/api-reference/show-model-details) |
| Clients should query capabilities for think support | [ollama#10966](https://github.com/ollama/ollama/issues/10966) (`jq .capabilities`) |
| Capability constant `CapabilityThinking = "thinking"` | [capability.go](https://github.com/ollama/ollama/blob/main/types/model/capability.go) |
| Sending `think: true` to a non-thinking model errors | e.g. `"… does not support thinking"` ([ollama#10473](https://github.com/ollama/ollama/issues/10473)) |
| Qwen3-VL family is **not** a uniform think/toggle story | [ollama#16945](https://github.com/ollama/ollama/issues/16945), [ollama#14798](https://github.com/ollama/ollama/issues/14798) — use capabilities / correct variant, not name folklore |

## Product pin in this monorepo

| Pin | Value |
|-----|-------|
| Workspace dep | `edgequake-llm = "0.10.4"` (`edgequake/Cargo.toml`) |
| Local sibling clone (dev) | `/Users/raphaelmansuy/Github/03-working/edgequake-llm` |
| Registry copy | `~/.cargo/registry/.../edgequake-llm-0.10.4` |
