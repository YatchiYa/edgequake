/**
 * Provider connection types.
 *
 * @see edgequake/crates/edgequake-api/src/handlers/connections.rs
 * @see edgequake/crates/edgequake-api/src/providers/probe.rs
 */

export interface Connection {
  id: string;
  tenant_id?: string | null;
  slug: string;
  display_name: string;
  api_shape: string;
  locality: string;
  base_url: string;
  auth_scheme: string;
  key_fingerprint?: string | null;
  key_configured: boolean;
  timeout_secs: number;
  allow_private_network: boolean;
  source: string;
  last_test_ok?: boolean | null;
  last_test_error?: string | null;
}

export interface UpsertConnection {
  slug: string;
  display_name: string;
  api_shape: string;
  base_url: string;
  locality?: string;
  auth_scheme?: string;
  /** Write-only. Never returned on Connection. */
  api_key?: string;
  timeout_secs?: number;
  allow_private_network?: boolean;
}

export interface ProviderTestRequest {
  shape: string;
  base_url?: string;
  model?: string;
  embedding_model?: string;
  api_key?: string;
  auth_scheme?: string;
  allow_private_network?: boolean;
  expected_dimension?: number;
}

export interface ProbeResult {
  ok: boolean;
  kind: string;
  latency_ms: number;
  message: string;
  models: string[];
  embedding_dimension?: number | null;
  chat_ok: boolean;
  embed_ok: boolean;
  list_ok: boolean;
}
