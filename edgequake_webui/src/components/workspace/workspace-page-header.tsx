"use client";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import type { Workspace } from "@/types";
import { FolderKanban, RefreshCw, Settings } from "lucide-react";
import { useTranslation } from "react-i18next";

export interface WorkspacePageHeaderProps {
  workspace: Workspace;
  onRefresh: () => void;
  /** Opens Reconfigure Workspace wizard (LAW-101-12). */
  onEditStart: () => void;
}

export function WorkspacePageHeader({
  workspace,
  onRefresh,
  onEditStart,
}: WorkspacePageHeaderProps) {
  const { t } = useTranslation();

  return (
    <div className="flex flex-col gap-4 lg:flex-row lg:items-center lg:justify-between">
      <div className="space-y-1">
        <div className="flex items-center gap-3">
          <FolderKanban className="h-8 w-8 text-primary" />
          <h1 className="text-2xl font-bold">{workspace.name}</h1>
          <Badge variant={workspace.is_active ? "default" : "secondary"}>
            {workspace.is_active
              ? t("common.active", "Active")
              : t("common.inactive", "Inactive")}
          </Badge>
        </div>
        {workspace.description && (
          <p className="text-muted-foreground">{workspace.description}</p>
        )}
      </div>
      <div className="flex flex-wrap items-center gap-2 self-start lg:self-auto">
        <Button variant="outline" size="sm" onClick={onRefresh}>
          <RefreshCw className="h-4 w-4 mr-2" />
          {t("common.refresh", "Refresh")}
        </Button>
        <Button
          variant="default"
          size="sm"
          onClick={onEditStart}
          data-testid="workspace-edit-config"
        >
          <Settings className="h-4 w-4 mr-2" />
          {t("workspace.editConfig", "Edit Configuration")}
        </Button>
      </div>
    </div>
  );
}
