//! Workspace ontology routes (SPEC-184).
//!
//! The document lives in workspace metadata (`ontology`). Saving it also writes the
//! SPEC-114 lists so every existing extraction mechanism sees the same vocabulary;
//! the lists it replaced are kept in `ontology_schema_before` and restored on delete.
//! Realignment reuses the entity create/merge/update handlers, so it inherits their
//! isolation, audit and side effects.

use std::collections::HashMap;

use axum::{
    extract::{Path, State},
    Json,
};
use edgequake_core::UpdateWorkspaceRequest;
use edgequake_pipeline::ontology::{
    import, plan::alignment_plan, plan::AlignmentPlan, plan::NodeFact, schema_projection, validate,
    CompiledOntology, OntologySource, ValidationMode, WorkspaceOntology, METADATA_KEY,
    SCHEMA_BEFORE_KEY,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use utoipa::ToSchema;
use uuid::Uuid;

use super::helpers::verify_workspace_tenant_access;
use crate::error::ApiError;
use crate::middleware::TenantContext;
use crate::state::AppState;

/// Graph ids looked up per storage call while planning.
const LOOKUP_BATCH: usize = 500;
/// Realign operations run by one request (the caller repeats until `remaining` is 0).
const MAX_REALIGN_OPERATIONS: usize = 1_000;
const DEFAULT_REALIGN_OPERATIONS: usize = 200;

// ---------------------------------------------------------------------------
// DTOs
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default, Serialize, ToSchema)]
pub struct OntologyCounts {
    pub entity_types: usize,
    pub relation_types: usize,
    pub terms: usize,
    pub aliases: usize,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct OntologyResponse {
    /// The saved ontology, or `null` when the workspace has none.
    #[schema(value_type = Object)]
    pub ontology: Option<Value>,
    pub counts: OntologyCounts,
    /// What saving normalized or dropped.
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct OntologyDraftRequest {
    /// Text of the source: JSON, CSV/TSV, Markdown or plain text (e.g. a converted PDF).
    pub text: String,
    /// File name (its extension picks the parser) or document title.
    #[serde(default)]
    pub file_name: Option<String>,
    /// Workspace document the text came from (recorded as the source).
    #[serde(default)]
    pub document_id: Option<String>,
    /// `file`, `document`, `text` or `copilot`.
    #[serde(default)]
    pub source_kind: Option<String>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct OntologyDraftResponse {
    #[schema(value_type = Object)]
    pub ontology: Value,
    pub counts: OntologyCounts,
    /// `json`, `table` or `llm`.
    pub format: String,
    pub warnings: Vec<String>,
    /// LLM segments read (0 for JSON and tables).
    pub segments: usize,
    /// The text was longer than the draft reads.
    pub truncated: bool,
}

#[derive(Debug, Clone, Default, Deserialize, ToSchema)]
pub struct OntologyPlanRequest {
    /// Ontology to compare; the saved one when omitted.
    #[serde(default)]
    #[schema(value_type = Object)]
    pub ontology: Option<Value>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct TypeCoverage {
    pub entity_type: String,
    pub count: usize,
    pub in_ontology: bool,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct OntologyPlanResponse {
    #[schema(value_type = Object)]
    pub plan: AlignmentPlan,
    /// Entity counts per graph type, marked when the ontology defines the type.
    pub types: Vec<TypeCoverage>,
    pub entities_total: usize,
    /// Entities whose type the ontology defines (0 when it defines none).
    pub entities_in_ontology_types: usize,
    pub documents: usize,
}

#[derive(Debug, Clone, Default, Deserialize, ToSchema)]
pub struct OntologyRealignRequest {
    /// Operations to run in this call (default 200, max 1000).
    #[serde(default)]
    pub max_operations: Option<usize>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct RealignStep {
    /// `create`, `merge` or `retype`.
    pub op: String,
    pub entity: String,
    pub ok: bool,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct OntologyRealignResponse {
    pub steps: Vec<RealignStep>,
    pub merged: usize,
    pub retyped: usize,
    pub failed: usize,
    /// Planned operations not run yet (call again).
    pub remaining: usize,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn counts(o: &WorkspaceOntology) -> OntologyCounts {
    OntologyCounts {
        entity_types: o.entity_types.len(),
        relation_types: o.relation_types.len(),
        terms: o.terms.len(),
        aliases: o.terms.iter().map(|t| t.aliases.len()).sum(),
    }
}

/// The ontology routes act on the workspace the caller selected (its membership was
/// checked by the middleware): the path must name that workspace.
async fn scoped_workspace(
    state: &AppState,
    workspace_id: Uuid,
    ctx: &TenantContext,
) -> Result<(edgequake_core::Workspace, TenantContext), ApiError> {
    let ws = verify_workspace_tenant_access(state, workspace_id, ctx).await?;
    if let Some(selected) = ctx.workspace_id.as_deref() {
        if selected != workspace_id.to_string() {
            return Err(ApiError::BadRequest(
                "Select this workspace (X-Workspace-ID) before changing its ontology.".into(),
            ));
        }
    }
    let scoped = TenantContext {
        tenant_id: Some(ws.tenant_id.to_string()),
        workspace_id: Some(workspace_id.to_string()),
        user_id: ctx.user_id.clone(),
    };
    Ok((ws, scoped))
}

fn stored_ontology(metadata: &HashMap<String, Value>) -> Option<WorkspaceOntology> {
    metadata
        .get(METADATA_KEY)
        .filter(|v| !v.is_null())
        .and_then(|v| serde_json::from_value(v.clone()).ok())
}

fn parse_ontology(value: Value) -> Result<WorkspaceOntology, ApiError> {
    serde_json::from_value(value).map_err(|e| ApiError::BadRequest(format!("Not an ontology: {e}")))
}

fn validation_error(errors: Vec<String>) -> ApiError {
    ApiError::BadRequest(format!("The ontology is not valid: {}", errors.join(" | ")))
}

async fn save_metadata(
    state: &AppState,
    workspace_id: Uuid,
    request: UpdateWorkspaceRequest,
) -> Result<edgequake_core::Workspace, ApiError> {
    state
        .workspace_service
        .update_workspace(workspace_id, request)
        .await
        .map_err(|e| match &e {
            edgequake_core::Error::Validation(msg) => ApiError::BadRequest(msg.clone()),
            edgequake_core::Error::NotFound(msg) => ApiError::NotFound(msg.clone()),
            _ => ApiError::Internal(e.to_string()),
        })
}

// ---------------------------------------------------------------------------
// GET / PUT / DELETE
// ---------------------------------------------------------------------------

/// Get the workspace ontology.
#[utoipa::path(
    get,
    path = "/api/v1/workspaces/{workspace_id}/ontology",
    params(("workspace_id" = Uuid, Path, description = "Workspace ID")),
    responses((status = 200, description = "Ontology or null", body = OntologyResponse)),
    tags = ["workspaces"]
)]
pub async fn get_workspace_ontology(
    State(state): State<AppState>,
    Path(workspace_id): Path<Uuid>,
    tenant_ctx: TenantContext,
) -> Result<Json<OntologyResponse>, ApiError> {
    let ws = verify_workspace_tenant_access(&state, workspace_id, &tenant_ctx).await?;
    let ontology = stored_ontology(&ws.metadata);
    Ok(Json(OntologyResponse {
        counts: ontology.as_ref().map(counts).unwrap_or_default(),
        ontology: ontology.map(|o| serde_json::to_value(o).unwrap_or(Value::Null)),
        warnings: Vec::new(),
    }))
}

/// Save the workspace ontology (future extractions only — LAW-184-5).
#[utoipa::path(
    put,
    path = "/api/v1/workspaces/{workspace_id}/ontology",
    params(("workspace_id" = Uuid, Path, description = "Workspace ID")),
    request_body(content = Object, description = "WorkspaceOntology JSON"),
    responses(
        (status = 200, description = "Saved", body = OntologyResponse),
        (status = 400, description = "Invalid ontology (every error listed)"),
    ),
    tags = ["workspaces"]
)]
pub async fn put_workspace_ontology(
    State(state): State<AppState>,
    Path(workspace_id): Path<Uuid>,
    tenant_ctx: TenantContext,
    Json(body): Json<Value>,
) -> Result<Json<OntologyResponse>, ApiError> {
    let (ws, _) = scoped_workspace(&state, workspace_id, &tenant_ctx).await?;
    let mut input = parse_ontology(body)?;
    input.updated_at = Some(chrono::Utc::now().to_rfc3339());
    let validated = validate(input, ValidationMode::Strict).map_err(validation_error)?;
    let ontology = validated.ontology;
    if ontology.entity_types.is_empty()
        && ontology.relation_types.is_empty()
        && ontology.terms.is_empty()
    {
        return Err(ApiError::BadRequest(
            "The ontology is empty: give at least one entity type, relation type or term.".into(),
        ));
    }
    let mut warnings = validated.warnings;
    let projection = schema_projection(&ontology);
    if projection.dropped_edges > 0 {
        warnings.push(format!(
            "{} type pairs beyond the {} typed-edge limit are not enforced.",
            projection.dropped_edges,
            edgequake_pipeline::ontology::MAX_RELATION_EDGES
        ));
    }

    // First ontology on this workspace: remember the lists it replaces.
    let snapshot = (!ws.metadata.contains_key(SCHEMA_BEFORE_KEY)).then(|| {
        let key = |k: &str| ws.metadata.get(k).cloned().unwrap_or(Value::Null);
        json!({
            "entity_types": key("entity_types"),
            "relation_types": key("relation_types"),
            "relation_edges": key("relation_edges"),
            "kg_schema_preset": key("kg_schema_preset"),
        })
    });

    let request = UpdateWorkspaceRequest {
        entity_types: projection.entity_types.clone(),
        relation_types: projection.relation_types.clone(),
        relation_edges: projection.relation_types.as_ref().map(|_| {
            projection
                .relation_edges
                .clone()
                .unwrap_or_default()
                .into_iter()
                .map(
                    |(source, relation, target)| edgequake_core::type_list::RelationEdge {
                        source,
                        relation,
                        target,
                    },
                )
                .collect()
        }),
        // A domain preset no longer describes the vocabulary.
        kg_schema_preset: (projection.entity_types.is_some()
            || projection.relation_types.is_some())
        .then(String::new),
        ontology: Some(
            serde_json::to_value(&ontology).map_err(|e| ApiError::Internal(e.to_string()))?,
        ),
        ontology_schema_before: snapshot,
        ..Default::default()
    };
    save_metadata(&state, workspace_id, request).await?;
    tracing::info!(
        workspace_id = %workspace_id,
        entity_types = ontology.entity_types.len(),
        relation_types = ontology.relation_types.len(),
        terms = ontology.terms.len(),
        "Workspace ontology saved"
    );
    crate::services::record_compliance_event(
        &state,
        ws.tenant_id.to_string(),
        edgequake_audit::AuditEventType::WorkspaceAccess,
        "workspace_ontology_saved",
        edgequake_audit::AuditResult::Success,
        Some(workspace_id.to_string()),
        tenant_ctx.user_id.clone(),
        None,
    );
    Ok(Json(OntologyResponse {
        counts: counts(&ontology),
        ontology: Some(serde_json::to_value(&ontology).unwrap_or(Value::Null)),
        warnings,
    }))
}

/// Remove the ontology and restore the lists it replaced.
#[utoipa::path(
    delete,
    path = "/api/v1/workspaces/{workspace_id}/ontology",
    params(("workspace_id" = Uuid, Path, description = "Workspace ID")),
    responses((status = 200, description = "Removed", body = OntologyResponse)),
    tags = ["workspaces"]
)]
pub async fn delete_workspace_ontology(
    State(state): State<AppState>,
    Path(workspace_id): Path<Uuid>,
    tenant_ctx: TenantContext,
) -> Result<Json<OntologyResponse>, ApiError> {
    let (ws, _) = scoped_workspace(&state, workspace_id, &tenant_ctx).await?;
    if !ws.metadata.contains_key(METADATA_KEY) && !ws.metadata.contains_key(SCHEMA_BEFORE_KEY) {
        return Ok(Json(OntologyResponse {
            ontology: None,
            counts: OntologyCounts::default(),
            warnings: Vec::new(),
        }));
    }
    let before = ws
        .metadata
        .get(SCHEMA_BEFORE_KEY)
        .cloned()
        .unwrap_or(Value::Null);
    let list = |k: &str| -> Option<Vec<String>> {
        // `null` in the snapshot = the key was absent = server defaults / free-form.
        Some(
            before
                .get(k)
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or_default(),
        )
    };
    let has_snapshot = !before.is_null();
    let request = UpdateWorkspaceRequest {
        entity_types: has_snapshot.then(|| list("entity_types")).flatten(),
        relation_types: has_snapshot.then(|| list("relation_types")).flatten(),
        relation_edges: has_snapshot.then(|| {
            before
                .get("relation_edges")
                .and_then(|v| {
                    serde_json::from_value::<Vec<edgequake_core::type_list::RelationEdge>>(
                        v.clone(),
                    )
                    .ok()
                })
                .unwrap_or_default()
        }),
        kg_schema_preset: has_snapshot.then(|| {
            before
                .get("kg_schema_preset")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string()
        }),
        ontology: Some(Value::Null),
        ontology_schema_before: Some(Value::Null),
        ..Default::default()
    };
    save_metadata(&state, workspace_id, request).await?;
    tracing::info!(workspace_id = %workspace_id, "Workspace ontology removed; previous lists restored");
    crate::services::record_compliance_event(
        &state,
        ws.tenant_id.to_string(),
        edgequake_audit::AuditEventType::WorkspaceAccess,
        "workspace_ontology_removed",
        edgequake_audit::AuditResult::Success,
        Some(workspace_id.to_string()),
        tenant_ctx.user_id.clone(),
        None,
    );
    Ok(Json(OntologyResponse {
        ontology: None,
        counts: OntologyCounts::default(),
        warnings: Vec::new(),
    }))
}

// ---------------------------------------------------------------------------
// Draft
// ---------------------------------------------------------------------------

/// Propose an ontology from a source text (nothing is saved).
#[utoipa::path(
    post,
    path = "/api/v1/workspaces/{workspace_id}/ontology/draft",
    params(("workspace_id" = Uuid, Path, description = "Workspace ID")),
    request_body = OntologyDraftRequest,
    responses((status = 200, description = "Proposed ontology", body = OntologyDraftResponse)),
    tags = ["workspaces"]
)]
pub async fn draft_workspace_ontology(
    State(state): State<AppState>,
    Path(workspace_id): Path<Uuid>,
    tenant_ctx: TenantContext,
    Json(req): Json<OntologyDraftRequest>,
) -> Result<Json<OntologyDraftResponse>, ApiError> {
    let (ws, _) = scoped_workspace(&state, workspace_id, &tenant_ctx).await?;
    if req.text.trim().is_empty() {
        return Err(ApiError::BadRequest("The source text is empty.".into()));
    }
    let mut warnings = Vec::new();
    let (mut draft, format, segments_read, truncated) =
        match import::parse_structured(&req.text, req.file_name.as_deref())
            .map_err(ApiError::BadRequest)?
        {
            Some((o, import::DraftFormat::Json)) => (o, "json", 0, false),
            Some((o, _)) => (o, "table", 0, false),
            None => {
                let (draft, read, truncated) =
                    draft_with_llm(&state, &ws, &req.text, &mut warnings).await?;
                (draft, "llm", read, truncated)
            }
        };
    if truncated {
        warnings.push(format!(
            "Only the first {} characters were read.",
            import::MAX_DRAFT_TEXT_CHARS
        ));
    }
    if draft.name.trim().is_empty() {
        draft.name = req
            .file_name
            .as_deref()
            .map(|n| {
                n.rsplit_once('.')
                    .map(|(stem, _)| stem)
                    .unwrap_or(n)
                    .to_string()
            })
            .unwrap_or_else(|| ws.name.clone());
    }
    draft.sources.push(OntologySource {
        kind: req.source_kind.clone().unwrap_or_else(|| {
            if req.document_id.is_some() {
                "document".into()
            } else {
                "file".into()
            }
        }),
        name: req.file_name.clone().unwrap_or_default(),
        document_id: req.document_id.clone(),
        at: Some(chrono::Utc::now().to_rfc3339()),
    });
    let validated = validate(draft, ValidationMode::Repair).map_err(validation_error)?;
    warnings.extend(validated.warnings);
    Ok(Json(OntologyDraftResponse {
        counts: counts(&validated.ontology),
        ontology: serde_json::to_value(&validated.ontology)
            .map_err(|e| ApiError::Internal(e.to_string()))?,
        format: format.into(),
        warnings,
        segments: segments_read,
        truncated,
    }))
}

