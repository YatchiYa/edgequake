'use client';

/**
 * Debounced polite live region for the documents feedback zone.
 * Announces Cleaning → Queued → stage/% so AT users hear progress updates.
 */

import { useEffect, useRef, useState } from 'react';

const DEBOUNCE_MS = 800;

export interface FeedbackZoneLiveRegionProps {
  /** Plain-text announcement; empty string clears without speaking. */
  announcement: string;
}

export function FeedbackZoneLiveRegion({
  announcement,
}: FeedbackZoneLiveRegionProps) {
  const [spoken, setSpoken] = useState('');
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const lastSpokenRef = useRef('');

  useEffect(() => {
    const next = announcement.trim();
    if (!next || next === lastSpokenRef.current) return;

    if (timerRef.current) clearTimeout(timerRef.current);
    timerRef.current = setTimeout(() => {
      lastSpokenRef.current = next;
      setSpoken(next);
    }, DEBOUNCE_MS);

    return () => {
      if (timerRef.current) clearTimeout(timerRef.current);
    };
  }, [announcement]);

  return (
    <div
      className="sr-only"
      role="status"
      aria-live="polite"
      aria-atomic="true"
      data-testid="spec051-feedback-live"
    >
      {spoken}
    </div>
  );
}
