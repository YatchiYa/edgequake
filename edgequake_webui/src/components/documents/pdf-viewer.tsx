/**
 * @module PDFViewer
 * @description Reusable PDF viewer using react-pdf.
 * SPEC-143: continuous multi-page scroll stack + onPageChange for Markdown sync.
 *
 * @implements SPEC-002 - Document Viewer with PDF display
 * @implements SPEC-033 - Controlled currentPage / deeplink
 * @implements SPEC-143 - Continuous stack, wheel page nav, onPageChange
 * @implements FEAT0711 - PDF rendering with react-pdf
 * @implements FEAT0712 - Page navigation controls
 * @implements FEAT0713 - Zoom controls
 */
'use client';

import {
  PdfPageOverlay,
  type OverlayChips,
} from '@/components/documents/pdf-page-overlay';
import { Button } from '@/components/ui/button';
import { Skeleton } from '@/components/ui/skeleton';
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from '@/components/ui/tooltip';
import {
  getDocumentPageLayout,
  listDocumentPages,
} from '@/lib/api/edgequake/documents';
import { cn } from '@/lib/utils';
import { useQuery } from '@tanstack/react-query';
import {
  ChevronLeft,
  ChevronRight,
  Loader2,
  Maximize2,
  Minimize2,
  XCircle,
  ZoomIn,
  ZoomOut,
} from 'lucide-react';
import dynamic from 'next/dynamic';
import type { DocumentProps } from 'react-pdf';
import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { useTranslation } from 'react-i18next';
import 'react-pdf/dist/Page/AnnotationLayer.css';
import 'react-pdf/dist/Page/TextLayer.css';

type PDFFileSource =
  | string
  | { url: string }
  | { data: ArrayBuffer | Uint8Array }
  | null;

const Document = dynamic(() => import('react-pdf').then((mod) => mod.Document), {
  ssr: false,
  loading: () => <PDFLoadingSkeleton />,
});

const Page = dynamic(() => import('react-pdf').then((mod) => mod.Page), {
  ssr: false,
});

if (typeof window !== 'undefined') {
  import('react-pdf').then(({ pdfjs }) => {
    pdfjs.GlobalWorkerOptions.workerSrc = '/pdf.worker.min.mjs';
  });
}

/** Windowed render kicks in above this page count (SPEC-143). */
const WINDOW_THRESHOLD = 20;
const WINDOW_RADIUS = 2;

interface PDFViewerProps {
  file: PDFFileSource;
  className?: string;
  initialPage?: number;
  /**
   * Controlled current page (1-indexed).
   * External navigation (deeplink, markdown sync, chunk click).
   */
  currentPage?: number;
  /** SPEC-143: emitted when the active page changes via scroll / toolbar / keyboard. */
  onPageChange?: (page: number) => void;
  initialScale?: number;
  showToolbar?: boolean;
  width?: number;
  height?: number;
  onLoadSuccess?: (numPages: number) => void;
  onLoadError?: (error: Error) => void;
  /** Document id for SPEC-128 layout overlay. */
  documentId?: string;
}

function PDFLoadingSkeleton() {
  return (
    <div className="flex flex-col items-center justify-center p-8 space-y-4">
      <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
      <Skeleton className="h-[400px] w-[300px]" />
    </div>
  );
}

function PDFErrorState({ error, onRetry }: { error: string; onRetry?: () => void }) {
  const { t } = useTranslation();
  const is404 = error.includes('404') || error.includes('not found');
  const isNetworkError =
    error.includes('NetworkError') ||
    error.includes('Failed to fetch') ||
    error.includes('network');

  const displayMessage = is404
    ? t(
        'documents.viewer.pdfNotFound',
        'PDF file is not available. The file may have been removed or processing may not be complete.',
      )
    : isNetworkError
      ? t(
          'documents.viewer.pdfNetworkError',
          'Unable to connect to the server. Please check your connection and try again.',
        )
      : error;

  return (
    <div className="flex flex-col items-center justify-center p-8 space-y-4 text-center">
      <div className="rounded-full bg-muted p-3">
        <XCircle className="h-6 w-6 text-muted-foreground" />
      </div>
      <div className="space-y-1">
        <p className="text-sm font-medium text-muted-foreground">
          {is404
            ? t('documents.viewer.pdfUnavailable', 'PDF Unavailable')
            : t('documents.viewer.loadError', 'Failed to Load PDF')}
        </p>
        <p className="text-xs text-muted-foreground max-w-sm">{displayMessage}</p>
      </div>
      {onRetry && !is404 && (
        <Button variant="outline" size="sm" onClick={onRetry}>
          {t('common.retry', 'Retry')}
        </Button>
      )}
    </div>
  );
}

