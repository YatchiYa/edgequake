/**
<<<<<<< HEAD
 * Load markdown images that require API auth headers.
 *
 * Browser `<img src>` cannot send `Authorization`. For `/mm-assets/` URLs we
 * fetch with the session bearer token and display a blob URL (DRY with PDF/WS
 * auth patterns).
 */
'use client';

import { getTokens } from '@/lib/api/client';
=======
 * Load markdown images that require API auth + tenant/workspace headers.
 *
 * Browser `<img src>` cannot send `Authorization`, `X-Tenant-ID`, or
 * `X-Workspace-ID`. mm-asset serving is workspace-scoped (SPEC-091 SSOT): a
 * headerless `<img src>` defaults to the default workspace on the backend and
 * 404s the document. For mm-asset URLs we therefore fetch with the full session
 * headers (`buildHeaders` — tenant/workspace + optional bearer) and display a
 * blob URL (DRY with PDF/WS auth patterns).
 */
'use client';

import { buildHeaders } from '@/lib/api/client';
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
import { useEffect, useState } from 'react';

interface AuthenticatedMarkdownImageProps {
  src: string;
  alt?: string;
  title?: string;
  className?: string;
}

function isMmAssetUrl(src: string): boolean {
  // Path splat: …/mm-assets/assets/page-0001.png
  // Id REST:    …/documents/{id}/assets/page-0001
  return (
    src.includes('/mm-assets/') ||
    /\/documents\/[^/]+\/assets\/[^/]+/.test(src)
  );
}

export function AuthenticatedMarkdownImage({
  src,
  alt,
  title,
  className,
}: AuthenticatedMarkdownImageProps) {
  const [resolvedSrc, setResolvedSrc] = useState<string | null>(
    isMmAssetUrl(src) ? null : src,
  );

  useEffect(() => {
    if (!isMmAssetUrl(src)) {
      setResolvedSrc(src);
      return;
    }

<<<<<<< HEAD
    const token = getTokens().accessToken;
    // Dev (auth off): direct src works. Auth on without token: still try direct.
    if (!token) {
      setResolvedSrc(src);
      return;
    }

=======
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
    let objectUrl: string | null = null;
    const ac = new AbortController();

    (async () => {
<<<<<<< HEAD
      const headers = new Headers();
      headers.set('Authorization', `Bearer ${token}`);
=======
      // Always fetch with the full session headers (tenant/workspace + optional
      // bearer) — a plain <img src> would 404 under workspace scoping. This holds
      // in dev (auth off) too, where workspace scoping still applies.
      const headers = buildHeaders();
      headers.delete('Content-Type'); // GET with no body
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
      const res = await fetch(src, { headers, signal: ac.signal });
      if (!res.ok) {
        setResolvedSrc(src);
        return;
      }
      const blob = await res.blob();
      objectUrl = URL.createObjectURL(blob);
      setResolvedSrc(objectUrl);
    })().catch(() => {
      if (!ac.signal.aborted) {
        setResolvedSrc(src);
      }
    });

    return () => {
      ac.abort();
      if (objectUrl) {
        URL.revokeObjectURL(objectUrl);
      }
    };
  }, [src]);

  if (!resolvedSrc) {
    return (
      <span className="text-muted-foreground text-sm italic my-2 inline-block">
        Loading image…
      </span>
    );
  }

  return (
    // eslint-disable-next-line @next/next/no-img-element
    <img
      src={resolvedSrc}
      alt={alt ?? ''}
      title={title}
      className={className}
      loading="lazy"
    />
  );
}