async fn draft_with_llm(
    state: &AppState,
    ws: &edgequake_core::Workspace,
    text: &str,
    warnings: &mut Vec<String>,
) -> Result<(WorkspaceOntology, usize, bool), ApiError> {
    use edgequake_llm::traits::{ChatMessage, CompletionOptions};

    let resolver = crate::providers::WorkspaceProviderResolver::from_app_state(state);
    let request = crate::providers::LlmResolutionRequest {
        provider: None,
        model: None,
        extra_headers: None,
    };
    let llm = match resolver
        .resolve_llm_provider_for_workspace(Some(ws), &request)
        .await
    {
        Ok(Some(resolved)) => resolved.provider,
        Ok(None) => state.query.llm_provider.clone(),
        Err(e) => return Err(ApiError::from(e)),
    };
    let (segments, truncated) = import::segments(text);
    let total = segments.len();
    let mut draft = WorkspaceOntology::default();
    let mut read = 0;
    for (i, segment) in segments.iter().enumerate() {
        let messages = vec![
            ChatMessage::system(import::DRAFT_SYSTEM_PROMPT),
            ChatMessage::user(import::draft_user_prompt(
                segment,
                i,
                total,
                &import::known_type_ids(&draft),
            )),
        ];
        let options = CompletionOptions {
            max_tokens: Some(8_000),
            temperature: edgequake_llm::resolve_effective_temperature(llm.model(), 0.1),
            ..Default::default()
        };
        match llm.chat(&messages, Some(&options)).await {
            Ok(response) => match import::parse_draft_response(&response.content) {
                Ok(part) => {
                    import::merge_draft(&mut draft, part);
                    read += 1;
                }
                Err(e) => warnings.push(format!("Segment {} of {total}: {e}", i + 1)),
            },
            Err(e) => {
                if read == 0 && i == 0 {
                    return Err(ApiError::Internal(format!(
                        "The language model could not read the source: {e}"
                    )));
                }
                warnings.push(format!("Segment {} of {total} was not read: {e}", i + 1));
            }
        }
    }
    if read == 0 {
        return Err(ApiError::BadRequest(
            "The language model returned no usable ontology for this text.".into(),
        ));
    }
    Ok((draft, read, truncated))
}

