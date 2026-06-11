/**
 * SPEC-020 health assertions — operational readiness + schema migration signals.
 */
import { expect } from "@playwright/test";
import { BACKEND_URL } from "./backend-url";

export type HealthSnapshot = {
  status?: string;
  version?: string;
  storage_mode?: string;
  components?: Record<string, boolean>;
  llm_provider_name?: string;
  schema?: {
    latest_version?: number;
    migrations_applied?: number;
    source_ids_indexes?: {
      ready?: boolean;
      graphs_checked?: number;
      missing_indexes?: string[];
    };
  };
};

const OPERATIONAL = new Set(["healthy", "degraded"]);

export async function fetchHealth(
  request: { get: (url: string) => Promise<{ ok: () => boolean; json: () => Promise<unknown> }> },
): Promise<HealthSnapshot> {
  const res = await request.get(`${BACKEND_URL}/health`);
  expect(res.ok()).toBeTruthy();
  return (await res.json()) as HealthSnapshot;
}

/** Assert backend is operational (healthy or degraded with all storage components). */
export function assertOperationalHealth(health: HealthSnapshot): void {
  expect(OPERATIONAL.has(health.status ?? "")).toBeTruthy();
  expect(health.storage_mode).toBe("postgresql");
  const c = health.components;
  expect(c?.kv_storage).toBe(true);
  expect(c?.vector_storage).toBe(true);
  expect(c?.graph_storage).toBe(true);
  expect(c?.llm_provider).toBe(true);
}

/** When SPEC020_STRICT_MIGRATION=1, fail QC if migration-038 indexes missing. */
export function assertMigration038IfStrict(
  mig: ReturnType<typeof migration038Status>,
): void {
  if (process.env.SPEC020_STRICT_MIGRATION === "1") {
    expect(mig.ready).toBe(true);
    expect(mig.missingCount).toBe(0);
  }
}

/** K8s readiness — 503 when migration-038 indexes block traffic. */
export async function fetchReady(
  request: { get: (url: string) => Promise<{ status: () => number }> },
): Promise<number> {
  const res = await request.get(`${BACKEND_URL}/ready`);
  return res.status();
}

export function assertReadyIfStrict(status: number): void {
  if (process.env.SPEC020_STRICT_MIGRATION === "1") {
    expect(status).toBe(200);
  }
}

/** K8s liveness — always 200 when process is up. */
export async function fetchLive(
  request: { get: (url: string) => Promise<{ status: () => number; text: () => Promise<string> }> },
): Promise<{ status: number; body: string }> {
  const res = await request.get(`${BACKEND_URL}/live`);
  return { status: res.status(), body: await res.text() };
}

export function assertLiveProbe(live: { status: number; body: string }): void {
  expect(live.status).toBe(200);
  expect(live.body.toUpperCase()).toContain("OK");
}

/** Record migration-038 readiness without failing QC on legacy dev DBs. */
export function migration038Status(health: HealthSnapshot): {
  ready: boolean;
  missingCount: number;
  migrationsApplied: number;
  latestVersion: number;
} {
  const schema = health.schema ?? {};
  const idx = schema.source_ids_indexes;
  return {
    ready: idx?.ready ?? true,
    missingCount: idx?.missing_indexes?.length ?? 0,
    migrationsApplied: schema.migrations_applied ?? 0,
    latestVersion: schema.latest_version ?? 0,
  };
}
