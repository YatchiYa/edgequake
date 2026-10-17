'use client';

import { StreamingMarkdown } from '@/components/query/markdown';
import { Button } from '@/components/ui/button';
import { Dialog, DialogContent, DialogTitle } from '@/components/ui/dialog';
import { Skeleton } from '@/components/ui/skeleton';
import { agentsApi, agentsFetch, type ArtifactRef } from '@/lib/agents/api';
import { useQuery } from '@tanstack/react-query';
import { CircleAlert, Download, FileText, Maximize2 } from 'lucide-react';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';

type Kind = 'image' | 'html' | 'markdown' | 'file';

function kindOf(a: ArtifactRef): Kind {
  const mime = a.mime_type.toLowerCase();
  if (mime.startsWith('image/')) return 'image';
  if (mime.startsWith('text/html')) return 'html';
  if (mime.startsWith('text/markdown') || /\.md$/i.test(a.name)) return 'markdown';
  return 'file';
}

/** Artifacts need the bearer, so they are fetched as blobs (an <img src> cannot carry it). */
function useArtifact(conversationId: string, artifact: ArtifactRef, withText: boolean) {
  const path = agentsApi.artifactUrl(conversationId, artifact);
  return useQuery({
    queryKey: ['agents-platform', 'artifact', path],
    queryFn: async () => {
      const res = await agentsFetch(path);
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      const blob = await res.blob();
      // ponytail: object URLs live as long as the tab (artifacts are small); revoke on cache eviction if that changes.
      return { url: URL.createObjectURL(blob), text: withText ? await blob.text() : null };
    },
    staleTime: Infinity,
    gcTime: Infinity,
    retry: 1,
  });
}

export function ArtifactCard({ conversationId, artifact }: { conversationId: string; artifact: ArtifactRef }) {
  const { t } = useTranslation();
  const kind = kindOf(artifact);
  const { data, isError } = useArtifact(conversationId, artifact, kind === 'html' || kind === 'markdown');
  const url = data?.url ?? null;
  const text = data?.text ?? null;
  const [open, setOpen] = useState(false);

  const download = url && (
    <Button asChild variant="ghost" size="icon-sm" aria-label={t('agentsPlatform.artifact.download', { name: artifact.name })}>
      <a href={url} download={artifact.name}>
        <Download />
      </a>
    </Button>
  );
  const header = (
    <div className="flex items-center gap-2 border-b px-3 py-1.5 text-sm">
      <FileText className="size-4 shrink-0 text-muted-foreground" aria-hidden="true" />
      <span className="min-w-0 flex-1 truncate font-medium" title={artifact.name}>
        {artifact.name}
      </span>
      {artifact.version > 0 && <span className="text-xs text-muted-foreground">v{artifact.version + 1}</span>}
      {(kind === 'html' || kind === 'image') && url && (
        <Button variant="ghost" size="icon-sm" onClick={() => setOpen(true)} aria-label={t('agentsPlatform.artifact.fullscreen')}>
          <Maximize2 />
        </Button>
      )}
      {download}
    </div>
  );

  if (isError) {
    return (
      <div className="flex items-center gap-2 rounded-xl border border-destructive/30 px-3 py-2 text-sm text-destructive">
        <CircleAlert className="size-4" /> {t('agentsPlatform.artifact.failed', { name: artifact.name })}
      </div>
    );
  }

  if (kind === 'image') {
    return (
      <figure className="w-fit max-w-full overflow-hidden rounded-xl border bg-card">
        {url ? (
          <button type="button" onClick={() => setOpen(true)} className="block cursor-zoom-in" aria-label={t('agentsPlatform.artifact.enlarge', { name: artifact.name })}>
            {/* eslint-disable-next-line @next/next/no-img-element -- blob URL */}
            <img src={url} alt={artifact.name} className="max-h-80 w-auto max-w-full object-contain" />
          </button>
        ) : (
          <Skeleton className="h-56 w-80 max-w-full rounded-none" />
        )}
        <figcaption>{header}</figcaption>
        <Dialog open={open} onOpenChange={setOpen}>
          <DialogContent className="w-auto max-w-[95vw] p-2 sm:max-w-[95vw]">
            <DialogTitle className="sr-only">{artifact.name}</DialogTitle>
            {/* eslint-disable-next-line @next/next/no-img-element -- blob URL */}
            {url && <img src={url} alt={artifact.name} className="max-h-[88vh] max-w-full rounded-lg object-contain" />}
          </DialogContent>
        </Dialog>
      </figure>
    );
  }

  if (kind === 'html') {
    // Opaque origin (no allow-same-origin): scripts run but cannot reach the app, cookies or storage.
    const frame = (className: string) =>
      text === null ? (
        <Skeleton className={`${className} rounded-none`} />
      ) : (
        <iframe title={artifact.name} sandbox="allow-scripts" srcDoc={text} className={`${className} w-full border-0 bg-white`} />
      );
    return (
      <div className="overflow-hidden rounded-xl border bg-card">
        {header}
        {frame('h-80')}
        <Dialog open={open} onOpenChange={setOpen}>
          <DialogContent className="flex h-[92vh] w-[95vw] max-w-[95vw] flex-col gap-0 p-0 sm:max-w-[95vw]">
            <DialogTitle className="border-b px-4 py-3 pr-12 text-sm">{artifact.name}</DialogTitle>
            {frame('min-h-0 flex-1 rounded-b-2xl')}
          </DialogContent>
        </Dialog>
      </div>
    );
  }

  if (kind === 'markdown') {
    return (
      <div className="overflow-hidden rounded-xl border bg-card">
        {header}
        <div className="max-h-96 overflow-y-auto px-4 py-3 text-sm">
          {text === null ? <Skeleton className="h-16 w-full" /> : <StreamingMarkdown content={text} />}
        </div>
      </div>
    );
  }

  return <div className="w-fit min-w-64 overflow-hidden rounded-xl border bg-card">{header}</div>;
}