// ---------------------------------------------------------------------------
// Plan / realign
// ---------------------------------------------------------------------------

/// Nodes of the workspace whose bare name is one of `keys`, with type, degree, description.
async fn lookup_nodes(
    state: &AppState,
    ctx: &TenantContext,
    keys: &[String],
) -> Result<HashMap<String, (NodeFact, String)>, ApiError> {
    let ws = ctx.workspace_id.as_deref();
    let graph = state.storage.graph_storage.as_ref();
    let mut found = HashMap::new();
    for batch in keys.chunks(LOOKUP_BATCH) {
        let mut ids: Vec<String> = Vec::with_capacity(batch.len() * 2);
        for key in batch {
            let entity = edgequake_storage::EntityId::new(key);
            let scoped = entity.graph_node_id_for_workspace(ws);
            if scoped != *key {
                ids.push(scoped);
            }
            ids.push(key.clone());
        }
        let nodes = crate::handlers::isolation::filter_nodes_by_tenant_context(
            graph.get_nodes_by_ids(&ids).await?,
            ctx,
        );
        if nodes.is_empty() {
            continue;
        }
        let node_ids: Vec<String> = nodes.iter().map(|n| n.id.clone()).collect();
        let degrees: HashMap<String, usize> = graph
            .node_degrees_batch(&node_ids)
            .await?
            .into_iter()
            .collect();
        for node in nodes {
            let bare =
                edgequake_storage::EntityId::bare_name_from_graph_node_id(&node.id).to_string();
            let prop = |k: &str| {
                node.properties
                    .get(k)
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string()
            };
            let fact = NodeFact {
                entity_type: prop("entity_type"),
                degree: degrees.get(&node.id).copied().unwrap_or(0),
            };
            found.insert(bare, (fact, prop("description")));
        }
    }
    Ok(found)
}