function layoutToggleDisabled(
  summary: { layout_status: string; region_count?: number | null } | undefined,
  pagesLoaded: boolean,
  pagesError: boolean,
): boolean {
  if (pagesError) return false;
  if (!pagesLoaded) return true;
  if (!summary) return true;
  const status = summary.layout_status;
  const count = summary.region_count ?? 0;
  if (status === 'failed' || status === 'pending') return true;
  if (status === 'skipped' && count === 0) return true;
  return false;
}

function layoutToggleHint(
  summary: { layout_status: string; region_count?: number | null } | undefined,
  disabled: boolean,
  t: (key: string, fallback: string) => string,
): string {
  if (!disabled) {
    return t('documents.viewer.layout.toggleHint', 'Toggle layout regions (O)');
  }
  if (!summary) {
    return t('documents.viewer.layout.unavailable', 'Layout data unavailable');
  }
  return t('documents.viewer.layout.disabled', 'Layout overlay unavailable for this page');
}

export function PDFViewer({
  file,
  className,
  initialPage = 1,
  currentPage,
  onPageChange,
  initialScale = 1.0,
  showToolbar = true,
  width,
  height,
  onLoadSuccess,
  onLoadError,
  documentId,
}: PDFViewerProps) {
  const { t } = useTranslation();
  const [numPages, setNumPages] = useState(0);
  const [pageNumber, setPageNumber] = useState(initialPage);
  const [scale, setScale] = useState(initialScale);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [isFullWidth, setIsFullWidth] = useState(false);
  const [overlayOn, setOverlayOn] = useState(false);
  const [pageBox, setPageBox] = useState<{ width: number; height: number } | null>(null);
  const [basePageHeight, setBasePageHeight] = useState(800);
  const [chips, setChips] = useState<OverlayChips>({
    figures: true,
    charts: true,
    tables: true,
    paragraphs: false,
    columns: false,
    noise: false,
  });

  const scrollRef = useRef<HTMLDivElement>(null);
  const sheetRefs = useRef<Map<number, HTMLDivElement>>(new Map());
  const lastEmittedRef = useRef<number | null>(null);
  const programmaticScrollRef = useRef(false);

  const pagesQuery = useQuery({
    queryKey: ['document-pages', documentId],
    queryFn: () => listDocumentPages(documentId!),
    enabled: Boolean(documentId),
  });
  const pageSummary = pagesQuery.data?.pages?.find((p) => p.page_number === pageNumber);
  const overlayDisabled = layoutToggleDisabled(
    pageSummary,
    pagesQuery.isSuccess,
    pagesQuery.isError,
  );
  const overlayHint = layoutToggleHint(pageSummary, overlayDisabled, t);

  const layoutQuery = useQuery({
    queryKey: ['document-page-layout', documentId, pageNumber],
    queryFn: () => getDocumentPageLayout(documentId!, pageNumber),
    enabled: Boolean(documentId) && overlayOn && pageNumber > 0 && !overlayDisabled,
  });

  const emitPage = useCallback(
    (page: number) => {
      const clamped =
        numPages > 0 ? Math.max(1, Math.min(numPages, page)) : Math.max(1, page);
      setPageNumber(clamped);
      if (lastEmittedRef.current === clamped) return;
      lastEmittedRef.current = clamped;
      onPageChange?.(clamped);
    },
    [numPages, onPageChange],
  );

  const scrollToPage = useCallback(
    (page: number, behavior: ScrollBehavior = 'smooth') => {
      const clamped =
        numPages > 0 ? Math.max(1, Math.min(numPages, page)) : Math.max(1, page);
      const el = sheetRefs.current.get(clamped);
      if (!el) {
        setPageNumber(clamped);
        return;
      }
      programmaticScrollRef.current = true;
      el.scrollIntoView({ behavior, block: 'start' });
      setPageNumber(clamped);
      window.setTimeout(() => {
        programmaticScrollRef.current = false;
      }, 500);
    },
    [numPages],
  );

  // External controlled page (deeplink / markdown sync)
  useEffect(() => {
    if (currentPage === undefined) return;
    const clamped =
      numPages > 0
        ? Math.max(1, Math.min(numPages, currentPage))
        : Math.max(1, currentPage);
    if (clamped === lastEmittedRef.current && clamped === pageNumber) return;
    lastEmittedRef.current = clamped;
    scrollToPage(clamped, 'smooth');
    // eslint-disable-next-line react-hooks/exhaustive-deps -- avoid loop on pageNumber
  }, [currentPage, numPages, scrollToPage]);

  // IntersectionObserver for active page from continuous stack
  useEffect(() => {
    const root = scrollRef.current;
    if (!root || numPages < 1) return;

    const pickActivePage = () => {
      if (programmaticScrollRef.current) return;
      const rootTop = root.getBoundingClientRect().top;
      let best = 1;
      let bestDist = Number.POSITIVE_INFINITY;
      sheetRefs.current.forEach((el, p) => {
        const top = el.getBoundingClientRect().top;
        const dist = Math.abs(top - rootTop - 8);
        if (dist < bestDist) {
          bestDist = dist;
          best = p;
        }
      });
      if (best >= 1) emitPage(best);
    };

    const io = new IntersectionObserver(() => pickActivePage(), {
      root,
      threshold: [0, 0.1, 0.25, 0.5, 1],
    });

    sheetRefs.current.forEach((el) => io.observe(el));
    root.addEventListener('scroll', pickActivePage, { passive: true });
    return () => {
      io.disconnect();
      root.removeEventListener('scroll', pickActivePage);
    };
  }, [numPages, emitPage, scale]);

  // Keyboard: PageUp/Down, Arrows (when not in input)
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if ((e.target as HTMLElement | null)?.closest?.('input,textarea,[contenteditable]')) {
        return;
      }
      if (e.key === 'o' || e.key === 'O') {
        if (overlayDisabled) return;
        setOverlayOn((v) => !v);
        return;
      }
      const root = scrollRef.current;
      if (!root) return;
      // Only when PDF pane contains focus or is hovered / default document focus
      const pdfFocused =
        root.contains(document.activeElement) ||
        root.matches(':hover') ||
        document.activeElement === document.body;
      if (!pdfFocused) return;

      if (e.key === 'PageDown' || e.key === 'ArrowDown') {
        e.preventDefault();
        const next = Math.min(numPages, pageNumber + 1);
        scrollToPage(next);
        emitPage(next);
      } else if (e.key === 'PageUp' || e.key === 'ArrowUp') {
        e.preventDefault();
        const prev = Math.max(1, pageNumber - 1);
        scrollToPage(prev);
        emitPage(prev);
      }
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [overlayDisabled, numPages, pageNumber, scrollToPage, emitPage]);

  const [urlOk, setUrlOk] = useState<boolean | null>(null);
  const [probeError, setProbeError] = useState<string | null>(null);

  useEffect(() => {
    const url = typeof file === 'string' ? file : (file as { url?: string } | null)?.url;
    let cancelled = false;
    Promise.resolve().then(async () => {
      if (!url) {
        if (!cancelled) {
          setUrlOk(true);
          setProbeError(null);
        }
        return;
      }
      if (!cancelled) {
        setUrlOk(null);
        setProbeError(null);
      }
      try {
        const res = await fetch(url, { method: 'HEAD' });
        if (cancelled) return;
        setUrlOk(res.ok);
        if (!res.ok) {
          setProbeError(`ResponseException: Unexpected server response (${res.status})`);
          setIsLoading(false);
        }
      } catch {
        if (!cancelled) setUrlOk(true);
      }
    });
    return () => {
      cancelled = true;
    };
  }, [file]);

  const handleLoadSuccess = useCallback(
    ({ numPages: n }: { numPages: number }) => {
      setNumPages(n);
      setIsLoading(false);
      setError(null);
      onLoadSuccess?.(n);
    },
    [onLoadSuccess],
  );

  const handleLoadError = useCallback(
    (err: Error) => {
      setError(err.message || 'Failed to load PDF');
      setIsLoading(false);
      onLoadError?.(err);
    },
    [onLoadError],
  );

  const goToPreviousPage = useCallback(() => {
    const prev = Math.max(1, pageNumber - 1);
    scrollToPage(prev);
    emitPage(prev);
  }, [pageNumber, scrollToPage, emitPage]);

  const goToNextPage = useCallback(() => {
    const next = Math.min(numPages, pageNumber + 1);
    scrollToPage(next);
    emitPage(next);
  }, [numPages, pageNumber, scrollToPage, emitPage]);

  const zoomIn = useCallback(() => setScale((prev) => Math.min(3.0, prev + 0.25)), []);
  const zoomOut = useCallback(() => setScale((prev) => Math.max(0.5, prev - 0.25)), []);
  const toggleFullWidth = useCallback(() => setIsFullWidth((prev) => !prev), []);

  const useWindowing = numPages > WINDOW_THRESHOLD;
  const pageList = useMemo(() => {
    if (numPages < 1) return [];
    return Array.from({ length: numPages }, (_, i) => i + 1);
  }, [numPages]);

  const placeholderHeight = Math.max(200, Math.round(basePageHeight * scale));

  if (!file) {
    return (
      <div className="flex items-center justify-center p-8 text-muted-foreground">
        {t('documents.viewer.noFile', 'No PDF file selected')}
      </div>
    );
  }

  const displayError = error ?? probeError;
  if (displayError) {
    return (
      <PDFErrorState
        error={displayError}
        onRetry={() => {
          setError(null);
          setProbeError(null);
          setUrlOk(null);
        }}
      />
    );
  }

  if (urlOk === null) {
    return (
      <div className={cn('flex flex-col h-full min-h-0', className)}>
        <PDFLoadingSkeleton />
      </div>
    );
  }

  const documentFile: DocumentProps['file'] = file;

  return (
    <div
      data-testid="pdf-viewer"
      className={cn('flex flex-col h-full min-h-0', className)}
      tabIndex={0}
    >
      {showToolbar && (
        <div className="flex items-center justify-between gap-2 p-2 border-b bg-muted/30">
          <div className="flex items-center gap-1">
            <Button
              variant="ghost"
              size="icon"
              className="h-8 w-8"
              onClick={goToPreviousPage}
              disabled={pageNumber <= 1 || isLoading}
              title={t('documents.viewer.previousPage', 'Previous page')}
              data-testid="pdf-prev-page"
            >
              <ChevronLeft className="h-4 w-4" />
            </Button>
            <span
              className="text-sm text-muted-foreground min-w-[80px] text-center"
              data-testid="pdf-page-indicator"
              data-page={pageNumber}
            >
              {isLoading ? '...' : `${pageNumber} / ${numPages}`}
            </span>
            <Button
              variant="ghost"
              size="icon"
              className="h-8 w-8"
              onClick={goToNextPage}
              disabled={pageNumber >= numPages || isLoading}
              title={t('documents.viewer.nextPage', 'Next page')}
              data-testid="pdf-next-page"
            >
              <ChevronRight className="h-4 w-4" />
            </Button>
          </div>

          <div className="flex items-center gap-1">
            <Button
              variant="ghost"
              size="icon"
              className="h-8 w-8"
              onClick={zoomOut}
              disabled={scale <= 0.5 || isLoading}
              title={t('documents.viewer.zoomOut', 'Zoom out')}
            >
              <ZoomOut className="h-4 w-4" />
            </Button>
            <span className="text-sm text-muted-foreground min-w-[50px] text-center">
              {Math.round(scale * 100)}%
            </span>
            <Button
              variant="ghost"
              size="icon"
              className="h-8 w-8"
              onClick={zoomIn}
              disabled={scale >= 3.0 || isLoading}
              title={t('documents.viewer.zoomIn', 'Zoom in')}
            >
              <ZoomIn className="h-4 w-4" />
            </Button>
            <Button
              variant="ghost"
              size="icon"
              className="h-8 w-8"
              onClick={toggleFullWidth}
              title={
                isFullWidth
                  ? t('documents.viewer.fitWidth', 'Fit width')
                  : t('documents.viewer.fullWidth', 'Full width')
              }
            >
              {isFullWidth ? (
                <Minimize2 className="h-4 w-4" />
              ) : (
                <Maximize2 className="h-4 w-4" />
              )}
            </Button>
            {documentId ? (
              <Tooltip>
                <TooltipTrigger asChild>
                  <span className="inline-flex">
                    <Button
                      variant={overlayOn ? 'secondary' : 'ghost'}
                      size="sm"
                      className="h-8 px-2 text-xs"
                      data-testid="pdf-layout-toggle"
                      aria-pressed={overlayOn}
                      disabled={overlayDisabled}
                      onClick={() => {
                        if (overlayDisabled) return;
                        setOverlayOn((v) => !v);
                      }}
                      title={overlayHint}
                    >
                      {t('documents.viewer.layout.toggle', 'Layout')}
                    </Button>
                  </span>
                </TooltipTrigger>
                <TooltipContent>{overlayHint}</TooltipContent>
              </Tooltip>
            ) : null}
          </div>
        </div>
      )}
      {overlayOn && documentId && !overlayDisabled ? (
        <div className="flex flex-wrap items-center gap-1 px-2 py-1 border-b bg-muted/20 text-xs">
          {(
            [
              ['figures', t('documents.viewer.layout.chipFigures', 'Figures')],
              ['charts', t('documents.viewer.layout.chipCharts', 'Charts')],
              ['tables', t('documents.viewer.layout.chipTables', 'Tables')],
              ['paragraphs', t('documents.viewer.layout.chipParagraphs', 'Paragraphs')],
              ['columns', t('documents.viewer.layout.chipColumns', 'Columns')],
              ['noise', t('documents.viewer.layout.chipNoise', 'Noise')],
            ] as const
          ).map(([key, label]) => (
            <Button
              key={key}
              variant={chips[key] ? 'secondary' : 'ghost'}
              size="sm"
              className="h-6 px-2 text-xs"
              data-testid={`pdf-layout-chip-${key}`}
              onClick={() => setChips((c) => ({ ...c, [key]: !c[key] }))}
            >
              {label}
            </Button>
          ))}
        </div>
      ) : null}

      <div
        ref={scrollRef}
        className={cn(
          'flex-1 min-h-0 overflow-y-auto overflow-x-hidden',
          'scroll-smooth bg-muted/10',
        )}
        style={{
          ...(height ? { height: `${height}px` } : {}),
          WebkitOverflowScrolling: 'touch',
        }}
        data-testid="pdf-scroll-container"
      >
        <div className="flex flex-col items-center gap-4 py-4">
          <Document
            file={documentFile}
            onLoadSuccess={handleLoadSuccess}
            onLoadError={handleLoadError}
            loading={<PDFLoadingSkeleton />}
            className="pdf-document w-full flex flex-col items-center gap-4"
          >
            {pageList.map((n) => {
              const inWindow =
                !useWindowing || Math.abs(n - pageNumber) <= WINDOW_RADIUS;
              return (
                <div
                  key={n}
                  ref={(el) => {
                    if (el) sheetRefs.current.set(n, el);
                    else sheetRefs.current.delete(n);
                  }}
                  data-testid="pdf-page-sheet"
                  data-page={n}
                  className="relative shadow-md bg-background"
                  style={
                    !inWindow
                      ? { width: pageBox?.width ?? width ?? 600, height: placeholderHeight }
                      : pageBox && n === pageNumber
                        ? { width: pageBox.width, minHeight: pageBox.height }
                        : undefined
                  }
                >
                  {inWindow ? (
                    <Page
                      pageNumber={n}
                      scale={scale}
                      width={isFullWidth ? undefined : width}
                      className="shadow-md"
                      renderTextLayer={n === pageNumber}
                      renderAnnotationLayer={n === pageNumber}
                      onRenderSuccess={(page) => {
                        if (n === pageNumber || !pageBox) {
                          setPageBox({ width: page.width, height: page.height });
                        }
                        if (scale > 0) {
                          setBasePageHeight(page.height / scale);
                        }
                      }}
                      loading={
                        <div className="flex items-center justify-center p-8">
                          <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
                        </div>
                      }
                    />
                  ) : (
                    <div
                      className="flex items-center justify-center text-xs text-muted-foreground"
                      style={{ height: placeholderHeight }}
                    >
                      {t('documents.viewer.pagePlaceholder', 'Page {{n}}', { n })}
                    </div>
                  )}
                  {inWindow &&
                  n === pageNumber &&
                  overlayOn &&
                  pageBox &&
                  !overlayDisabled ? (
                    <PdfPageOverlay
                      regions={layoutQuery.data?.regions ?? []}
                      chips={chips}
                      empty={
                        layoutQuery.data?.layout_status === 'extracted' &&
                        (layoutQuery.data.regions?.length ?? 0) === 0
                      }
                    />
                  ) : null}
                </div>
              );
            })}
          </Document>
        </div>
      </div>
    </div>
  );
}

export default PDFViewer;
