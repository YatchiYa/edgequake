/**
 * SSOT for WebUI `/live`+`/health` refetch intervals.
 *
 * Default is off (one probe on mount). Opt in with EDGEQUAKE_HEALTH_POLL_MS.
 * Playwright/Cypress always disable the loop.
 */

import { getRuntimeConfig } from "@/lib/runtime-config";
import { isAutomatedBrowser } from "@/lib/runtime/browser-detection";

export const HEALTH_DETAILS_MIN_MS = 30_000;

export function resolveHealthPollIntervals(
  configuredMs: number | false,
  isAutomated: boolean,
): { backendReady: number | false; healthDetails: number | false } {
  if (isAutomated || configuredMs === false) {
    return { backendReady: false, healthDetails: false };
  }
  return {
    backendReady: configuredMs,
    healthDetails: Math.max(configuredMs, HEALTH_DETAILS_MIN_MS),
  };
}

export function getBackendReadyRefetchInterval(): number | false {
  return resolveHealthPollIntervals(
    getRuntimeConfig().healthPollIntervalMs,
    isAutomatedBrowser(),
  ).backendReady;
}

export function getHealthDetailsRefetchInterval(): number | false {
  return resolveHealthPollIntervals(
    getRuntimeConfig().healthPollIntervalMs,
    isAutomatedBrowser(),
  ).healthDetails;
}
