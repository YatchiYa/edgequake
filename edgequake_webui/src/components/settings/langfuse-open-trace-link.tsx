'use client';

/**
 * SPEC-124: Open a Langfuse Session when configured + sessionId present.
 *
 * Per-trace `/trace/{api_trace_id}` is deferred until API `trace_id` matches
 * OTEL TraceId. Sessions are the honest operator deep-link today.
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

/** Build Langfuse session UI URL from settings base (shared with Rust `session_ui_url`). */
export function langfuseSessionHref(uiUrl: string, sessionId: string): string {
  const base = uiUrl.replace(/\/$/, '');
  return `${base}/sessions/${encodeURIComponent(sessionId)}`;
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
        if (cfg.export_active && cfg.ui_url) {
          setHref(langfuseSessionHref(cfg.ui_url, sessionId));
        } else {
          setHref(null);
        }
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
