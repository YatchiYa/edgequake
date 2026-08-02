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
