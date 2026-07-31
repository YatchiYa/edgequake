/**
 * Parse resource — SPEC-094 stateless PDF → Markdown.
 *
 * @module resources/parse
 */

import type { HttpTransport } from "../transport/types.js";
import { Resource } from "./base.js";

/** Per-request parse options. */
export interface ParseOptions {
  backend?: "vision" | "edgeparse" | string;
  provider?: string;
  model?: string;
  dpi?: number;
  concurrency?: number;
  pages?: string;
  table_method?: string;
  emit_assets?: boolean;
  allow_fallback?: boolean;
  include_page_timings?: boolean;
  async?: boolean;
}

export interface ParseMetrics {
  total_ms: number;
  render_ms?: number;
  ocr_ms?: number;
  assemble_ms?: number;
  pages_per_second?: number;
  prompt_tokens?: number;
  completion_tokens?: number;
  estimated_cost_usd?: number;
}

export interface ParseResponse {
  markdown: string;
  backend: string;
  backend_effective: string;
  fallback_applied: boolean;
  page_count: number;
  metrics: ParseMetrics;
  page_timings?: Array<{ page: number; ms: number; chars: number }>;
  warnings: string[];
  request_id: string;
}

export interface ParseAsyncAccepted {
  job_id: string;
  status: string;
  request_id: string;
}

export interface ParseJobStatusResponse {
  job_id: string;
  status: string;
  result?: ParseResponse;
  error?: { code: string; message: string };
  request_id: string;
}

export interface ParseBackendsResponse {
  backends: Array<{
    name: string;
    available: boolean;
    providers: Array<{ name: string; available: boolean; models: string[] }>;
  }>;
  limits: {
    sync_max_pages: number;
    sync_max_bytes: number;
    async_max_pages: number;
    async_max_bytes: number;
    max_concurrency: number;
    dpi_min: number;
    dpi_max: number;
  };
  default_backend: string;
}

export class ParseResource extends Resource {
  constructor(transport: HttpTransport) {
    super(transport);
  }

  /**
   * Parse a PDF to Markdown (sync by default).
   * Pass `options.async = true` or Prefer respond-async via headers for 202 jobs.
   */
  async parse(
    file: File | Blob,
    options?: ParseOptions,
  ): Promise<ParseResponse | ParseAsyncAccepted> {
    const meta: Record<string, string> = {};
    if (options) {
      meta["options"] = JSON.stringify(options);
    }
    return this.transport.upload("/api/v1/parse", file, meta);
  }

  /** List available parse backends and limits. */
  async backends(): Promise<ParseBackendsResponse> {
    return this._get("/api/v1/parse/backends");
  }

  /** Poll an async parse job. */
  async job(id: string): Promise<ParseJobStatusResponse> {
    return this._get(`/api/v1/parse/jobs/${encodeURIComponent(id)}`);
  }
}
