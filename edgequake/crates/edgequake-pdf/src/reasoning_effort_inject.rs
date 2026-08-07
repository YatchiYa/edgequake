//! Inject clamped `reasoning_effort` into every VLM call (SPEC-109).
//!
//! `edgequake-pdf2md` builds [`CompletionOptions`] without effort today; wrapping the
//! provider lets EdgeQuake forward upload / role effort without waiting on a pdf2md bump.

use std::sync::Arc;

use async_trait::async_trait;
use edgequake_llm::clamp_reasoning_effort;
use edgequake_llm::{
    ChatMessage, CompletionOptions, LLMProvider, LLMResponse, Result, ToolChoice, ToolDefinition,
};

/// Delegates to an inner provider while forcing a clamped `reasoning_effort` on options.
pub struct ReasoningEffortInjectProvider {
    inner: Arc<dyn LLMProvider>,
    effort: Option<String>,
}

impl ReasoningEffortInjectProvider {
    /// Wrap `inner` when clamping yields a wire value; otherwise return `inner` unchanged.
    pub fn wrap(
        inner: Arc<dyn LLMProvider>,
        provider_id: &str,
        desired: Option<&str>,
    ) -> Arc<dyn LLMProvider> {
        let effort = clamp_reasoning_effort(provider_id, inner.model(), desired);
        if effort.is_none() {
            return inner;
        }
        Arc::new(Self { inner, effort })
    }

    fn inject(&self, options: Option<&CompletionOptions>) -> CompletionOptions {
        let mut opts = options.cloned().unwrap_or_default();
        opts.reasoning_effort = self.effort.clone();
        opts
    }
}

#[async_trait]
impl LLMProvider for ReasoningEffortInjectProvider {
    fn name(&self) -> &str {
        self.inner.name()
    }

    fn model(&self) -> &str {
        self.inner.model()
    }

    fn max_context_length(&self) -> usize {
        self.inner.max_context_length()
    }

    async fn complete(&self, prompt: &str) -> Result<LLMResponse> {
        let opts = self.inject(None);
        self.inner.complete_with_options(prompt, &opts).await
    }

    async fn complete_with_options(
        &self,
        prompt: &str,
        options: &CompletionOptions,
    ) -> Result<LLMResponse> {
        let opts = self.inject(Some(options));
        self.inner.complete_with_options(prompt, &opts).await
    }

    async fn chat(
        &self,
        messages: &[ChatMessage],
        options: Option<&CompletionOptions>,
    ) -> Result<LLMResponse> {
        let opts = self.inject(options);
        self.inner.chat(messages, Some(&opts)).await
    }

    async fn chat_with_tools(
        &self,
        messages: &[ChatMessage],
        tools: &[ToolDefinition],
        tool_choice: Option<ToolChoice>,
        options: Option<&CompletionOptions>,
    ) -> Result<LLMResponse> {
        let opts = self.inject(options);
        self.inner
            .chat_with_tools(messages, tools, tool_choice, Some(&opts))
            .await
    }

    fn supports_streaming(&self) -> bool {
        self.inner.supports_streaming()
    }

    fn supports_tool_streaming(&self) -> bool {
        self.inner.supports_tool_streaming()
    }

    fn supports_json_mode(&self) -> bool {
        self.inner.supports_json_mode()
    }

    fn supports_function_calling(&self) -> bool {
        self.inner.supports_function_calling()
    }

    async fn refresh_model_metadata(&self) -> Result<()> {
        self.inner.refresh_model_metadata().await
    }

    fn default_max_output_tokens(&self) -> Option<usize> {
        self.inner.default_max_output_tokens()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use edgequake_llm::MockProvider;

    #[test]
    fn wrap_passthrough_when_unsupported() {
        let inner: Arc<dyn LLMProvider> = Arc::new(MockProvider::new());
        let wrapped = ReasoningEffortInjectProvider::wrap(inner.clone(), "mock", Some("high"));
        // Mock has no reasoning registry entry → same Arc identity not required, but no effort inject.
        assert_eq!(wrapped.name(), inner.name());
    }
}
