/**
 * @module WorkspacePage
 * @description Current workspace detail page showing configuration, stats, and actions.
 *
 * @implements SPEC-032: Workspace configuration display
 * @implements SPEC-101 LAW-101-12: Reconfigure via guided wizard (read-only overview)
 * @implements FEAT0801: Workspace detail view with LLM/embedding configuration
 * @implements UC0305: User views workspace configuration
 *
 * @enforces BR0305: Workspace config is visible and editable
 * @enforces BR0306: Rebuild action available when model changes
 */
'use client';

<<<<<<< HEAD
import {
  type PdfParserBackendChoice,
} from '@/components/settings/pdf-parser-backend-field';
=======
import { ReconfigureWorkspaceWizard } from '@/components/onboarding/reconfigure-workspace-wizard';
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/components/ui/alert-dialog';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Separator } from '@/components/ui/separator';
import { Skeleton } from '@/components/ui/skeleton';
<<<<<<< HEAD
import type { EmbeddingSelection } from '@/components/workspace/embedding-model-selector';
import type { LLMSelection } from '@/components/workspace/llm-model-selector';
=======
import { ProviderStatusHub } from '@/components/settings/provider-status-hub';
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
import { WorkspaceActionsCard } from '@/components/workspace/workspace-actions-card';
import { WorkspaceExtendedModelConfig } from '@/components/workspace/workspace-extended-model-config';
import { WorkspaceModelConfigGrid } from '@/components/workspace/workspace-model-config-grid';
import { WorkspaceStatusFooter } from '@/components/workspace/workspace-status-footer';
import { WorkspaceEntityTypesCard } from '@/components/workspace/workspace-entity-types-card';
<<<<<<< HEAD
import { WorkspacePageHeader } from '@/components/workspace/workspace-page-header';
import { ProviderStatusHub } from '@/components/settings/provider-status-hub';
=======
import { WorkspaceExtractionLanguageCard } from '@/components/workspace/workspace-extraction-language-card';
import { WorkspacePageHeader } from '@/components/workspace/workspace-page-header';
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
import { WorkspaceStatsCards } from '@/components/workspace/workspace-stats-cards';
import { ENTITY_PRESETS } from '@/constants/entity-presets';
import { refreshDynamicModels } from '@/hooks/use-providers';
import { useWorkspaceDetailQueries } from '@/hooks/use-workspace-detail-queries';
import { useWorkspaceTenantValidator } from '@/hooks/use-workspace-tenant-validator';
<<<<<<< HEAD
import { deleteWorkspace, updateWorkspace } from '@/lib/api/edgequake';
import {
  getWorkspaceEmbeddingSelection,
  getWorkspaceLlmSelection,
  getWorkspacePdfParserBackend,
  getWorkspaceVisionSelection,
} from '@/lib/workspace/drafts';
import { useTenantStore } from '@/stores/use-tenant-store';
import { useMutation, useQueryClient } from '@tanstack/react-query';
=======
import { deleteWorkspace } from '@/lib/api/edgequake';
import type { WorkspaceRebuildHints } from '@/lib/onboarding/workspace-config-diff';
import {
  getWorkspacePdfParserBackend,
} from '@/lib/workspace/drafts';
import { useTenantStore } from '@/stores/use-tenant-store';
import { useQueryClient } from '@tanstack/react-query';
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
import {
  AlertTriangle,
  FolderKanban,
  RefreshCw,
  Trash2,
} from 'lucide-react';
import { useRouter } from 'next/navigation';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { toast } from 'sonner';

<<<<<<< HEAD

