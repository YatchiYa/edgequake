'use client';

/**
 * SPEC-124: Open a Langfuse Session when export is active + project id known.
 *
 * Deep links must use the configured `LANGFUSE_BASE_URL` (`ui_url`) and the
 * project-scoped path `/project/{id}/sessions/{sessionId}`. Bare `/sessions/{id}`
 * is a Langfuse Cloud 404.
 */

import { Button } from '@/components/ui/button';
import { apiClient } from '@/lib/api/client';
import { ExternalLink } from 'lucide-react';
import { useEffect, useState } from 'react';
import type { LangfuseSettingsResponse } from './langfuse-observability-card';

type Props = {
  /** Chat conversation_id or query session_id bound as Langfuse session. */
  sessionId?: string | null;
  className?: string;
};

/**
 * Build a valid Langfuse session URL from the configured host + project id.
 * Returns null when Langfuse is not activated or the project id is unknown
 * (never emit a 404 `/sessions/{id}` URL).
 */
export function langfuseSessionHref(
  uiUrl: string,
  sessionId: string,
  projectId?: string | null,
): string | null {
  const base = uiUrl.replace(/\/$/, '');
  const sid = sessionId.trim();
  const pid = projectId?.trim();
  if (!base || !sid || !pid) {
    return null;
  }
  return `${base}/project/${encodeURIComponent(pid)}/sessions/${encodeURIComponent(sid)}`;
}

function langfuseIsActivated(cfg: LangfuseSettingsResponse): boolean {
  return Boolean(cfg.export_active && (cfg.ui_url || cfg.base_url));
}

export function LangfuseOpenSessionLink({ sessionId, className }: Props) {
  const [href, setHref] = useState<string | null>(null);

  useEffect(() => {
    if (!sessionId) {
      setHref(null);
      return;
    }
    let cancelled = false;
    void (async () => {
      try {
        const cfg = await apiClient<LangfuseSettingsResponse>('/settings/langfuse');
        if (cancelled) return;
        const host = cfg.ui_url || cfg.base_url;
        if (!langfuseIsActivated(cfg) || !cfg.project_id || !host) {
          setHref(null);
          return;
        }
        setHref(langfuseSessionHref(host, sessionId, cfg.project_id));
      } catch {
        if (!cancelled) setHref(null);
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [sessionId]);

  if (!href) return null;

  return (
    <Button type="button" variant="ghost" size="sm" className={className} asChild>
      <a
        href={href}
        target="_blank"
        rel="noopener noreferrer"
        data-testid="langfuse-open-session"
        data-id="langfuse-open-session"
      >
        Open session in Langfuse
        <ExternalLink className="h-3 w-3 ml-1.5" aria-hidden />
      </a>
    </Button>
  );
}

/** @deprecated Use LangfuseOpenSessionLink — TraceId deep-links are not honest yet. */
export const LangfuseOpenTraceLink = LangfuseOpenSessionLink;