async fn plan_for(
    state: &AppState,
    ctx: &TenantContext,
    compiled: &CompiledOntology,
) -> Result<(AlignmentPlan, HashMap<String, (NodeFact, String)>), ApiError> {
    let keys: Vec<String> = compiled.graph_keys().map(|(k, _)| k.to_string()).collect();
    let found = lookup_nodes(state, ctx, &keys).await?;
    let facts: HashMap<String, NodeFact> = found
        .iter()
        .map(|(k, (f, _))| (k.clone(), f.clone()))
        .collect();
    Ok((alignment_plan(compiled, &facts), found))
}

/// What realigning the current graph to an ontology would change.
#[utoipa::path(
    post,
    path = "/api/v1/workspaces/{workspace_id}/ontology/plan",
    params(("workspace_id" = Uuid, Path, description = "Workspace ID")),
    request_body = OntologyPlanRequest,
    responses((status = 200, description = "Alignment plan", body = OntologyPlanResponse)),
    tags = ["workspaces"]
)]
pub async fn plan_workspace_ontology(
    State(state): State<AppState>,
    Path(workspace_id): Path<Uuid>,
    tenant_ctx: TenantContext,
    body: Option<Json<OntologyPlanRequest>>,
) -> Result<Json<OntologyPlanResponse>, ApiError> {
    let (ws, scoped) = scoped_workspace(&state, workspace_id, &tenant_ctx).await?;
    let ontology = match body.and_then(|Json(b)| b.ontology) {
        Some(value) => {
            validate(parse_ontology(value)?, ValidationMode::Repair)
                .map_err(validation_error)?
                .ontology
        }
        None => stored_ontology(&ws.metadata)
            .ok_or_else(|| ApiError::BadRequest("This workspace has no ontology yet.".into()))?,
    };
    let compiled = CompiledOntology::compile(&ontology);
    let (plan, _) = plan_for(&state, &scoped, &compiled).await?;

    let facets = crate::handlers::get_graph_facets(State(state.storage.clone()), scoped.clone())
        .await?
        .0;
    let defined: std::collections::HashSet<&str> = ontology
        .entity_types
        .iter()
        .map(|t| t.id.as_str())
        .collect();
    let types: Vec<TypeCoverage> = facets
        .entity_types
        .iter()
        .map(|f| TypeCoverage {
            in_ontology: compiled.canonical_type(&f.name).is_some()
                || defined.contains(f.name.as_str()),
            entity_type: f.name.clone(),
            count: f.count,
        })
        .collect();
    let entities_total = types.iter().map(|t| t.count).sum();
    let entities_in_ontology_types = types
        .iter()
        .filter(|t| t.in_ontology)
        .map(|t| t.count)
        .sum();
    let documents = state
        .workspace_service
        .get_workspace_stats(workspace_id)
        .await
        .map(|s| s.document_count)
        .unwrap_or(0);
    Ok(Json(OntologyPlanResponse {
        plan,
        types,
        entities_total,
        entities_in_ontology_types,
        documents,
    }))
}