=======
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
export default function WorkspacePage() {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const router = useRouter();
  const { selectedTenantId, selectedWorkspaceId, selectWorkspace } = useTenantStore();

  useWorkspaceTenantValidator({
    onValidationFailed: (result) => {
      console.error('[Workspace] Workspace-tenant mismatch detected:', result.reason);
      toast.error('Workspace context corrected', {
        description: 'Your workspace selection was updated to match the current tenant.',
      });
    },
  });

<<<<<<< HEAD
  // Edit mode state
  const [isEditing, setIsEditing] = useState(false);
  const [selectedLLM, setSelectedLLM] = useState<LLMSelection | undefined>(undefined);
  const [selectedEmbedding, setSelectedEmbedding] = useState<EmbeddingSelection | undefined>(undefined);
  const [selectedVisionLLM, setSelectedVisionLLM] = useState<LLMSelection | undefined>(undefined);
  const [selectedPdfParserBackend, setSelectedPdfParserBackend] =
    useState<PdfParserBackendChoice>('none');
  const [selectedEntityTypes, setSelectedEntityTypes] = useState<string[]>([
    ...ENTITY_PRESETS.general.types,
  ]);
  const [selectedEntityTypesStrict, setSelectedEntityTypesStrict] = useState(true);
  // FIX #171: Delete workspace state
  const [showDeleteConfirm, setShowDeleteConfirm] = useState(false);
  const [isDeleting, setIsDeleting] = useState(false);

  // FIX #171: Delete workspace handler
=======
  const [reconfigureOpen, setReconfigureOpen] = useState(false);
  const [showDeleteConfirm, setShowDeleteConfirm] = useState(false);
  const [isDeleting, setIsDeleting] = useState(false);
  const [pendingRebuild, setPendingRebuild] = useState<WorkspaceRebuildHints | null>(null);

>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
  const handleDeleteWorkspace = async () => {
    if (!selectedWorkspaceId) return;
    setIsDeleting(true);
    try {
      await deleteWorkspace(selectedWorkspaceId);
      selectWorkspace(null);
      queryClient.invalidateQueries({ queryKey: ['workspaces'] });
      toast.success(t('workspace.deleted', 'Workspace deleted'));
      router.push('/');
    } catch (err) {
<<<<<<< HEAD
      toast.error(`Failed to delete workspace: ${err instanceof Error ? err.message : 'Unknown error'}`);
=======
      toast.error(
        `Failed to delete workspace: ${err instanceof Error ? err.message : 'Unknown error'}`,
      );
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
    } finally {
      setIsDeleting(false);
      setShowDeleteConfirm(false);
    }
<<<<<<< HEAD
  };

  const {
    workspace,
    stats,
    providerHealth,
    isLoadingWorkspace,
    isLoadingStats,
    isLoadingHealth,
    refetchWorkspace,
  } = useWorkspaceDetailQueries(selectedTenantId, selectedWorkspaceId);

  // Update workspace mutation
  const updateMutation = useMutation({
    mutationFn: (data: {
      llm_model?: string;
      llm_provider?: string;
      embedding_model?: string;
      embedding_provider?: string;
      embedding_dimension?: number;
      vision_llm_provider?: string;
      vision_llm_model?: string;
      pdf_parser_backend?: PdfParserBackendChoice;
      entity_types?: string[];
      entity_types_strict?: boolean;
      _embeddingChanged?: boolean;
      _llmChanged?: boolean;
      _visionChanged?: boolean;
    }) =>
      updateWorkspace(selectedTenantId!, selectedWorkspaceId!, {
        llm_model: data.llm_model,
        llm_provider: data.llm_provider,
        embedding_model: data.embedding_model,
        embedding_provider: data.embedding_provider,
        embedding_dimension: data.embedding_dimension,
        vision_llm_provider: data.vision_llm_provider,
        vision_llm_model: data.vision_llm_model,
        pdf_parser_backend: data.pdf_parser_backend,
        entity_types: data.entity_types,
        entity_types_strict: data.entity_types_strict,
      }),
    onSuccess: (_result, variables) => {
      toast.success(t('workspace.updateSuccess', 'Workspace updated successfully'));
      queryClient.invalidateQueries({ queryKey: ['workspace', selectedTenantId, selectedWorkspaceId] });
      setIsEditing(false);
      
      // Check if model changes require rebuild
      const needsEmbeddingRebuild = variables._embeddingChanged;
      const needsExtractionRebuild = variables._llmChanged;
      const needsVisionRebuild = variables._visionChanged;
      
      if (needsEmbeddingRebuild || needsExtractionRebuild || needsVisionRebuild) {
        setPendingRebuild({
          embeddings: needsEmbeddingRebuild ?? false,
          extraction: needsExtractionRebuild ?? false,
          vision: needsVisionRebuild ?? false,
        });
        
        if (needsEmbeddingRebuild && needsExtractionRebuild) {
          toast.info(
            t('workspace.rebuildRequired', 'Model changes detected'),
            {
              description: t(
                'workspace.rebuildBothHint',
                'Both embedding and LLM models changed. Use "Rebuild Embeddings" to reprocess all documents.'
              ),
              duration: 8000,
            }
          );
        } else if (needsEmbeddingRebuild) {
          toast.info(
            t('workspace.embeddingRebuildRequired', 'Embedding model changed'),
            {
              description: t(
                'workspace.embeddingRebuildHint',
                'Use "Rebuild Embeddings" to regenerate vector embeddings with the new model.'
              ),
              duration: 6000,
            }
          );
        } else if (needsExtractionRebuild) {
          toast.info(
            t('workspace.llmRebuildRequired', 'LLM model changed'),
            {
              description: t(
                'workspace.llmRebuildHint',
                'Use "Rebuild Knowledge Graph" to re-extract entities with the new LLM model.'
              ),
              duration: 6000,
            }
          );
        } else if (needsVisionRebuild) {
          toast.info(
            t('workspace.visionRebuildRequired', 'Vision LLM model changed'),
            {
              description: t(
                'workspace.visionRebuildHint',
                'Use "Rebuild Knowledge Graph" to re-extract PDF documents with the new vision model from original files.'
              ),
              duration: 6000,
            }
          );
        }
      }
    },
    onError: (error) => {
      toast.error(t('workspace.updateFailed', 'Failed to update workspace'), {
        description: error instanceof Error ? error.message : 'Unknown error',
      });
    },
  });

  const handleSave = () => {
    const data: Parameters<typeof updateMutation.mutate>[0] = {
      _embeddingChanged: embeddingModelChanged ?? false,
      _llmChanged: llmModelChanged ?? false,
      _visionChanged: visionLLMChanged ?? false,
      entity_types: selectedEntityTypes,
      entity_types_strict: selectedEntityTypesStrict,
    };

    // SPEC-013: empty strings clear workspace override → server/env defaults (same as vision)
    data.llm_model = selectedLLM?.model ?? '';
    data.llm_provider = selectedLLM?.provider ?? '';

    if (selectedEmbedding) {
      data.embedding_model = selectedEmbedding.model;
      data.embedding_provider = selectedEmbedding.provider;
      data.embedding_dimension = selectedEmbedding.dimension;
    } else {
      data.embedding_model = '';
      data.embedding_provider = '';
      data.embedding_dimension = 0;
    }

    // Vision LLM config (SPEC-040: empty string clears workspace override)
    data.vision_llm_provider = selectedVisionLLM?.provider ?? '';
    data.vision_llm_model = selectedVisionLLM?.model ?? '';
    data.pdf_parser_backend = selectedPdfParserBackend;
    updateMutation.mutate(data);
  };

  const handleCancel = () => {
    setIsEditing(false);
    setSelectedLLM(getWorkspaceLlmSelection(workspace));
    setSelectedEmbedding(getWorkspaceEmbeddingSelection(workspace));
    setSelectedVisionLLM(getWorkspaceVisionSelection(workspace));
    setSelectedPdfParserBackend(getWorkspacePdfParserBackend(workspace));
    setSelectedEntityTypes(
      workspace?.entity_types?.length
        ? [...workspace.entity_types]
        : [...ENTITY_PRESETS.general.types]
    );
    setSelectedEntityTypesStrict(workspace?.entity_types_strict ?? true);
  };

  const handleEditStart = () => {
    setSelectedLLM(getWorkspaceLlmSelection(workspace));
    setSelectedEmbedding(getWorkspaceEmbeddingSelection(workspace));
    setSelectedVisionLLM(getWorkspaceVisionSelection(workspace));
    setSelectedPdfParserBackend(getWorkspacePdfParserBackend(workspace));
    setSelectedEntityTypes(
      workspace?.entity_types?.length
        ? [...workspace.entity_types]
        : [...ENTITY_PRESETS.general.types]
    );
    setSelectedEntityTypesStrict(workspace?.entity_types_strict ?? true);
    setIsEditing(true);
  };

  // Check if embedding model changed (needs rebuild)
  const embeddingModelChanged = Boolean(
    workspace && (
      selectedEmbedding
        ? workspace.embedding_model !== selectedEmbedding.model ||
          workspace.embedding_provider !== selectedEmbedding.provider
        : Boolean(workspace.embedding_provider || workspace.embedding_model)
    )
  );

  // Check if LLM model changed (needs extraction rebuild)
  const llmModelChanged = Boolean(
    workspace && (
      selectedLLM
        ? workspace.llm_model !== selectedLLM.model ||
          workspace.llm_provider !== selectedLLM.provider
        : Boolean(workspace.llm_provider || workspace.llm_model)
    )
  );

  // Check if Vision LLM changed (triggers full re-extraction of existing PDF documents from originals)
  const visionLLMChanged = Boolean(
    workspace && selectedVisionLLM && (
      workspace.vision_llm_model !== selectedVisionLLM.model ||
      workspace.vision_llm_provider !== selectedVisionLLM.provider
    )
  );

  // Track if rebuild is needed after save
  const [pendingRebuild, setPendingRebuild] = useState<{
    embeddings: boolean;
    extraction: boolean;
    vision: boolean;
  } | null>(null);
=======
  };

  const {
    workspace,
    stats,
    providerHealth,
    isLoadingWorkspace,
    isLoadingStats,
    isLoadingHealth,
    refetchWorkspace,
  } = useWorkspaceDetailQueries(selectedTenantId, selectedWorkspaceId);

  const documentCount = stats?.document_count ?? workspace?.document_count ?? 0;

  const handleReconfigureApplied = (result: {
    pendingRebuild: WorkspaceRebuildHints | null;
    extractionLanguageChanged: boolean;
  }) => {
    toast.success(t('workspace.updateSuccess', 'Workspace updated successfully'));
    queryClient.invalidateQueries({
      queryKey: ['workspace', selectedTenantId, selectedWorkspaceId],
    });
    if (result.extractionLanguageChanged) {
      toast.info(
        t(
          'workspace.extractionLanguage.changedToast',
          'Extraction language updated. Reprocess documents to refresh the graph.',
        ),
        { duration: 6000 },
      );
    }
    if (result.pendingRebuild) {
      setPendingRebuild(result.pendingRebuild);
      const { embeddings, extraction, vision } = result.pendingRebuild;
      if (embeddings && extraction) {
        toast.info(t('workspace.rebuildRequired', 'Model changes detected'), {
          description: t(
            'workspace.rebuildBothHint',
            'Both embedding and LLM models changed. Use "Rebuild Embeddings" to reprocess all documents.',
          ),
          duration: 8000,
        });
      } else if (embeddings) {
        toast.info(t('workspace.embeddingRebuildRequired', 'Embedding model changed'), {
          description: t(
            'workspace.embeddingRebuildHint',
            'Use "Rebuild Embeddings" to regenerate vector embeddings with the new model.',
          ),
          duration: 6000,
        });
      } else if (extraction) {
        toast.info(t('workspace.llmRebuildRequired', 'LLM model changed'), {
          description: t(
            'workspace.llmRebuildHint',
            'Use "Rebuild Knowledge Graph" to re-extract entities with the new LLM model.',
          ),
          duration: 6000,
        });
      } else if (vision) {
        toast.info(t('workspace.visionRebuildRequired', 'Vision LLM model changed'), {
          description: t(
            'workspace.visionRebuildHint',
            'Use "Rebuild Knowledge Graph" to re-extract PDF documents with the new vision model from original files.',
          ),
          duration: 6000,
        });
      }
    }
  };
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042

  if (!selectedTenantId || !selectedWorkspaceId) {
    return (
      <ScrollArea className="h-full">
        <div className="container mx-auto p-6">
          <Card>
            <CardContent className="flex flex-col items-center justify-center py-12">
              <FolderKanban className="h-12 w-12 text-muted-foreground mb-4" />
              <h2 className="text-lg font-medium text-muted-foreground">
                {t('workspace.noWorkspaceSelected', 'No Workspace Selected')}
              </h2>
              <p className="text-sm text-muted-foreground mt-2">
                {t(
                  'workspace.selectWorkspaceHint',
                  'Please select a workspace from the sidebar.',
                )}
              </p>
            </CardContent>
          </Card>
        </div>
      </ScrollArea>
    );
  }

  if (isLoadingWorkspace) {
    return (
      <ScrollArea className="h-full">
<<<<<<< HEAD
        <div className="container mx-auto p-6 space-y-6">
          <Skeleton className="h-8 w-64" />
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
=======
        <div
          className="container mx-auto space-y-6 p-6"
          data-testid="spec100-workspace-skeleton"
        >
          <Skeleton className="h-10 w-72" />
          <div className="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-4">
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
            {[...Array(4)].map((_, i) => (
              <Skeleton key={i} className="h-32" />
            ))}
          </div>
          <Skeleton className="min-h-[16rem] w-full" />
          <Skeleton className="min-h-[12rem] w-full" />
        </div>
      </ScrollArea>
    );
  }

  if (!workspace) {
    return (
      <ScrollArea className="h-full">
        <div className="container mx-auto p-6">
          <Card>
            <CardContent className="flex flex-col items-center justify-center py-12">
              <AlertTriangle className="h-12 w-12 text-destructive mb-4" />
              <h2 className="text-lg font-medium">
                {t('workspace.notFound', 'Workspace Not Found')}
              </h2>
              <p className="text-sm text-muted-foreground mt-2 mb-4">
                {t(
                  'workspace.notFoundHint',
                  'The selected workspace could not be loaded.',
                )}
              </p>
              <Button variant="outline" onClick={() => refetchWorkspace()}>
                <RefreshCw className="h-4 w-4 mr-2" />
                {t('common.retry', 'Retry')}
              </Button>
            </CardContent>
          </Card>
        </div>
      </ScrollArea>
    );
  }

  const entityTypes = workspace.entity_types?.length
    ? workspace.entity_types
    : [...ENTITY_PRESETS.general.types];

  return (
    <ScrollArea className="h-full">
      <div className="container mx-auto p-6 space-y-6">
        <WorkspacePageHeader
          workspace={workspace}
<<<<<<< HEAD
          isEditing={isEditing}
          isSaving={updateMutation.isPending}
          onRefresh={() => refetchWorkspace()}
          onEditStart={handleEditStart}
          onCancel={handleCancel}
          onSave={handleSave}
        />

      <Separator />

      <WorkspaceStatsCards
        workspace={workspace}
        stats={stats}
        isLoadingStats={isLoadingStats}
      />

      <WorkspaceModelConfigGrid
        workspace={workspace}
        isEditing={isEditing}
        selectedLLM={selectedLLM}
        selectedEmbedding={selectedEmbedding}
        onLlmChange={setSelectedLLM}
        onEmbeddingChange={setSelectedEmbedding}
        llmModelChanged={llmModelChanged ?? false}
        embeddingModelChanged={embeddingModelChanged ?? false}
      />

      <WorkspaceExtendedModelConfig
        workspace={workspace}
        isEditing={isEditing}
        selectedVisionLLM={selectedVisionLLM}
        selectedPdfParserBackend={selectedPdfParserBackend}
        onVisionLlmChange={setSelectedVisionLLM}
        onPdfParserBackendChange={setSelectedPdfParserBackend}
        visionLLMChanged={visionLLMChanged ?? false}
      />

      <WorkspaceEntityTypesCard
        isEditing={isEditing}
        workspace={workspace}
        selectedTypes={selectedEntityTypes}
        onTypesChange={setSelectedEntityTypes}
        strictLimit={selectedEntityTypesStrict}
        onStrictLimitChange={setSelectedEntityTypesStrict}
      />

      <ProviderStatusHub
        providers={providerHealth}
        isLoading={isLoadingHealth}
        onRefresh={() => {
          void refreshDynamicModels(queryClient);
        }}
      />

      <WorkspaceActionsCard
        workspace={workspace}
        pendingRebuild={pendingRebuild}
        includeVisionPending
        onRebuildComplete={() => {
          queryClient.invalidateQueries({ queryKey: ['workspaceStats', selectedWorkspaceId] });
          queryClient.invalidateQueries({ queryKey: ['documents'] });
          setPendingRebuild(null);
        }}
      />

      <WorkspaceStatusFooter />

        {/* FIX #171: Danger Zone — Delete Workspace */}
=======
          onRefresh={() => refetchWorkspace()}
          onEditStart={() => setReconfigureOpen(true)}
        />

        <Separator />

        <WorkspaceStatsCards
          workspace={workspace}
          stats={stats}
          isLoadingStats={isLoadingStats}
        />

        <WorkspaceModelConfigGrid
          workspace={workspace}
          isEditing={false}
          selectedLLM={undefined}
          selectedEmbedding={undefined}
          onLlmChange={() => {}}
          onEmbeddingChange={() => {}}
          llmModelChanged={false}
          embeddingModelChanged={false}
        />

        <WorkspaceExtendedModelConfig
          workspace={workspace}
          isEditing={false}
          selectedVisionLLM={undefined}
          selectedPdfParserBackend={getWorkspacePdfParserBackend(workspace)}
          onVisionLlmChange={() => {}}
          onPdfParserBackendChange={() => {}}
          visionLLMChanged={false}
        />

        <WorkspaceExtractionLanguageCard
          isEditing={false}
          workspace={workspace}
          selectedLanguage={workspace.extraction_language ?? null}
          onLanguageChange={() => {}}
        />

        <WorkspaceEntityTypesCard
          isEditing={false}
          workspace={workspace}
          selectedTypes={entityTypes}
          onTypesChange={() => {}}
          strictLimit={workspace.entity_types_strict ?? true}
          onStrictLimitChange={() => {}}
          extractionLanguage={workspace.extraction_language ?? null}
        />

        <ProviderStatusHub
          providers={providerHealth}
          isLoading={isLoadingHealth}
          onRefresh={() => {
            void refreshDynamicModels(queryClient);
          }}
        />

        <WorkspaceActionsCard
          workspace={workspace}
          pendingRebuild={pendingRebuild}
          includeVisionPending
          onRebuildComplete={() => {
            queryClient.invalidateQueries({
              queryKey: ['workspaceStats', selectedWorkspaceId],
            });
            queryClient.invalidateQueries({ queryKey: ['documents'] });
            setPendingRebuild(null);
          }}
        />

        <WorkspaceStatusFooter />

>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
        <Card className="border-destructive/50">
          <CardHeader>
            <CardTitle className="flex items-center gap-2 text-destructive">
              <Trash2 className="h-5 w-5" />
              {t('workspace.dangerZone', 'Danger Zone')}
            </CardTitle>
            <CardDescription>
<<<<<<< HEAD
              {t('workspace.deleteWarning', 'Deleting a workspace permanently removes all documents, entities, relationships, and embeddings. This action cannot be undone.')}
=======
              {t(
                'workspace.deleteWarning',
                'Deleting a workspace permanently removes all documents, entities, relationships, and embeddings. This action cannot be undone.',
              )}
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
            </CardDescription>
          </CardHeader>
          <CardContent>
            <Button
              variant="destructive"
              className="w-full sm:w-auto"
<<<<<<< HEAD
              aria-label={t('workspace.deleteButtonAria', 'Delete workspace {{name}}', { name: workspace.name })}
=======
              aria-label={t('workspace.deleteButtonAria', 'Delete workspace {{name}}', {
                name: workspace.name,
              })}
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
              onClick={() => setShowDeleteConfirm(true)}
            >
              <Trash2 className="h-4 w-4 mr-2" />
              {t('workspace.deleteButton', 'Delete this workspace')}
            </Button>
          </CardContent>
        </Card>
      </div>

<<<<<<< HEAD
      {/* Delete Workspace Confirmation */}
      <AlertDialog open={showDeleteConfirm} onOpenChange={setShowDeleteConfirm}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{t('workspace.deleteConfirmTitle', 'Delete Workspace')}</AlertDialogTitle>
            <AlertDialogDescription>
              {t('workspace.deleteConfirmDesc', 'Are you sure you want to delete workspace "{name}"? This will permanently remove all documents, entities, relationships, and embeddings.', { name: workspace?.name || '' })}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel autoFocus disabled={isDeleting}>{t('common.cancel', 'Cancel')}</AlertDialogCancel>
=======
      <ReconfigureWorkspaceWizard
        open={reconfigureOpen}
        onOpenChange={setReconfigureOpen}
        tenantId={selectedTenantId}
        workspace={workspace}
        documentCount={documentCount}
        onApplied={handleReconfigureApplied}
      />

      <AlertDialog open={showDeleteConfirm} onOpenChange={setShowDeleteConfirm}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>
              {t('workspace.deleteConfirmTitle', 'Delete Workspace')}
            </AlertDialogTitle>
            <AlertDialogDescription>
              {t(
                'workspace.deleteConfirmDesc',
                'Are you sure you want to delete workspace "{name}"? This will permanently remove all documents, entities, relationships, and embeddings.',
                { name: workspace?.name || '' },
              )}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel autoFocus disabled={isDeleting}>
              {t('common.cancel', 'Cancel')}
            </AlertDialogCancel>
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
            <AlertDialogAction
              onClick={handleDeleteWorkspace}
              disabled={isDeleting}
              className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
            >
<<<<<<< HEAD
              {isDeleting ? t('workspace.deleting', 'Deleting...') : t('workspace.deleteConfirmButton', 'Delete')}
=======
              {isDeleting
                ? t('workspace.deleting', 'Deleting...')
                : t('workspace.deleteConfirmButton', 'Delete')}
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </ScrollArea>
  );
}
