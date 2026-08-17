'use client';

<<<<<<< HEAD
import { ModelSelector } from '@/components/models/model-selector';
import { EntityTypeSelector } from '@/components/shared/entity-type-selector';
=======
import { CreateTenantWizard } from '@/components/onboarding/create-tenant-wizard';
import { CreateWorkspaceWizard } from '@/components/onboarding/create-workspace-wizard';
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { getTenants, getWorkspaces } from '@/lib/api/edgequake';
import {
<<<<<<< HEAD
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { ENTITY_PRESETS } from '@/constants/entity-presets';
import { createTenant, createWorkspace, getTenants, getWorkspaces } from '@/lib/api/edgequake';
import { useTenantStore } from '@/stores/use-tenant-store';
import type { Tenant } from '@/types';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
=======
  applyCreatedTenantContext,
  applyCreatedWorkspaceContext,
} from '@/lib/onboarding/apply-created-workspace-context';
import { useTenantStore } from '@/stores/use-tenant-store';
import { useQuery, useQueryClient } from '@tanstack/react-query';
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
import { AlertTriangle, Building2, FolderKanban, Loader2, Plus } from 'lucide-react';
import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { toast } from 'sonner';

interface TenantGuardProps {
  children: React.ReactNode;
}

/**
 * Ensures a tenant and workspace are selected.
 * SPEC-101: empty states open shared create wizards (LAW-101-1).
 */
