//! SPEC-109: resolve effective query/chat/vlm/extract reasoning effort for API handlers.

use edgequake_core::{current_defaults, resolve_role_reasoning_effort, LlmRole, Workspace};

/// Resolve clamped reasoning effort for the Extract role (KG entity extraction).
///
/// For Ollama thinking models the compiled floor is `"none"` so extract sends
/// `think: false` instead of Auto `think: true` (SPEC-113).
pub fn resolve_extract_reasoning_effort(
    workspace: Option<&Workspace>,
    provider: &str,
    model: &str,
    request_override: Option<&str>,
    tenant_default: Option<&str>,
) -> Option<String> {
    resolve_role_effort(
        LlmRole::Extract,
        workspace,
        provider,
        model,
        request_override,
        tenant_default,
    )
}

/// Resolve clamped reasoning effort for the Query role.
///
/// When `request_override` is set it wins; otherwise workspace/server/env/compiled apply.
/// Returns `None` when the model should omit the wire field (Auto / unsupported).
pub fn resolve_query_reasoning_effort(
    workspace: Option<&Workspace>,
    provider: &str,
    model: &str,
    request_override: Option<&str>,
    tenant_default: Option<&str>,
) -> Option<String> {
    resolve_role_effort(
        LlmRole::Query,
        workspace,
        provider,
        model,
        request_override,
        tenant_default,
    )
}

/// Resolve clamped reasoning effort for the Vlm (vision PDF) role (SPEC-109).
pub fn resolve_vlm_reasoning_effort(
    workspace: Option<&Workspace>,
    provider: &str,
    model: &str,
    request_override: Option<&str>,
    tenant_default: Option<&str>,
) -> Option<String> {
    resolve_role_effort(
        LlmRole::Vlm,
        workspace,
        provider,
        model,
        request_override,
        tenant_default,
    )
}

fn resolve_role_effort(
    role: LlmRole,
    workspace: Option<&Workspace>,
    provider: &str,
    model: &str,
    request_override: Option<&str>,
    tenant_default: Option<&str>,
) -> Option<String> {
    let defaults = current_defaults();
    let server_by_role = defaults
        .reasoning_by_role
        .get(role.as_str())
        .map(String::as_str);

    let mut empty = Workspace::new(uuid::Uuid::nil(), "resolve", "resolve");
    if let Some(ws) = workspace {
        empty = ws.clone();
    } else {
        empty.llm_provider = provider.to_string();
        empty.llm_model = model.to_string();
    }

    let resolved = resolve_role_reasoning_effort(
        role,
        provider,
        model,
        &empty,
        request_override,
        tenant_default,
        defaults.reasoning_effort.as_deref(),
        server_by_role,
    );
    if role == LlmRole::Extract {
        // Pipeline choke point: cloud `none` disables reasoning and 400s on
        // mandatory endpoints. Must run before omit-env (SPEC-131) clears the field.
        let seed = resolved
            .effective
            .as_deref()
            .or(resolved.desired.as_deref());
        return edgequake_pipeline::resolve_extraction_reasoning_effort(
            provider,
            model,
            seed,
        );
    }
    edgequake_llm::apply_omit_reasoning_effort(resolved.effective)
}
