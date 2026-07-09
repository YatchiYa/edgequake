/**
 * @module DocumentDownloadMenu
 * @description Dropdown for downloading document originals and extracted markdown.
 *
 * @implements SPEC-002 - Document Viewer download actions
 */
"use client";

import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSub,
  DropdownMenuSubContent,
  DropdownMenuSubTrigger,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import {
  canDownloadMarkdown,
  canDownloadOriginal,
  downloadDocumentMarkdown,
  downloadDocumentOriginal,
} from "@/lib/document-download";
import type { Document } from "@/types";
import { Download, FileImage, FileText } from "lucide-react";
import { useCallback } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";

interface DocumentDownloadMenuProps {
  document: Document;
  /** Override markdown content (e.g. PDF viewer dialog). */
  markdownContent?: string | null;
  /** Standalone dropdown button or submenu items for embedding. */
  variant?: "button" | "icon" | "submenu";
  className?: string;
}

function DisabledItem({
  label,
  tooltip,
}: {
  label: string;
  tooltip: string;
}) {
  return (
    <TooltipProvider>
      <Tooltip>
        <TooltipTrigger asChild>
          <span className="w-full">
            <DropdownMenuItem disabled className="opacity-50">
              {label}
            </DropdownMenuItem>
          </span>
        </TooltipTrigger>
        <TooltipContent side="left">{tooltip}</TooltipContent>
      </Tooltip>
    </TooltipProvider>
  );
}

function useDownloadHandlers(
  document: Document,
  markdownContent?: string | null,
) {
  const { t } = useTranslation();

  const handleOriginal = useCallback(() => {
    try {
      downloadDocumentOriginal(document);
      toast.success(t("documents.download.originalStarted", "Original download started"));
    } catch {
      toast.error(t("documents.download.originalFailed", "Failed to download original"));
    }
  }, [document, t]);

  const handleMarkdown = useCallback(() => {
    try {
      downloadDocumentMarkdown(document, markdownContent);
      toast.success(t("documents.download.markdownStarted", "Markdown download started"));
    } catch {
      toast.error(t("documents.download.markdownFailed", "Failed to download markdown"));
    }
  }, [document, markdownContent, t]);

  return { handleOriginal, handleMarkdown };
}

function DownloadMenuItems({
  document,
  markdownContent,
}: Pick<DocumentDownloadMenuProps, "document" | "markdownContent">) {
  const { t } = useTranslation();
  const { handleOriginal, handleMarkdown } = useDownloadHandlers(document, markdownContent);

  const showOriginal = canDownloadOriginal(document);
  const showMarkdown = canDownloadMarkdown(document, markdownContent);

  return (
    <>
      {showOriginal ? (
        <DropdownMenuItem onClick={handleOriginal}>
          <FileImage className="h-4 w-4 mr-2" />
          {t("documents.download.original", "Download original")}
        </DropdownMenuItem>
      ) : (
        <DisabledItem
          label={t("documents.download.original", "Download original")}
          tooltip={t(
            "documents.download.originalUnavailable",
            "Original file is not stored for this document type",
          )}
        />
      )}
      {showMarkdown ? (
        <DropdownMenuItem onClick={handleMarkdown}>
          <FileText className="h-4 w-4 mr-2" />
          {t("documents.download.markdown", "Download markdown")}
        </DropdownMenuItem>
      ) : (
        <DisabledItem
          label={t("documents.download.markdown", "Download markdown")}
          tooltip={t(
            "documents.download.markdownUnavailable",
            "Markdown content is not available yet",
          )}
        />
      )}
    </>
  );
}

export function DocumentDownloadMenu({
  document,
  markdownContent,
  variant = "button",
  className,
}: DocumentDownloadMenuProps) {
  const { t } = useTranslation();

  if (variant === "submenu") {
    return (
      <DropdownMenuSub>
        <DropdownMenuSubTrigger>
          <Download className="h-4 w-4 mr-2" />
          {t("documents.download.title", "Download")}
        </DropdownMenuSubTrigger>
        <DropdownMenuSubContent>
          <DownloadMenuItems document={document} markdownContent={markdownContent} />
        </DropdownMenuSubContent>
      </DropdownMenuSub>
    );
  }

  const trigger =
    variant === "icon" ? (
      <Button
        variant="ghost"
        size="sm"
        className={className ?? "h-8 w-8 p-0"}
        aria-label={t("documents.download.title", "Download")}
      >
        <Download className="h-3.5 w-3.5" />
      </Button>
    ) : (
      <Button variant="outline" size="sm" className={className ?? "h-8"}>
        <Download className="h-4 w-4 mr-1.5" />
        {t("documents.download.title", "Download")}
      </Button>
    );

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>{trigger}</DropdownMenuTrigger>
      <DropdownMenuContent align="end">
        <DownloadMenuItems document={document} markdownContent={markdownContent} />
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
