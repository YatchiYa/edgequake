/**
 * SPEC-099 LAW-099-9 / ISP — row actions without prop-drilling the Manager bag.
 */
"use client";

import type { Document } from "@/types";
import { createContext, useContext, type ReactNode } from "react";

export interface DocumentsActions {
  onClick: (doc: Document) => void;
  onDoubleClick: (doc: Document) => void;
  onSelect: (id: string, selected: boolean) => void;
  onReprocess: (doc: Document) => void;
  onRetry: (doc: Document) => void;
  onDelete: (doc: Document) => void;
  onViewDetails: (doc: Document) => void;
  onViewInGraph: (doc: Document) => void;
  onViewPdf: (doc: Document) => void;
}

const DocumentsActionsContext = createContext<DocumentsActions | null>(null);

export function DocumentsActionsProvider({
  value,
  children,
}: {
  value: DocumentsActions;
  children: ReactNode;
}) {
  return (
    <DocumentsActionsContext.Provider value={value}>
      {children}
    </DocumentsActionsContext.Provider>
  );
}

export function useDocumentsActions(): DocumentsActions {
  const ctx = useContext(DocumentsActionsContext);
  if (!ctx) {
    throw new Error("useDocumentsActions requires DocumentsActionsProvider");
  }
  return ctx;
}

/** Optional variant for components that may render outside the shell. */
export function useDocumentsActionsOptional(): DocumentsActions | null {
  return useContext(DocumentsActionsContext);
}
