/**
 * Workspace ontology API (SPEC-184).
 *
 * The ontology lives in workspace metadata; saving it also rewrites the workspace's
 * entity/relation type lists, and removing it restores them.
 */

import { api } from "../client";

export interface OntologyEntityType {
  id: string;
  label?: string;
  description?: string;
  parent?: string | null;
  synonyms?: string[];
}

export interface OntologyRelationType {
  id: string;
  description?: string;
  source_types?: string[];
  target_types?: string[];
}

export interface OntologyTerm {
  name: string;
  entity_type?: string | null;
  aliases?: string[];
  description?: string;
}

export interface OntologySource {
  kind: string;
  name: string;
  document_id?: string | null;
  at?: string | null;
}

export interface WorkspaceOntology {
  version?: number;
  name: string;
  description?: string;
  entity_types: OntologyEntityType[];
  relation_types: OntologyRelationType[];
  terms: OntologyTerm[];
  sources?: OntologySource[];
  updated_at?: string | null;
}

export interface OntologyCounts {
  entity_types: number;
  relation_types: number;
  terms: number;
  aliases: number;
}

export interface OntologyResponse {
  ontology: WorkspaceOntology | null;
  counts: OntologyCounts;
  warnings: string[];
}

export interface OntologyDraftRequest {
  text: string;
  file_name?: string;
  document_id?: string;
  source_kind?: "file" | "document" | "text" | "copilot";
}

export interface OntologyDraftResponse {
  ontology: WorkspaceOntology;
  counts: OntologyCounts;
  format: "json" | "table" | "llm";
  warnings: string[];
  segments: number;
  truncated: boolean;
}

export interface PlannedMerge {
  source: string;
  target: string;
  target_exists: boolean;
  target_type: string;
  source_degree: number;
}

export interface PlannedRetype {
  entity: string;
  from: string;
  to: string;
}

export interface OntologyPlanResponse {
  plan: {
    merges: PlannedMerge[];
    retypes: PlannedRetype[];
    terms_total: number;
    terms_in_graph: number;
  };
  types: { entity_type: string; count: number; in_ontology: boolean }[];
  entities_total: number;
  entities_in_ontology_types: number;
  documents: number;
}

export interface OntologyRealignResponse {
  steps: { op: string; entity: string; ok: boolean; detail: string }[];
  merged: number;
  retyped: number;
  failed: number;
  remaining: number;
}

/** LLM drafts read a long document in several model calls. */
const DRAFT_TIMEOUT_MS = 10 * 60 * 1000;
const REALIGN_TIMEOUT_MS = 5 * 60 * 1000;

const base = (workspaceId: string) => `/workspaces/${workspaceId}/ontology`;

export function getWorkspaceOntology(workspaceId: string): Promise<OntologyResponse> {
  return api.get<OntologyResponse>(base(workspaceId));
}

export function saveWorkspaceOntology(
  workspaceId: string,
  ontology: WorkspaceOntology,
): Promise<OntologyResponse> {
  return api.put<OntologyResponse>(base(workspaceId), ontology);
}

export function deleteWorkspaceOntology(workspaceId: string): Promise<OntologyResponse> {
  return api.delete<OntologyResponse>(base(workspaceId));
}

export function draftWorkspaceOntology(
  workspaceId: string,
  request: OntologyDraftRequest,
): Promise<OntologyDraftResponse> {
  return api.post<OntologyDraftResponse>(`${base(workspaceId)}/draft`, request, {
    timeoutMs: DRAFT_TIMEOUT_MS,
  });
}

export function planWorkspaceOntology(
  workspaceId: string,
  ontology?: WorkspaceOntology,
): Promise<OntologyPlanResponse> {
  return api.post<OntologyPlanResponse>(`${base(workspaceId)}/plan`, ontology ? { ontology } : {});
}

export function realignWorkspaceOntology(
  workspaceId: string,
  maxOperations?: number,
): Promise<OntologyRealignResponse> {
  return api.post<OntologyRealignResponse>(
    `${base(workspaceId)}/realign`,
    maxOperations ? { max_operations: maxOperations } : {},
    { timeoutMs: REALIGN_TIMEOUT_MS },
  );
}
