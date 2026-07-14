/**
 * @module useDocumentPreferences
 * @description Manages document list preferences with localStorage persistence.
 * Extracted from DocumentManager for SRP compliance (OODA-17).
 *
 * WHY: Preferences state and localStorage logic were inline in DocumentManager.
 * This hook:
 * - Initializes state from localStorage
 * - Persists changes to localStorage
 * - Handles SSR safely (typeof window check)
 * - Silently handles localStorage errors (incognito mode)
 *
 * @implements FEAT0004 - User preference persistence
 */
"use client";

import {
  isSortDirection,
  isSortField,
  type SortDirection,
  type SortField,
} from "@/lib/documents/document-sort";
import { useEffect, useState } from "react";

export type { SortDirection, SortField };

/**
 * Document status filter values.
 */
export type DocStatus =
  | "all"
  | "pending"
  | "processing"
  | "completed"
  | "failed"
  | "partial_failure"
  | "cancelled";

/**
 * localStorage key for document preferences.
 */
const STORAGE_KEY = "edgequake:documents:prefs";

/**
 * Default values for preferences.
 */
const DEFAULTS = {
  pageSize: 20,
  statusFilter: "all" as DocStatus,
  sortField: "created_at" as SortField,
  sortDirection: "desc" as SortDirection,
};

/**
 * Valid page sizes.
 */
const VALID_PAGE_SIZES = [10, 20, 50, 100];

/**
 * Return type for useDocumentPreferences hook.
 */
export interface UseDocumentPreferencesReturn {
  /** Number of items per page */
  pageSize: number;
  setPageSize: (size: number) => void;

  /** Status filter value */
  statusFilter: DocStatus;
  setStatusFilter: (status: DocStatus) => void;

  /** Sort field */
  sortField: SortField;
  setSortField: (field: SortField) => void;

  /** Sort direction */
  sortDirection: SortDirection;
  setSortDirection: (direction: SortDirection) => void;
}

/**
 * Read preferences from localStorage.
 * WHY: Centralized parsing with validation.
 */
function readPreferences(): Partial<{
  pageSize: number;
  statusFilter: DocStatus;
  sortField: SortField;
  sortDirection: SortDirection;
}> {
  if (typeof window === "undefined") return {};

  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (!stored) return {};
    return JSON.parse(stored);
  } catch {
    return {};
  }
}

/**
 * Hook for managing document list preferences with persistence.
 */
export function useDocumentPreferences(): UseDocumentPreferencesReturn {
  const [pageSize, setPageSize] = useState(() => {
    const prefs = readPreferences();
    const size = prefs.pageSize;
    return VALID_PAGE_SIZES.includes(size ?? 0) ? size! : DEFAULTS.pageSize;
  });

  const [statusFilter, setStatusFilter] = useState<DocStatus>(() => {
    const prefs = readPreferences();
    return prefs.statusFilter || DEFAULTS.statusFilter;
  });

  const [sortField, setSortField] = useState<SortField>(() => {
    const prefs = readPreferences();
    return isSortField(prefs.sortField) ? prefs.sortField : DEFAULTS.sortField;
  });

  const [sortDirection, setSortDirection] = useState<SortDirection>(() => {
    const prefs = readPreferences();
    return isSortDirection(prefs.sortDirection)
      ? prefs.sortDirection
      : DEFAULTS.sortDirection;
  });

  useEffect(() => {
    try {
      localStorage.setItem(
        STORAGE_KEY,
        JSON.stringify({
          pageSize,
          statusFilter,
          sortField,
          sortDirection,
        }),
      );
    } catch {
      // Ignore localStorage errors (e.g., in incognito mode)
    }
  }, [pageSize, statusFilter, sortField, sortDirection]);

  return {
    pageSize,
    setPageSize,
    statusFilter,
    setStatusFilter,
    sortField,
    setSortField,
    sortDirection,
    setSortDirection,
  };
}

export default useDocumentPreferences;