/// Merge alias entities into canonical ones and retype terms, as planned for the saved ontology.
#[utoipa::path(
    post,
    path = "/api/v1/workspaces/{workspace_id}/ontology/realign",
    params(("workspace_id" = Uuid, Path, description = "Workspace ID")),
    request_body = OntologyRealignRequest,
    responses((status = 200, description = "Steps run", body = OntologyRealignResponse)),
    tags = ["workspaces"]
)]
pub async fn realign_workspace_ontology(
    State(state): State<AppState>,
    Path(workspace_id): Path<Uuid>,
    tenant_ctx: TenantContext,
    body: Option<Json<OntologyRealignRequest>>,
) -> Result<Json<OntologyRealignResponse>, ApiError> {
    use crate::handlers::{CreateEntityRequest, MergeEntitiesRequest, UpdateEntityRequest};

    let (ws, scoped) = scoped_workspace(&state, workspace_id, &tenant_ctx).await?;
    let ontology = stored_ontology(&ws.metadata).ok_or_else(|| {
        ApiError::BadRequest("Save an ontology before realigning the graph.".into())
    })?;
    let budget = body
        .and_then(|Json(b)| b.max_operations)
        .unwrap_or(DEFAULT_REALIGN_OPERATIONS)
        .clamp(1, MAX_REALIGN_OPERATIONS);
    let compiled = CompiledOntology::compile(&ontology);
    let (plan, found) = plan_for(&state, &scoped, &compiled).await?;
    let planned = plan.merges.len() + plan.retypes.len();

    let mut steps = Vec::new();
    let (mut merged, mut retyped, mut failed, mut used) = (0, 0, 0, 0);
    let mut created: std::collections::HashSet<String> = std::collections::HashSet::new();
    let record = |steps: &mut Vec<RealignStep>,
                  op: &str,
                  entity: &str,
                  result: Result<String, ApiError>|
     -> bool {
        let ok = result.is_ok();
        steps.push(RealignStep {
            op: op.into(),
            entity: entity.into(),
            ok,
            detail: result.unwrap_or_else(|e| e.to_string()),
        });
        ok
    };

    for m in &plan.merges {
        if used >= budget {
            break;
        }
        used += 1;
        let mut target_new = false;
        if !m.target_exists && created.insert(m.target.clone()) {
            target_new = true;
            let description = found
                .get(&m.source)
                .map(|(_, d)| d.clone())
                .unwrap_or_default();
            let request = CreateEntityRequest {
                entity_name: m.target.clone(),
                entity_type: if m.target_type.is_empty() {
                    "OTHER".into()
                } else {
                    m.target_type.clone()
                },
                description,
                source_id: "manual_entry".into(),
                metadata: json!({ "created_by": "ontology_realign" }),
            };
            let created_ok =
                crate::handlers::create_entity(State(state.clone()), scoped.clone(), Json(request))
                    .await
                    .map(|_| format!("canonical entity for {}", m.source));
            if !record(&mut steps, "create", &m.target, created_ok) {
                failed += 1;
                continue;
            }
        }
        let request = MergeEntitiesRequest {
            source_entity: m.source.clone(),
            target_entity: m.target.clone(),
            // A target just created from this alias already carries its description.
            merge_strategy: if target_new {
                "prefer_target".into()
            } else {
                "merge".into()
            },
            metadata: json!({ "merged_by": "ontology_realign" }),
        };
        let result =
            crate::handlers::merge_entities(State(state.clone()), scoped.clone(), Json(request))
                .await
                .map(|r| {
                    format!(
                        "{} relationships moved to {}",
                        r.0.merge_details.relationships_merged, m.target
                    )
                });
        if record(&mut steps, "merge", &m.source, result) {
            merged += 1;
        } else {
            failed += 1;
        }
    }
    for r in &plan.retypes {
        if used >= budget {
            break;
        }
        used += 1;
        let request = UpdateEntityRequest {
            entity_type: Some(r.to.clone()),
            description: None,
            metadata: None,
        };
        let result = crate::handlers::update_entity(
            State(state.clone()),
            scoped.clone(),
            Path(r.entity.clone()),
            Json(request),
        )
        .await
        .map(|_| format!("{} → {}", r.from, r.to));
        if record(&mut steps, "retype", &r.entity, result) {
            retyped += 1;
        } else {
            failed += 1;
        }
    }
    tracing::info!(workspace_id = %workspace_id, merged, retyped, failed, planned, "Workspace graph realigned to ontology");
    crate::services::record_compliance_event(
        &state,
        ws.tenant_id.to_string(),
        edgequake_audit::AuditEventType::WorkspaceAccess,
        "workspace_ontology_realign",
        if failed == 0 {
            edgequake_audit::AuditResult::Success
        } else {
            edgequake_audit::AuditResult::Warning
        },
        Some(workspace_id.to_string()),
        tenant_ctx.user_id.clone(),
        None,
    );
    if merged + retyped > 0 {
        super::helpers::invalidate_workspace_stats_cache(workspace_id).await;
    }
    Ok(Json(OntologyRealignResponse {
        steps,
        merged,
        retyped,
        failed,
        remaining: planned.saturating_sub(used),
    }))
}
