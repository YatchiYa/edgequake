"use client";

import { ProviderIcon } from "@/components/providers/provider-icon";
import {
  PdfParserBackendField,
  type PdfParserBackendChoice,
} from "@/components/settings/pdf-parser-backend-field";
import { Badge } from "@/components/ui/badge";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import {
  LLMModelSelector,
  type LLMSelection,
} from "@/components/workspace/llm-model-selector";
import { useInheritedModelDefaults } from "@/hooks/use-inherited-model-defaults";
import { useTenantStore } from "@/stores/use-tenant-store";
import type { Workspace } from "@/types";
import { AlertTriangle, Eye, Gauge, Sparkles } from "lucide-react";
import { useTranslation } from "react-i18next";

export interface WorkspaceExtendedModelConfigProps {
  workspace: Workspace;
  isEditing: boolean;
  selectedVisionLLM: LLMSelection | undefined;
  selectedPdfParserBackend: PdfParserBackendChoice;
  onVisionLlmChange: (value: LLMSelection | undefined) => void;
  onPdfParserBackendChange: (value: PdfParserBackendChoice) => void;
  visionLLMChanged: boolean;
}

/** Vision LLM + PDF parser cards (dashboard workspace route only, SPEC-040). */
export function WorkspaceExtendedModelConfig({
  workspace,
  isEditing,
  selectedVisionLLM,
  selectedPdfParserBackend,
  onVisionLlmChange,
  onPdfParserBackendChange,
  visionLLMChanged,
}: WorkspaceExtendedModelConfigProps) {
  const { t } = useTranslation();
  const tenantId = useTenantStore((s) => s.selectedTenantId);
  const inherited = useInheritedModelDefaults(tenantId);
  const visionDefaultId =
    inherited.defaultVisionProvider && inherited.defaultVisionModel
      ? `${inherited.defaultVisionProvider}/${inherited.defaultVisionModel}`
      : undefined;

  return (
    <div
      className="grid grid-cols-1 lg:grid-cols-2 gap-6 items-stretch"
      data-testid="workspace-vision-parser-combo"
    >
      <Card className="h-full flex flex-col">
        <CardHeader className="space-y-2">
          <CardTitle className="flex items-center gap-2">
            <Sparkles className="h-5 w-5 text-orange-600 shrink-0" />
            {t("workspace.visionLlmConfig", "Vision LLM (PDF Extraction)")}
          </CardTitle>
          <CardDescription className="min-h-[2.75rem]">
            {t(
              "workspace.visionLlmConfigDesc",
              "Multimodal model used for PDF page rendering and text extraction. Overrides server default for this workspace.",
            )}
          </CardDescription>
        </CardHeader>
        <CardContent className="flex flex-1 flex-col gap-4">
          <div className="mt-auto space-y-4">
            {isEditing ? (
              <>
                <LLMModelSelector
                  value={selectedVisionLLM}
                  onChange={onVisionLlmChange}
                  showUsageHint
                />
                {visionLLMChanged && (
                  <div className="flex items-center gap-2 p-3 bg-orange-50 dark:bg-orange-900/20 border border-orange-200 dark:border-orange-800 rounded-lg">
                    <AlertTriangle className="h-4 w-4 text-orange-600 shrink-0" />
                    <span className="text-sm text-orange-700 dark:text-orange-300">
                      {t(
                        "workspace.visionLlmChangeWarning",
                        "New Vision LLM will be used for all subsequent PDF uploads.",
                      )}
                    </span>
                  </div>
                )}
              </>
            ) : (
              <div className="flex items-center gap-3 p-3 bg-muted/50 rounded-lg min-h-[3.75rem]">
                <ProviderIcon
                  providerId={
                    workspace.vision_llm_provider ||
                    visionDefaultId?.split("/")[0]
                  }
                />
                <div className="min-w-0 flex-1">
                  <div className="font-medium truncate">
                    {workspace.vision_llm_model ||
                      t(
                        "workspace.serverDefaultWithValue",
                        "Server Default ({{value}})",
                        {
                          value:
                            visionDefaultId ||
                            t("workspace.notConfigured", "not configured"),
                        },
                      )}
                  </div>
                  <div className="text-sm text-muted-foreground capitalize truncate">
                    {workspace.vision_llm_provider ||
                      (visionDefaultId
                        ? t("workspace.inheritedDefault", "Inherited default")
                        : t("workspace.autoDetect", "Auto-detected"))}
                  </div>
                </div>
                {(workspace.vision_llm_provider && workspace.vision_llm_model) ||
                visionDefaultId ? (
                  <Badge
                    variant="outline"
                    className="ml-auto font-mono text-xs shrink-0 max-w-[40%] truncate"
                  >
                    {workspace.vision_llm_provider && workspace.vision_llm_model
                      ? `${workspace.vision_llm_provider}/${workspace.vision_llm_model}`
                      : visionDefaultId}
                  </Badge>
                ) : null}
              </div>
            )}
          </div>
        </CardContent>
      </Card>

      <Card className="h-full flex flex-col">
        <CardHeader className="space-y-2">
          <CardTitle className="flex items-center gap-2">
            {selectedPdfParserBackend === "vision" ? (
              <Eye className="h-5 w-5 text-amber-600 shrink-0" />
            ) : (
              <Gauge className="h-5 w-5 text-amber-600 shrink-0" />
            )}
            {t("workspace.pdfParserConfig", "PDF Parser")}
          </CardTitle>
          <CardDescription className="min-h-[2.75rem]">
            {t(
              "workspace.pdfParserConfigDesc",
              "Choose the default parser for new PDF uploads in this workspace. EdgeParse is best for digital PDFs; Vision is better for scanned or image-heavy files.",
            )}
          </CardDescription>
        </CardHeader>
        <CardContent className="flex flex-1 flex-col gap-4">
          <div className="mt-auto space-y-4">
            <PdfParserBackendField
              value={selectedPdfParserBackend}
              isEditing={isEditing}
              onChange={onPdfParserBackendChange}
            />
            {isEditing && (
              <div className="flex items-center gap-2 p-3 bg-amber-50 dark:bg-amber-900/20 border border-amber-200 dark:border-amber-800 rounded-lg">
                <AlertTriangle className="h-4 w-4 text-amber-600 shrink-0" />
                <span className="text-sm text-amber-700 dark:text-amber-300">
                  {t(
                    "workspace.pdfParserChangeWarning",
                    "This default applies to subsequent PDF uploads. Existing documents keep their original extraction method unless reprocessed.",
                  )}
                </span>
              </div>
            )}
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
