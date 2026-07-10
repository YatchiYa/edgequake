//! Debounced community index refresh scheduler (SPEC-024 Phase 4.2 / SRP).
//!
//! Coalesces burst ingests into one Louvain run per workspace window.
//! Called from [`crate::community_persist::schedule_community_index_refresh`].
//!
//! SPEC-046 EQ-046-11: optional community_report vector indexing when an
//! embedder + vector store are supplied (DIP — storage never imports LLM).

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::OnceLock;
use std::time::Duration;

use tokio::sync::Mutex;
use tokio::task::JoinHandle;

use crate::community::CommunityConfig;
use crate::community_persist::{community_features_enabled, detect_and_persist_communities};
use crate::community_reports::{community_reports_enabled, index_community_reports_with_embedder};
use crate::traits::{GraphStorage, TextEmbedder, VectorStorage};

/// Debounce window for post-ingest community refresh (default 300s).
pub fn community_refresh_debounce_secs() -> u64 {
    std::env::var("EDGEQUAKE_COMMUNITY_REFRESH_DEBOUNCE_SECS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(300)
}

fn debounce_duration() -> Duration {
    Duration::from_secs(community_refresh_debounce_secs())
}

/// Optional vector indexing hooks for community reports (SPEC-046).
#[derive(Clone, Default)]
pub struct CommunityRefreshExtras {
    pub vector_storage: Option<Arc<dyn VectorStorage>>,
    pub embedder: Option<Arc<dyn TextEmbedder>>,
    pub tenant_id: Option<String>,
}

impl CommunityRefreshExtras {
    pub fn with_report_indexing(
        vector_storage: Arc<dyn VectorStorage>,
        embedder: Arc<dyn TextEmbedder>,
    ) -> Self {
        Self {
            vector_storage: Some(vector_storage),
            embedder: Some(embedder),
            tenant_id: None,
        }
    }

    pub fn with_tenant_id(mut self, tenant_id: Option<String>) -> Self {
        self.tenant_id = tenant_id;
        self
    }
}

/// Schedule a debounced community refresh for a workspace (SPEC-024 1.3).
pub async fn schedule_community_index_refresh(
    graph: Arc<dyn GraphStorage>,
    workspace_id: Option<String>,
) {
    schedule_community_index_refresh_with_extras(
        graph,
        workspace_id,
        CommunityRefreshExtras::default(),
    )
    .await;
}

/// Schedule refresh and optionally index community_report vectors after Louvain.
pub async fn schedule_community_index_refresh_with_extras(
    graph: Arc<dyn GraphStorage>,
    workspace_id: Option<String>,
    extras: CommunityRefreshExtras,
) {
    if !community_features_enabled() {
        return;
    }
    community_scheduler()
        .schedule(workspace_id, graph, extras)
        .await;
}

struct CommunityRefreshScheduler {
    timers: Mutex<HashMap<String, JoinHandle<()>>>,
}

impl CommunityRefreshScheduler {
    async fn pending_workspace_count(&self) -> usize {
        self.timers.lock().await.len()
    }

    async fn schedule(
        &self,
        workspace_id: Option<String>,
        graph: Arc<dyn GraphStorage>,
        extras: CommunityRefreshExtras,
    ) {
        let key = workspace_id
            .clone()
            .unwrap_or_else(|| "default".to_string());
        let debounce = debounce_duration();

        let mut timers = self.timers.lock().await;
        if let Some(handle) = timers.remove(&key) {
            handle.abort();
        }

        let handle = tokio::spawn(async move {
            tokio::time::sleep(debounce).await;
            run_community_refresh(graph, workspace_id, extras).await;
        });

        timers.insert(key, handle);
    }
}

fn community_scheduler() -> &'static CommunityRefreshScheduler {
    static SCHEDULER: OnceLock<CommunityRefreshScheduler> = OnceLock::new();
    SCHEDULER.get_or_init(|| CommunityRefreshScheduler {
        timers: Mutex::new(HashMap::new()),
    })
}

async fn run_community_refresh(
    graph: Arc<dyn GraphStorage>,
    workspace_id: Option<String>,
    extras: CommunityRefreshExtras,
) {
    if !community_features_enabled() {
        return;
    }
    match detect_and_persist_communities(graph, &CommunityConfig::default()).await {
        Ok(result) => {
            tracing::debug!(
                communities = result.communities.len(),
                labeled_nodes = result.node_to_community.len(),
                "Community index refreshed after ingest"
            );
            maybe_index_community_reports(&result, workspace_id.as_deref(), &extras).await;
        }
        Err(e) => tracing::warn!(
            error = %e,
            "Community index refresh failed (non-fatal)"
        ),
    }
}

async fn maybe_index_community_reports(
    result: &crate::community::CommunityDetectionResult,
    workspace_id: Option<&str>,
    extras: &CommunityRefreshExtras,
) {
    if !community_reports_enabled() {
        return;
    }
    let (Some(vs), Some(embedder)) = (extras.vector_storage.as_ref(), extras.embedder.as_ref())
    else {
        tracing::debug!(
            "SPEC-046: community reports enabled but embedder/vector missing — props only"
        );
        return;
    };
    match index_community_reports_with_embedder(
        result,
        vs.as_ref(),
        embedder.as_ref(),
        workspace_id,
        extras.tenant_id.as_deref(),
    )
    .await
    {
        Ok(n) => tracing::debug!(
            reports_indexed = n,
            "SPEC-046: community_report vectors upserted"
        ),
        Err(e) => tracing::warn!(
            error = %e,
            "SPEC-046: community_report vector index failed (non-fatal)"
        ),
    }
}

/// Number of workspaces with a debounced community refresh timer pending (scale signal).
pub async fn pending_community_refresh_workspaces() -> usize {
    if !community_features_enabled() {
        return 0;
    }
    community_scheduler().pending_workspace_count().await
}

/// Run community refresh immediately (tests / one-shot hooks).
pub async fn refresh_community_index_now(graph: Arc<dyn GraphStorage>) {
    run_community_refresh(graph, None, CommunityRefreshExtras::default()).await;
}

/// Immediate refresh + optional report vector indexing (tests / ops).
pub async fn refresh_community_index_now_with_extras(
    graph: Arc<dyn GraphStorage>,
    workspace_id: Option<String>,
    extras: CommunityRefreshExtras,
) {
    run_community_refresh(graph, workspace_id, extras).await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debounce_default_is_five_minutes() {
        std::env::remove_var("EDGEQUAKE_COMMUNITY_REFRESH_DEBOUNCE_SECS");
        assert_eq!(community_refresh_debounce_secs(), 300);
    }
}
