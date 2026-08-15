//! Copy allowlisted Langfuse/GenAI baggage onto span attributes (SPEC-124).
//!
//! Langfuse recommends propagating session/user to **all** spans. Rust has no
//! official BaggageSpanProcessor; this is a minimal allowlisted equivalent.

use std::time::Duration;

use opentelemetry::baggage::BaggageExt;
use opentelemetry::trace::Span as _;
use opentelemetry::{Context, KeyValue};
use opentelemetry_sdk::error::OTelSdkResult;
use opentelemetry_sdk::trace::{Span, SpanProcessor};
use opentelemetry_sdk::Resource;

use crate::langfuse_attrs::is_allowlisted_baggage_key;

/// Copies allowlisted baggage keys from the parent context onto new spans.
#[derive(Debug, Default)]
pub struct LangfuseBaggageSpanProcessor;

impl LangfuseBaggageSpanProcessor {
    pub fn new() -> Self {
        Self
    }
}

impl SpanProcessor for LangfuseBaggageSpanProcessor {
    fn on_start(&self, span: &mut Span, cx: &Context) {
        let baggage = cx.baggage();
        for (key, (value, _)) in baggage.iter() {
            let key_str = key.as_str();
            if is_allowlisted_baggage_key(key_str) {
                span.set_attribute(KeyValue::new(key_str.to_string(), value.to_string()));
            }
        }
    }

    fn on_end(&self, _span: opentelemetry_sdk::trace::SpanData) {}

    fn force_flush(&self) -> OTelSdkResult {
        Ok(())
    }

    fn shutdown_with_timeout(&self, _timeout: Duration) -> OTelSdkResult {
        Ok(())
    }

    fn set_resource(&mut self, _resource: &Resource) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::langfuse_attrs::{LANGFUSE_SESSION_ID, USER_ID};
    use opentelemetry::trace::{Tracer, TracerProvider as _};
    use opentelemetry_sdk::trace::InMemorySpanExporterBuilder;
    use opentelemetry_sdk::trace::SdkTracerProvider;

    #[test]
    fn copies_allowlisted_baggage_only() {
        let exporter = InMemorySpanExporterBuilder::new().build();
        let provider = SdkTracerProvider::builder()
            .with_span_processor(LangfuseBaggageSpanProcessor::new())
            .with_simple_exporter(exporter.clone())
            .build();

        let tracer = provider.tracer("test");
        let cx = Context::current_with_baggage(vec![
            KeyValue::new(LANGFUSE_SESSION_ID, "sess-1"),
            KeyValue::new(USER_ID, "user-1"),
            KeyValue::new("authorization", "should-not-copy"),
        ]);
        let _guard = cx.attach();

        {
            let _span = tracer.start("child");
        }

        provider.force_flush().expect("flush");
        let spans = exporter.get_finished_spans().expect("spans");
        assert_eq!(spans.len(), 1);
        let attrs: std::collections::HashMap<String, String> = spans[0]
            .attributes
            .iter()
            .map(|kv| (kv.key.as_str().to_string(), kv.value.to_string()))
            .collect();
        assert_eq!(
            attrs.get(LANGFUSE_SESSION_ID).map(String::as_str),
            Some("sess-1")
        );
        assert_eq!(attrs.get(USER_ID).map(String::as_str), Some("user-1"));
        assert!(!attrs.contains_key("authorization"));
    }
}
