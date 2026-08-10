"use client";

import { CheckCircle } from "lucide-react";
import { useTranslation } from "react-i18next";

export function WorkspaceStatusFooter() {
  const { t } = useTranslation();

  return (
    <div className="flex items-center justify-center gap-1.5 py-1 text-xs text-muted-foreground">
      <CheckCircle className="h-3.5 w-3.5 text-green-500" />
      {t(
        "workspace.statusReady",
        "Workspace ready for queries and document ingestion",
      )}
    </div>
  );
}
