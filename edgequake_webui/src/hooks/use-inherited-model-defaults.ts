'use client';

import { useServerModelDefaults } from '@/hooks/use-server-model-defaults';
import { getTenant, getWorkspaces } from '@/lib/api/edgequake';
import {
  pickDefaultWorkspaceLanguage,
  resolveInheritedModelDefaults,
  type ResolvedInheritedDefaults,
} from '@/lib/onboarding/inherited-defaults';
import { useTenantStore } from '@/stores/use-tenant-store';
import { useQuery } from '@tanstack/react-query';
import { useMemo } from 'react';

/**
 * Tenant → server inheritance for Create Workspace wizard (SPEC-101).
 * Language is taken from the tenant’s Default Workspace when present.
 */
export function useInheritedModelDefaults(
  tenantId: string | null | undefined,
): ResolvedInheritedDefaults & {
  isLoading: boolean;
} {
  const server = useServerModelDefaults();
  const storeTenant = useTenantStore((s) =>
    tenantId ? s.tenants.find((t) => t.id === tenantId) : undefined,
  );

  const tenantQuery = useQuery({
    queryKey: ['tenants', tenantId],
    queryFn: () => getTenant(tenantId!),
    enabled: Boolean(tenantId),
    staleTime: 60_000,
    // List payloads may omit model defaults; prefer GET /tenants/{id}.
    placeholderData: storeTenant,
  });

  const workspacesQuery = useQuery({
    queryKey: ['workspaces', tenantId],
    queryFn: () => getWorkspaces(tenantId!),
    enabled: Boolean(tenantId),
    staleTime: 60_000,
  });

  const tenant = tenantQuery.data ?? storeTenant ?? null;
  const extractionLanguage = useMemo(
    () => pickDefaultWorkspaceLanguage(workspacesQuery.data ?? []),
    [workspacesQuery.data],
  );

  const {
    defaultLlmProvider,
    defaultLlmModel,
    defaultEmbeddingProvider,
    defaultEmbeddingModel,
    defaultVisionProvider,
    defaultVisionModel,
    isLoading: serverLoading,
  } = server;

  const resolved = useMemo(
    () =>
      resolveInheritedModelDefaults(
        tenant,
        {
          defaultLlmProvider,
          defaultLlmModel,
          defaultEmbeddingProvider,
          defaultEmbeddingModel,
          defaultVisionProvider,
          defaultVisionModel,
        },
        extractionLanguage,
      ),
    [
      tenant,
      defaultLlmProvider,
      defaultLlmModel,
      defaultEmbeddingProvider,
      defaultEmbeddingModel,
      defaultVisionProvider,
      defaultVisionModel,
      extractionLanguage,
    ],
  );

  const isLoading =
    serverLoading ||
    (Boolean(tenantId) && tenantQuery.isLoading && !tenantQuery.data && !storeTenant) ||
    (Boolean(tenantId) && workspacesQuery.isLoading && !workspacesQuery.data);

  return { ...resolved, isLoading };
}
