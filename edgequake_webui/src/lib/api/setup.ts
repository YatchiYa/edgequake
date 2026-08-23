/**
 * SPEC-101 — First-run setup API client.
 */

import { apiClient } from './client';
import type { Tenant, Workspace } from '@/types';

export interface SetupStatus {
  needs_setup: boolean;
  has_login_users: boolean;
  tenant_count: number;
  workspace_count: number;
  auth_enabled: boolean;
  bootstrap_admin_configured: boolean;
}

export interface SetupInitializeRequest {
  admin_username?: string;
  admin_email?: string;
  admin_password?: string;
  tenant_name: string;
  tenant_description?: string;
  workspace_name: string;
  workspace_slug?: string;
  workspace_description?: string;
  default_llm_model?: string;
  default_llm_provider?: string;
  default_embedding_model?: string;
  default_embedding_provider?: string;
  default_vision_llm_model?: string;
  default_vision_llm_provider?: string;
  pdf_parser_backend?: string;
  extraction_language?: string | null;
  chunking_mode?: string | null;
  chunk_token_size?: number | null;
  chunk_overlap_token_size?: number | null;
  extract_budget_mode?: string | null;
  extract_max_entities?: number | null;
  extract_max_records?: number | null;
  entity_types?: string[];
  entity_types_strict?: boolean;
  entity_type_colors?: Record<string, string>;
  relation_types?: string[];
  relation_types_strict?: boolean;
  kg_schema_preset?: string;
  relation_edges?: Array<{ source: string; relation: string; target: string }>;
  default_reasoning_effort?: string;
  vision_extract_images?: boolean;
  vision_extract_charts?: boolean;
  vision_extract_figures?: boolean;
  vision_page_system_prompt?: string;
  vision_image_system_prompt?: string;
  vision_chart_system_prompt?: string;
  vision_figure_system_prompt?: string;
}

export interface SetupInitializeResponse {
  tenant: Tenant;
  workspace: Workspace;
  admin_username?: string | null;
  already_initialized: boolean;
}

export async function fetchSetupStatus(): Promise<SetupStatus> {
  return apiClient<SetupStatus>('/setup/status');
}

export async function initializeSetup(
  data: SetupInitializeRequest,
): Promise<SetupInitializeResponse> {
  return apiClient<SetupInitializeResponse>('/setup/initialize', {
    method: 'POST',
    body: JSON.stringify(data),
  });
}
