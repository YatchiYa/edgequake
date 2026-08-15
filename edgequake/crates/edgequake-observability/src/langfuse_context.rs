//! Bind Langfuse session/user identity onto the active OTEL span (SPEC-124).
//!
//! Sets attributes on the current span only (Send-safe for Axum). Child spans
//! share the same OTEL trace; Langfuse promotes `sessionId` / `userId` from
//! span attributes to the trace. Optional baggage is still supported when a
//! caller attaches context without holding a guard across `.await`.

use crate::langfuse_attrs::LangfuseTraceIdentity;

/// Marker returned for call-site clarity (`let _ = bind…`).
///
/// Historically held an OTEL `ContextGuard`; that type is `!Send` and cannot
/// live across Axum `.await` points. Session attrs are applied synchronously
/// to the current span instead.
#[must_use = "call bind_langfuse_identity for its side effects on the current span"]
#[derive(Debug, Default)]
pub struct LangfuseIdentityGuard;

impl LangfuseIdentityGuard {
    pub fn noop() -> Self {
        Self
    }
}

/// Bind session/user/tenant/workspace onto the current OTEL span.
///
/// Empty `session_id` → no session attrs (never synthesizes).
pub fn bind_langfuse_identity(
    session_id: Option<&str>,
    user_id: Option<&str>,
    tenant_id: Option<&str>,
    workspace_id: Option<&str>,
) -> LangfuseIdentityGuard {
    bind_langfuse_trace_identity(LangfuseTraceIdentity::from_parts(
        session_id,
        user_id,
        tenant_id,
        workspace_id,
    ))
}

/// Bind a full identity (GUIDs + optional slugs) onto the current span.
pub fn bind_langfuse_trace_identity(identity: LangfuseTraceIdentity) -> LangfuseIdentityGuard {
    if identity.is_empty() {
        return LangfuseIdentityGuard::noop();
    }

    #[cfg(feature = "otel")]
    {
        use opentelemetry::trace::TraceContextExt;
        use opentelemetry::{Context, KeyValue};

        let current = Context::current();
        if current.has_active_span() {
            let span = current.span();
            for (k, v) in identity.key_values() {
                span.set_attribute(KeyValue::new(k, v));
            }
        }
    }

    #[cfg(not(feature = "otel"))]
    {
        let _ = identity;
    }

    LangfuseIdentityGuard
}

/// Attach allowlisted identity as OTEL baggage for the duration of `f`.
///
/// Prefer this only for synchronous sections or when wrapping a whole
/// `tokio::spawn` future via [`with_langfuse_identity_async`].
#[cfg(feature = "otel")]
pub fn with_langfuse_identity_sync<R>(
    session_id: Option<&str>,
    user_id: Option<&str>,
    tenant_id: Option<&str>,
    workspace_id: Option<&str>,
    f: impl FnOnce() -> R,
) -> R {
    use opentelemetry::baggage::BaggageExt;
    use opentelemetry::{Context, KeyValue};

    let identity = LangfuseTraceIdentity::from_parts(session_id, user_id, tenant_id, workspace_id);
    if identity.is_empty() {
        return f();
    }
    let kvs: Vec<KeyValue> = identity
        .key_values()
        .into_iter()
        .map(|(k, v)| KeyValue::new(k, v))
        .collect();
    let _guard = Context::current().with_baggage(kvs).attach();
    f()
}

/// Run an async block with Langfuse identity baggage attached (Send-safe).
///
/// The baggage `ContextGuard` lives only inside this helper's future poll,
/// so the caller's future stays `Send`. Child spans started while the future
/// runs receive allowlisted attrs via [`crate::baggage_span_processor`].
#[cfg(feature = "otel")]
pub async fn with_langfuse_identity_async<F, T>(
    session_id: Option<&str>,
    user_id: Option<&str>,
    tenant_id: Option<&str>,
    workspace_id: Option<&str>,
    fut: F,
) -> T
where
    F: std::future::Future<Output = T>,
{
    bind_langfuse_trace_identity_async(
        LangfuseTraceIdentity::from_parts(session_id, user_id, tenant_id, workspace_id),
        fut,
    )
    .await
}

/// Run an async block with full identity (GUIDs + slugs) as baggage.
#[cfg(feature = "otel")]
pub async fn bind_langfuse_trace_identity_async<F, T>(identity: LangfuseTraceIdentity, fut: F) -> T
where
    F: std::future::Future<Output = T>,
{
    use opentelemetry::baggage::BaggageExt;
    use opentelemetry::trace::TraceContextExt;
    use opentelemetry::{Context, KeyValue};
    use std::pin::pin;
    use std::task::Context as TaskCx;

    if identity.is_empty() {
        return fut.await;
    }

    let kvs: Vec<KeyValue> = identity
        .key_values()
        .into_iter()
        .map(|(k, v)| KeyValue::new(k, v))
        .collect();

    let current = Context::current();
    if current.has_active_span() {
        let span = current.span();
        for kv in &kvs {
            span.set_attribute(kv.clone());
        }
    }

    let cx = current.with_baggage(kvs);
    let mut fut = pin!(fut);
    std::future::poll_fn(|task_cx: &mut TaskCx<'_>| {
        let _guard = cx.clone().attach();
        fut.as_mut().poll(task_cx)
    })
    .await
}

#[cfg(not(feature = "otel"))]
pub async fn bind_langfuse_trace_identity_async<F, T>(_identity: LangfuseTraceIdentity, fut: F) -> T
where
    F: std::future::Future<Output = T>,
{
    fut.await
}

#[cfg(not(feature = "otel"))]
pub async fn with_langfuse_identity_async<F, T>(
    _session_id: Option<&str>,
    _user_id: Option<&str>,
    _tenant_id: Option<&str>,
    _workspace_id: Option<&str>,
    fut: F,
) -> T
where
    F: std::future::Future<Output = T>,
{
    fut.await
}

#[cfg(all(test, feature = "otel"))]
mod tests {
    use super::*;
    use crate::langfuse_attrs::LANGFUSE_SESSION_ID;
    use opentelemetry::baggage::BaggageExt;
    use opentelemetry::Context;

    #[test]
    fn bind_empty_is_ok() {
        let _g = bind_langfuse_identity(None, None, None, None);
    }

    #[test]
    fn sync_scope_sets_baggage() {
        with_langfuse_identity_sync(Some("conv-abc"), Some("user-x"), None, None, || {
            let cx = Context::current();
            assert_eq!(
                cx.baggage().get(LANGFUSE_SESSION_ID).map(|v| v.to_string()),
                Some("conv-abc".into())
            );
        });
        let cx = Context::current();
        assert!(cx.baggage().get(LANGFUSE_SESSION_ID).is_none());
    }

    #[tokio::test]
    async fn async_scope_sets_baggage_during_poll() {
        with_langfuse_identity_async(Some("conv-async"), None, None, None, async {
            let cx = Context::current();
            assert_eq!(
                cx.baggage().get(LANGFUSE_SESSION_ID).map(|v| v.to_string()),
                Some("conv-async".into())
            );
        })
        .await;
    }
}
