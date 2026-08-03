/**
 * SPEC-101 LAW-101-11 — Pure helpers for Tenant/Workspace context chip copy.
 */

export interface ContextNames {
  tenantName?: string | null;
  workspaceName?: string | null;
}

export interface FormattedContextLabels {
  /** Full name (or placeholder) — use for title / aria / e2e. */
  tenantDisplay: string;
  workspaceDisplay: string;
  /** End-biased truncated labels for narrow chrome. */
  tenantShort: string;
  workspaceShort: string;
  /** One-line chrome: `Tenant — Workspace` (shortened). */
  lineDisplay: string;
  title: string;
  ariaLabel: string;
  hasTenant: boolean;
  hasWorkspace: boolean;
}

const DEFAULT_SELECT_TENANT = 'Select tenant';
const DEFAULT_SELECT_WORKSPACE = 'Select workspace';

/**
 * Truncate long names while keeping the distinctive suffix (timestamps / IDs).
 */
export function smartTruncate(text: string, maxLen = 22): string {
  const trimmed = text.trim();
  if (trimmed.length <= maxLen) return trimmed;
  if (maxLen < 8) return `${trimmed.slice(0, Math.max(0, maxLen - 1))}…`;
  const endKeep = Math.min(8, Math.floor(maxLen * 0.35));
  const startKeep = maxLen - endKeep - 1;
  return `${trimmed.slice(0, startKeep)}…${trimmed.slice(-endKeep)}`;
}

/**
 * Build display + accessibility strings for the context trigger.
 * Product: one-line `Tenant — Workspace`; full names in title/aria/data-full-name.
 */
export function formatContextLabels(
  names: ContextNames,
  options?: {
    selectTenant?: string;
    selectWorkspace?: string;
    tenantLabel?: string;
    workspaceLabel?: string;
    /** Max length per side of the one-line chip. */
    maxLen?: number;
  },
): FormattedContextLabels {
  const selectTenant = options?.selectTenant ?? DEFAULT_SELECT_TENANT;
  const selectWorkspace = options?.selectWorkspace ?? DEFAULT_SELECT_WORKSPACE;
  const tenantLabel = options?.tenantLabel ?? 'Tenant';
  const workspaceLabel = options?.workspaceLabel ?? 'Workspace';
  const maxLen = options?.maxLen ?? 18;

  const tenantTrimmed = typeof names.tenantName === 'string' ? names.tenantName.trim() : '';
  const workspaceTrimmed =
    typeof names.workspaceName === 'string' ? names.workspaceName.trim() : '';

  const hasTenant = tenantTrimmed.length > 0;
  const hasWorkspace = workspaceTrimmed.length > 0;

  const tenantDisplay = hasTenant ? tenantTrimmed : selectTenant;
  const workspaceDisplay = hasWorkspace ? workspaceTrimmed : selectWorkspace;

  const tenantShort = hasTenant ? smartTruncate(tenantDisplay, maxLen) : tenantDisplay;
  const workspaceShort = hasWorkspace
    ? smartTruncate(workspaceDisplay, maxLen)
    : workspaceDisplay;

  const title = hasTenant
    ? `${tenantDisplay} — ${workspaceDisplay}`
    : selectTenant;

  const lineDisplay = hasTenant
    ? `${tenantShort} — ${workspaceShort}`
    : selectTenant;

  const ariaLabel = `${tenantLabel} ${tenantDisplay}, ${workspaceLabel} ${workspaceDisplay}`;

  return {
    tenantDisplay,
    workspaceDisplay,
    tenantShort,
    workspaceShort,
    lineDisplay,
    title,
    ariaLabel,
    hasTenant,
    hasWorkspace,
  };
}