export function TenantGuard({ children }: TenantGuardProps) {
  const { t } = useTranslation();
  const queryClient = useQueryClient();

  const {
    selectedTenantId,
    selectedWorkspaceId,
    setTenants,
    setWorkspaces,
    selectTenant,
    selectWorkspace,
    initializeFromStorage,
  } = useTenantStore();

  const [showCreateTenant, setShowCreateTenant] = useState(false);
  const [showCreateWorkspace, setShowCreateWorkspace] = useState(false);
<<<<<<< HEAD
  const [newTenantName, setNewTenantName] = useState('EdgeQuake');
  const [newWorkspaceName, setNewWorkspaceName] = useState('Default Workspace');
  const [newWorkspaceSlug, setNewWorkspaceSlug] = useState('');
  
  // SPEC-032: Model selection states for tenant creation
  const [tenantLlmModel, setTenantLlmModel] = useState<string>();
  const [tenantEmbeddingModel, setTenantEmbeddingModel] = useState<string>();
  // SPEC-041: Default Vision LLM for PDF-to-Markdown extraction
  const [tenantVisionLlmModel, setTenantVisionLlmModel] = useState<string>();
  
  // SPEC-032: Model selection states for workspace creation
  const [workspaceLlmModel, setWorkspaceLlmModel] = useState<string>();
  const [workspaceEmbeddingModel, setWorkspaceEmbeddingModel] = useState<string>();
  // SPEC-041: Vision LLM for PDF-to-Markdown extraction (workspace-level override)
  const [workspaceVisionLlmModel, setWorkspaceVisionLlmModel] = useState<string>();
  // SPEC-085: Custom entity types for workspace
  const [workspaceEntityTypes, setWorkspaceEntityTypes] = useState<string[]>([...ENTITY_PRESETS.general.types]);
  
  // Track if we're in the middle of context setup (prevents premature children render)
  const [isSettingUpContext, setIsSettingUpContext] = useState(false);
  // After a long wait, surface the connection card instead of an indefinite spinner.
=======
  const [isSettingUpContext, setIsSettingUpContext] = useState(false);
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
  const [loadingTimedOut, setLoadingTimedOut] = useState(false);

  useEffect(() => {
    initializeFromStorage();
  }, [initializeFromStorage]);

  const {
    data: tenantsData,
    isLoading: isLoadingTenants,
    error: tenantsError,
  } = useQuery({
    queryKey: ['tenants'],
    queryFn: getTenants,
    staleTime: 60000,
  });

  const {
    data: workspacesData,
    isLoading: isLoadingWorkspaces,
    isFetching: isFetchingWorkspaces,
  } = useQuery({
    queryKey: ['workspaces', selectedTenantId],
    queryFn: () => (selectedTenantId ? getWorkspaces(selectedTenantId) : Promise.resolve([])),
    enabled: !!selectedTenantId,
    staleTime: 60000,
  });

  const isLoading =
<<<<<<< HEAD
    isLoadingTenants ||
    (!!selectedTenantId && isLoadingWorkspaces) ||
    isSettingUpContext;

  // WHY 8s: BackendStatusBanner already polls /health; a stuck TenantGuard
  // spinner during DB pressure looked like a blank /documents page.
=======
    isLoadingTenants || (!!selectedTenantId && isLoadingWorkspaces) || isSettingUpContext;

>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
  useEffect(() => {
    if (!isLoading) {
      setLoadingTimedOut(false);
      return;
    }
    const timer = window.setTimeout(() => setLoadingTimedOut(true), 8000);
    return () => window.clearTimeout(timer);
  }, [isLoading]);

<<<<<<< HEAD
  // Auto-select tenant and validate existing selection
  // WHY: Prevents stale tenant IDs from causing cascading workspace lookup failures
=======
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
  useEffect(() => {
    if (tenantsData && tenantsData.length > 0) {
      setTenants(tenantsData);
      const tenantExists =
        selectedTenantId && tenantsData.some((te) => te.id === selectedTenantId);
      if (!selectedTenantId || !tenantExists) {
        selectTenant(tenantsData[0].id);
      }
    }
  }, [tenantsData, setTenants, selectedTenantId, selectTenant]);

  useEffect(() => {
    if (!workspacesData || workspacesData.length === 0) return;

    // Merge server list with any optimistic entries (just-created workspace) so we
    // do not drop a selection that is not yet in the refetch payload.
    const storeWorkspaces = useTenantStore.getState().workspaces;
    const byId = new Map<string, (typeof workspacesData)[number]>();
    for (const w of workspacesData) byId.set(w.id, w);
    for (const w of storeWorkspaces) {
      if (!byId.has(w.id)) byId.set(w.id, w);
    }
    const merged = Array.from(byId.values());
    setWorkspaces(merged);

    const workspaceExists =
      selectedWorkspaceId && merged.some((w) => w.id === selectedWorkspaceId);

<<<<<<< HEAD
  // Generate slug from name
  const generateSlug = useCallback((name: string): string => {
    return name
      .toLowerCase()
      .replace(/[^a-z0-9\s-]/g, '')
      .replace(/\s+/g, '-')
      .replace(/-+/g, '-')
      .substring(0, 50)
      .replace(/^-|-$/g, '');
  }, []);

  /**
   * Pre-fill workspace model fields from a tenant's defaults, then open the dialog.
   * ModelSelector expects "provider:model" format (e.g., "ollama:gemma3:12b").
   */
  const handleOpenCreateWorkspace = useCallback((tenantOverride?: Tenant) => {
    const tenant = tenantOverride ?? tenantsData?.find((te) => te.id === selectedTenantId);
    if (tenant) {
      if (tenant.default_llm_model) {
        const llmVal = tenant.default_llm_provider
          ? `${tenant.default_llm_provider}:${tenant.default_llm_model}`
          : tenant.default_llm_model;
        setWorkspaceLlmModel(llmVal);
      }
      if (tenant.default_embedding_model) {
        const embVal = tenant.default_embedding_provider
          ? `${tenant.default_embedding_provider}:${tenant.default_embedding_model}`
          : tenant.default_embedding_model;
        setWorkspaceEmbeddingModel(embVal);
      }
      if (tenant.default_vision_llm_model) {
        const visionVal = tenant.default_vision_llm_provider
          ? `${tenant.default_vision_llm_provider}:${tenant.default_vision_llm_model}`
          : tenant.default_vision_llm_model;
        setWorkspaceVisionLlmModel(visionVal);
      }
    }
    setShowCreateWorkspace(true);
  }, [selectedTenantId, tenantsData]);

  // Create tenant mutation - SPEC-032: Now accepts model configuration
  const createTenantMutation = useMutation({
    mutationFn: (data: {
      name: string;
      default_llm_model?: string;
      default_llm_provider?: string;
      default_embedding_model?: string;
      default_embedding_provider?: string;
      default_vision_llm_model?: string;
      default_vision_llm_provider?: string;
    }) => createTenant(data),
  });

  // Create workspace mutation - SPEC-032/SPEC-041: Accepts LLM, embedding and vision config
  const createWorkspaceMutation = useMutation({
    mutationFn: (data: {
      name: string;
      slug?: string;
      llm_model?: string;
      llm_provider?: string;
      embedding_model?: string;
      embedding_provider?: string;
      vision_llm_model?: string;
      vision_llm_provider?: string;
    }) =>
      selectedTenantId
        ? createWorkspace(selectedTenantId, data)
        : Promise.reject(new Error('No tenant selected')),
  });

  // Handle tenant creation with proper async flow
  const handleCreateTenant = useCallback(async () => {
    if (!newTenantName.trim()) return;
    if (!tenantLlmModel || !tenantEmbeddingModel || !tenantVisionLlmModel) return;
    
    setIsSettingUpContext(true);
    try {
      // SPEC-032: Parse model selections and include in tenant creation
      const llmConfig = parseModelValue(tenantLlmModel);
      const embeddingConfig = parseModelValue(tenantEmbeddingModel);
      // SPEC-041: Parse vision LLM selection
      const visionConfig = parseModelValue(tenantVisionLlmModel);
      
      const tenantData = {
        name: newTenantName,
        ...(llmConfig.model && { default_llm_model: llmConfig.model }),
        ...(llmConfig.provider && { default_llm_provider: llmConfig.provider }),
        ...(embeddingConfig.model && { default_embedding_model: embeddingConfig.model }),
        ...(embeddingConfig.provider && { default_embedding_provider: embeddingConfig.provider }),
        ...(visionConfig.model && { default_vision_llm_model: visionConfig.model }),
        ...(visionConfig.provider && { default_vision_llm_provider: visionConfig.provider }),
      };
      
      const newTenant = await createTenantMutation.mutateAsync(tenantData);
      toast.success(t('tenant.createSuccess', 'Tenant created'));
      
      // Select the new tenant
      selectTenant(newTenant.id);
      
      // Invalidate and wait for tenants to refetch
      await queryClient.invalidateQueries({ queryKey: ['tenants'] });
      
      // The backend auto-creates a default workspace, so refetch workspaces
      await queryClient.invalidateQueries({ queryKey: ['workspaces', newTenant.id] });
      
      setShowCreateTenant(false);
      setNewTenantName('EdgeQuake');
      // Reset model selections
      setTenantLlmModel(undefined);
      setTenantEmbeddingModel(undefined);
      setTenantVisionLlmModel(undefined);
      // Pre-fill workspace form and open dialog for the new tenant
      handleOpenCreateWorkspace(newTenant);
    } catch (error) {
      setIsSettingUpContext(false);
      toast.error(t('tenant.createFailed', 'Failed to create tenant'), {
        description: error instanceof Error ? error.message : 'Unknown error',
      });
    }
  }, [newTenantName, tenantLlmModel, tenantEmbeddingModel, tenantVisionLlmModel, parseModelValue, createTenantMutation, selectTenant, queryClient, t, handleOpenCreateWorkspace]);

  // Handle workspace creation with proper async flow - SPEC-032/SPEC-041: Now includes model config
  const handleCreateWorkspace = useCallback(async () => {
    if (!newWorkspaceName.trim() || !selectedTenantId) return;
    if (!workspaceLlmModel || !workspaceEmbeddingModel || !workspaceVisionLlmModel) return;
    
    setIsSettingUpContext(true);
    try {
      // SPEC-032/SPEC-041: Parse model selections and include in workspace creation
      const llmConfig = parseModelValue(workspaceLlmModel);
      const embeddingConfig = parseModelValue(workspaceEmbeddingModel);
      const visionConfig = parseModelValue(workspaceVisionLlmModel);
      
      const workspaceData = {
        name: newWorkspaceName,
        ...(newWorkspaceSlug.trim() && { slug: newWorkspaceSlug.trim() }),
        ...(llmConfig.model && { llm_model: llmConfig.model }),
        ...(llmConfig.provider && { llm_provider: llmConfig.provider }),
        ...(embeddingConfig.model && { embedding_model: embeddingConfig.model }),
        ...(embeddingConfig.provider && { embedding_provider: embeddingConfig.provider }),
        ...(visionConfig.model && { vision_llm_model: visionConfig.model }),
        ...(visionConfig.provider && { vision_llm_provider: visionConfig.provider }),
        // SPEC-085: Custom entity types
        ...(workspaceEntityTypes.length > 0 && { entity_types: workspaceEntityTypes }),
      };
      
      const newWorkspace = await createWorkspaceMutation.mutateAsync(workspaceData);
      toast.success(t('workspace.createSuccess', 'Workspace created'));
      
      // Optimistically update the store with the new workspace
      setWorkspaces([...(workspacesData || []), newWorkspace]);
      selectWorkspace(newWorkspace.id);
      
      // Invalidate and wait for workspaces to refetch
      await queryClient.invalidateQueries({ queryKey: ['workspaces', selectedTenantId] });
      
      setShowCreateWorkspace(false);
      setNewWorkspaceName('Default Workspace');
      setNewWorkspaceSlug('');
      // Reset model selections
      setWorkspaceLlmModel(undefined);
      setWorkspaceEmbeddingModel(undefined);
      setWorkspaceVisionLlmModel(undefined);
      setWorkspaceEntityTypes([...ENTITY_PRESETS.general.types]); // SPEC-085: Reset entity types
      setIsSettingUpContext(false);
    } catch (error) {
      setIsSettingUpContext(false);
      toast.error(t('workspace.createFailed', 'Failed to create workspace'), {
        description: error instanceof Error ? error.message : 'Unknown error',
      });
    }
  }, [newWorkspaceName, newWorkspaceSlug, workspaceLlmModel, workspaceEmbeddingModel, workspaceVisionLlmModel, workspaceEntityTypes,
      selectedTenantId, parseModelValue, createWorkspaceMutation, 
      selectWorkspace, setWorkspaces, workspacesData, queryClient, t]);

  // Error / long-wait state — prefer connection card over indefinite blank spinner
=======
    // While refetching after create, do not snap back to workspacesData[0].
    if (!selectedWorkspaceId) {
      selectWorkspace(merged[0].id);
    } else if (!workspaceExists && !isFetchingWorkspaces) {
      selectWorkspace(merged[0].id);
    }
    // eslint-disable-next-line react-hooks/set-state-in-effect -- context ready after heal
    setIsSettingUpContext(false);
  }, [
    workspacesData,
    setWorkspaces,
    selectedWorkspaceId,
    selectWorkspace,
    isFetchingWorkspaces,
  ]);

>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
  if (tenantsError || (isLoading && loadingTimedOut && !isSettingUpContext)) {
    return (
      <div className="flex items-center justify-center h-full p-4">
        <Card className="max-w-md w-full">
          <CardHeader className="text-center pb-2">
            <div className="mx-auto w-12 h-12 rounded-full bg-red-100 dark:bg-red-900/30 flex items-center justify-center mb-3">
              <AlertTriangle className="h-6 w-6 text-red-600 dark:text-red-400" />
            </div>
            <CardTitle>{t('tenant.connectionError', 'Connection Error')}</CardTitle>
            <CardDescription>
              {loadingTimedOut && !tenantsError
                ? t(
                    'tenant.connectionSlowDesc',
                    'The backend is taking too long to respond. It may be busy with ingestion — check the status banner above, then retry.',
                  )
                : t(
                    'tenant.connectionErrorDesc',
                    'Unable to connect to the server. Please check your connection and try again.',
                  )}
            </CardDescription>
          </CardHeader>
          <CardContent className="text-center">
            <Button
              onClick={() => {
                setLoadingTimedOut(false);
<<<<<<< HEAD
                queryClient.invalidateQueries({ queryKey: ['tenants'] });
                if (selectedTenantId) {
                  queryClient.invalidateQueries({
=======
                void queryClient.invalidateQueries({ queryKey: ['tenants'] });
                if (selectedTenantId) {
                  void queryClient.invalidateQueries({
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
                    queryKey: ['workspaces', selectedTenantId],
                  });
                }
              }}
            >
              {t('common.retry', 'Retry')}
            </Button>
          </CardContent>
        </Card>
      </div>
    );
  }

<<<<<<< HEAD
  // Loading state (including context setup after tenant/workspace creation)
  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-center">
          <Loader2 className="h-8 w-8 animate-spin mx-auto text-muted-foreground mb-3" />
          <p className="text-sm text-muted-foreground">
            {isSettingUpContext 
              ? t('tenant.settingUp', 'Setting up your workspace...')
              : t('tenant.loading', 'Loading workspace...')}
          </p>
=======
  if (isLoading) {
    const loadingLabel = isSettingUpContext
      ? t('tenant.settingUp', 'Setting up your workspace...')
      : t('tenant.loading', 'Loading workspace...');
    if (selectedTenantId && selectedWorkspaceId && !isSettingUpContext) {
      return (
        <div className="relative h-full min-h-0" data-testid="tenant-guard-overlay">
          {children}
          <div
            className="absolute inset-0 z-20 flex items-center justify-center bg-background/60"
            role="status"
            aria-busy="true"
            aria-label={loadingLabel}
          >
            <div className="text-center">
              <Loader2 className="mx-auto mb-3 h-8 w-8 animate-spin text-muted-foreground" />
              <p className="text-sm text-muted-foreground">{loadingLabel}</p>
            </div>
          </div>
        </div>
      );
    }
    return (
      <div className="flex h-full items-center justify-center">
        <div className="text-center">
          <Loader2 className="mx-auto mb-3 h-8 w-8 animate-spin text-muted-foreground" />
          <p className="text-sm text-muted-foreground">{loadingLabel}</p>
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
        </div>
      </div>
    );
  }

<<<<<<< HEAD
  // No tenants exist - prompt to create one
=======
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
  if (tenantsData && tenantsData.length === 0) {
    return (
      <>
        <div className="flex items-center justify-center h-full p-4">
          <Card className="max-w-md w-full">
            <CardHeader className="text-center pb-2">
              <div className="mx-auto w-12 h-12 rounded-full bg-primary/10 flex items-center justify-center mb-3">
                <Building2 className="h-6 w-6 text-primary" />
              </div>
              <CardTitle>{t('tenant.welcome', 'Welcome to EdgeQuake')}</CardTitle>
              <CardDescription>
                {t(
                  'tenant.createFirstTenant',
                  'Create your first tenant to get started. A tenant represents an organization or project.',
                )}
              </CardDescription>
            </CardHeader>
            <CardContent className="text-center">
              <Button onClick={() => setShowCreateTenant(true)} data-testid="guard-create-tenant">
                <Plus className="h-4 w-4 mr-2" />
                {t('tenant.createTenant', 'Create Tenant')}
              </Button>
            </CardContent>
          </Card>
        </div>
<<<<<<< HEAD

        <Dialog open={showCreateTenant} onOpenChange={setShowCreateTenant}>
          <DialogContent className="w-[95vw] sm:max-w-190 max-h-[92vh] overflow-hidden grid-rows-[auto_minmax(0,1fr)_auto]">
            <DialogHeader>
              <DialogTitle>{t('tenant.createNew', 'Create Tenant')}</DialogTitle>
              <DialogDescription>
                {t('tenant.createNewDesc', 'Configure your organization and default models.')}
              </DialogDescription>
            </DialogHeader>
            <div className="grid gap-3 py-3 sm:grid-cols-2 overflow-y-auto pr-1">
              <div className="grid gap-2 sm:col-span-2">
                <Label htmlFor="tenant-name">{t('common.name', 'Name')}</Label>
                <Input
                  id="tenant-name"
                  value={newTenantName}
                  onChange={(e) => setNewTenantName(e.target.value)}
                  placeholder="My Organization"
                />
              </div>
              
              {/* SPEC-032: Default LLM Model Selection */}
              <div className="grid gap-2">
                <Label>
                  {t('tenant.defaultLlmModel', 'Default LLM Model')}
                  <span className="text-destructive ml-0.5">*</span>
                </Label>
                <ModelSelector
                  type="llm"
                  value={tenantLlmModel}
                  onChange={(value) => setTenantLlmModel(value)}
                  placeholder={t('tenant.selectLlmModel', 'Select LLM model...')}
                />
                <p className="text-xs text-muted-foreground">
                  {t('tenant.llmModelHint', 'Used for knowledge graph generation and summarization')}
                </p>
              </div>

              {/* SPEC-032: Default Embedding Model Selection */}
              <div className="grid gap-2">
                <Label>
                  {t('tenant.defaultEmbeddingModel', 'Default Embedding Model')}
                  <span className="text-destructive ml-0.5">*</span>
                </Label>
                <ModelSelector
                  type="embedding"
                  value={tenantEmbeddingModel}
                  onChange={(value) => setTenantEmbeddingModel(value)}
                  placeholder={t('tenant.selectEmbeddingModel', 'Select embedding model...')}
                />
                <p className="text-xs text-muted-foreground">
                  {t('tenant.embeddingModelHint', 'Used for document search and retrieval')}
                </p>
              </div>

              {/* SPEC-041: Default Vision LLM Selection */}
              <div className="grid gap-2">
                <Label>
                  {t('tenant.defaultVisionLlmModel', 'Default Vision LLM')}
                  <span className="text-destructive ml-0.5">*</span>
                </Label>
                <ModelSelector
                  type="llm"
                  filterVision
                  value={tenantVisionLlmModel}
                  onChange={(value) => setTenantVisionLlmModel(value)}
                  placeholder={t('tenant.selectVisionLlmModel', 'Select vision model...')}
                />
                <p className="text-xs text-muted-foreground">
                  {t('tenant.visionLlmModelHint', 'Used for PDF-to-Markdown image extraction (must support vision)')}
                </p>
              </div>
            </div>
            <DialogFooter>
              <Button variant="outline" onClick={() => setShowCreateTenant(false)}>
                {t('common.cancel', 'Cancel')}
              </Button>
              <Button
                onClick={handleCreateTenant}
                disabled={!newTenantName.trim() || !tenantLlmModel || !tenantEmbeddingModel || !tenantVisionLlmModel || createTenantMutation.isPending}
              >
                {createTenantMutation.isPending && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
                {t('common.create', 'Create')}
              </Button>
            </DialogFooter>
          </DialogContent>
        </Dialog>
=======
        <CreateTenantWizard
          open={showCreateTenant}
          onOpenChange={setShowCreateTenant}
          onCreated={(tenant, workspace) => {
            setIsSettingUpContext(true);
            applyCreatedTenantContext(tenant, workspace);
            void queryClient.invalidateQueries({ queryKey: ['tenants'] });
            void queryClient.invalidateQueries({ queryKey: ['workspaces', tenant.id] });
            toast.success(t('tenant.createSuccess', 'Tenant created successfully'), {
              action: {
                label: t('onboarding.uploadDocuments', 'Upload documents'),
                onClick: () => {
                  window.location.href = '/documents';
                },
              },
            });
          }}
        />
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
      </>
    );
  }

  if (selectedTenantId && workspacesData && workspacesData.length === 0) {
    return (
      <>
        <div className="flex items-center justify-center h-full p-4">
          <Card className="max-w-md w-full">
            <CardHeader className="text-center pb-2">
              <div className="mx-auto w-12 h-12 rounded-full bg-primary/10 flex items-center justify-center mb-3">
                <FolderKanban className="h-6 w-6 text-primary" />
              </div>
              <CardTitle>{t('workspace.createFirst', 'Create a Workspace')}</CardTitle>
              <CardDescription>
                {t(
                  'workspace.createFirstDesc',
                  'Create your first workspace to start uploading documents and building your knowledge graph.',
                )}
              </CardDescription>
            </CardHeader>
            <CardContent className="text-center">
<<<<<<< HEAD
              <Button onClick={() => handleOpenCreateWorkspace()}>
=======
              <Button
                onClick={() => setShowCreateWorkspace(true)}
                data-testid="guard-create-workspace"
              >
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
                <Plus className="h-4 w-4 mr-2" />
                {t('workspace.createWorkspace', 'Create Workspace')}
              </Button>
            </CardContent>
          </Card>
        </div>
<<<<<<< HEAD

        <Dialog open={showCreateWorkspace} onOpenChange={setShowCreateWorkspace}>
          <DialogContent className="w-[95vw] sm:max-w-190 max-h-[92vh] overflow-hidden grid-rows-[auto_minmax(0,1fr)_auto]">
            <DialogHeader>
              <DialogTitle>{t('workspace.createNew', 'Create Workspace')}</DialogTitle>
              <DialogDescription>
                {t('workspace.createNewDesc', 'Configure your workspace and AI models.')}
              </DialogDescription>
            </DialogHeader>
            <div className="grid gap-3 py-3 sm:grid-cols-2 overflow-y-auto pr-1">
              <div className="grid gap-2">
                <Label htmlFor="workspace-name">{t('common.name', 'Name')}</Label>
                <Input
                  id="workspace-name"
                  value={newWorkspaceName}
                  onChange={(e) => {
                    setNewWorkspaceName(e.target.value);
                    // Auto-generate slug if user hasn't manually edited it
                    if (!newWorkspaceSlug || newWorkspaceSlug === generateSlug(newWorkspaceName)) {
                      setNewWorkspaceSlug(generateSlug(e.target.value));
                    }
                  }}
                  placeholder="My Project"
                />
              </div>
              <div className="grid gap-2 sm:col-span-2">
                <Label htmlFor="workspace-slug">
                  {t('workspace.slug', 'URL Slug')}
                  <span className="text-muted-foreground text-xs ml-2">
                    {t('workspace.slugHint', '(optional, auto-generated)')}
                  </span>
                </Label>
                <Input
                  id="workspace-slug"
                  value={newWorkspaceSlug}
                  onChange={(e) => setNewWorkspaceSlug(e.target.value.toLowerCase().replace(/[^a-z0-9-]/g, '-'))}
                  placeholder="my-project"
                  pattern="[a-z0-9-]+"
                />
                <p className="text-xs text-muted-foreground">
                  {t('workspace.slugDescription', 'Used in URLs: /w/{slug}/query')}
                </p>
              </div>

              {/* SPEC-032: LLM Model Selection */}
              <div className="grid gap-2">
                <Label>
                  {t('workspace.llmModel', 'LLM Model')}
                  <span className="text-destructive ml-0.5">*</span>
                </Label>
                <ModelSelector
                  type="llm"
                  value={workspaceLlmModel}
                  onChange={(value) => setWorkspaceLlmModel(value)}
                  placeholder={t('workspace.selectLlmModel', 'Use tenant default...')}
                />
                <p className="text-xs text-muted-foreground">
                  {t('workspace.llmModelHint', 'For knowledge graph generation and queries')}
                </p>
              </div>

              {/* SPEC-032: Embedding Model Selection */}
              <div className="grid gap-2">
                <Label>
                  {t('workspace.embeddingModel', 'Embedding Model')}
                  <span className="text-destructive ml-0.5">*</span>
                </Label>
                <ModelSelector
                  type="embedding"
                  value={workspaceEmbeddingModel}
                  onChange={(value) => setWorkspaceEmbeddingModel(value)}
                  placeholder={t('workspace.selectEmbeddingModel', 'Use tenant default...')}
                />
                <p className="text-xs text-muted-foreground">
                  {t('workspace.embeddingModelHint', 'For document search and similarity')}
                </p>
              </div>

              {/* SPEC-041: Vision LLM Selection */}
              <div className="grid gap-2">
                <Label>
                  {t('workspace.visionLlmModel', 'Vision LLM')}
                  <span className="text-destructive ml-0.5">*</span>
                </Label>
                <ModelSelector
                  type="llm"
                  filterVision
                  value={workspaceVisionLlmModel}
                  onChange={(value) => setWorkspaceVisionLlmModel(value)}
                  placeholder={t('workspace.selectVisionLlmModel', 'Use tenant default...')}
                />
                <p className="text-xs text-muted-foreground">
                  {t('workspace.visionLlmModelHint', 'For PDF-to-Markdown image extraction (must support vision)')}
                </p>
              </div>
              {/* SPEC-085: Entity type configuration */}
              <div className="grid gap-2 sm:col-span-2">
                <Label>{t('entityTypes.title', 'Entity Types')}</Label>
                <p className="text-xs text-muted-foreground">
                  {t('entityTypes.description', 'Types of entities to extract from documents in this workspace.')}
                </p>
                <EntityTypeSelector
                  value={workspaceEntityTypes}
                  onChange={setWorkspaceEntityTypes}
                />
              </div>
            </div>
            <DialogFooter>
              <Button variant="outline" onClick={() => setShowCreateWorkspace(false)}>
                {t('common.cancel', 'Cancel')}
              </Button>
              <Button
                onClick={handleCreateWorkspace}
                disabled={!newWorkspaceName.trim() || !workspaceLlmModel || !workspaceEmbeddingModel || !workspaceVisionLlmModel || createWorkspaceMutation.isPending}
              >
                {createWorkspaceMutation.isPending && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
                {t('common.create', 'Create')}
              </Button>
            </DialogFooter>
          </DialogContent>
        </Dialog>
=======
        <CreateWorkspaceWizard
          open={showCreateWorkspace}
          onOpenChange={setShowCreateWorkspace}
          tenantId={selectedTenantId}
          onCreated={(workspace) => {
            setIsSettingUpContext(true);
            applyCreatedWorkspaceContext(workspace);
            void queryClient.invalidateQueries({
              queryKey: ['workspaces', selectedTenantId],
            });
            toast.success(t('workspace.createSuccess', 'Workspace created successfully'), {
              action: {
                label: t('onboarding.uploadDocuments', 'Upload documents'),
                onClick: () => {
                  window.location.href = '/documents';
                },
              },
            });
          }}
        />
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
      </>
    );
  }

  if (!selectedTenantId || !selectedWorkspaceId) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-center">
          <Loader2 className="h-8 w-8 animate-spin mx-auto text-muted-foreground mb-3" />
          <p className="text-sm text-muted-foreground">
            {t('tenant.selectingWorkspace', 'Selecting workspace...')}
          </p>
        </div>
      </div>
    );
  }

  return <>{children}</>;
}

export default TenantGuard;
