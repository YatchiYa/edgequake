"use client";

import {
  getWorkspace,
  getWorkspaceStats,
} from "@/lib/api/edgequake";
import { fetchProvidersHealth } from "@/lib/api/models";
import { useQuery } from "@tanstack/react-query";

export interface UseWorkspaceDetailQueriesOptions {
  /** When false, all queries are disabled. */
  enabled?: boolean;
}

/**
 * Shared workspace detail data fetching for dashboard and deeplink routes.
 * @implements SPEC-017 UI-P3-001
 */
export function useWorkspaceDetailQueries(
  tenantId: string | null | undefined,
  workspaceId: string | null | undefined,
  options: UseWorkspaceDetailQueriesOptions = {},
) {
  const enabled =
    options.enabled !== false && !!tenantId && !!workspaceId;

  const workspaceQuery = useQuery({
    queryKey: ["workspace", tenantId, workspaceId],
    queryFn: () => getWorkspace(tenantId!, workspaceId!),
    enabled,
    staleTime: 30_000,
<<<<<<< HEAD
=======
    // SPEC-100: keep cached workspace painted during soft refetch
    placeholderData: (previous) => previous,
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
  });

  const statsQuery = useQuery({
    queryKey: ["workspaceStats", workspaceId],
    queryFn: () => getWorkspaceStats(workspaceId!),
    enabled: enabled && !!workspaceId,
    staleTime: 0,
    refetchOnMount: "always",
<<<<<<< HEAD
=======
    placeholderData: (previous) => previous,
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
  });

  const providerHealthQuery = useQuery({
    queryKey: ["providersHealth"],
    queryFn: fetchProvidersHealth,
    enabled,
    staleTime: 60_000,
    retry: 1,
<<<<<<< HEAD
=======
    placeholderData: (previous) => previous,
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
  });

  return {
    workspace: workspaceQuery.data,
    stats: statsQuery.data,
    providerHealth: providerHealthQuery.data,
<<<<<<< HEAD
    isLoadingWorkspace: workspaceQuery.isLoading,
    isLoadingStats: statsQuery.isLoading,
    isLoadingHealth: providerHealthQuery.isLoading,
=======
    // SPEC-100: full-page skeleton only on cold load (no cached workspace)
    isLoadingWorkspace: workspaceQuery.isLoading && !workspaceQuery.data,
    isLoadingStats: statsQuery.isLoading && !statsQuery.data,
    isLoadingHealth: providerHealthQuery.isLoading && !providerHealthQuery.data,
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
    refetchWorkspace: workspaceQuery.refetch,
  };
}
