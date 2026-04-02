/**
 * @module use-injection
 * @description React Query hooks for knowledge injection CRUD operations.
 * @implements SPEC-0002 - Knowledge Injection for Enhanced Search
 */

import {
    deleteInjection,
    getInjection,
    listInjections,
    putInjection,
    putInjectionFile,
    updateInjection,
    type InjectionDetailResponse,
    type ListInjectionsResponse,
    type PutInjectionRequest,
    type PutInjectionResponse,
    type UpdateInjectionRequest
} from "@/lib/api/edgequake";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

export const injectionKeys = {
  all: ["injections"] as const,
  list: (workspaceId: string) =>
    [...injectionKeys.all, "list", workspaceId] as const,
  detail: (workspaceId: string, injectionId: string) =>
    [...injectionKeys.all, "detail", workspaceId, injectionId] as const,
};

export function useInjections(workspaceId: string | null) {
  return useQuery<ListInjectionsResponse>({
    queryKey: injectionKeys.list(workspaceId ?? ""),
    queryFn: () => listInjections(workspaceId!),
    enabled: !!workspaceId,
    staleTime: 30_000,
  });
}

export function useInjectionDetail(
  workspaceId: string | null,
  injectionId: string | null,
) {
  return useQuery<InjectionDetailResponse>({
    queryKey: injectionKeys.detail(workspaceId ?? "", injectionId ?? ""),
    queryFn: () => getInjection(workspaceId!, injectionId!),
    enabled: !!workspaceId && !!injectionId,
  });
}

export function useCreateInjection(workspaceId: string) {
  const queryClient = useQueryClient();
  return useMutation<PutInjectionResponse, Error, PutInjectionRequest>({
    mutationFn: (request) => putInjection(workspaceId, request),
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: injectionKeys.list(workspaceId),
      });
    },
  });
}

export function useDeleteInjection(workspaceId: string) {
  const queryClient = useQueryClient();
  return useMutation<{ deleted: boolean; message: string }, Error, string>({
    mutationFn: (injectionId) => deleteInjection(workspaceId, injectionId),
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: injectionKeys.list(workspaceId),
      });
    },
  });
}

export function useUpdateInjection(workspaceId: string, injectionId: string) {
  const queryClient = useQueryClient();
  return useMutation<PutInjectionResponse, Error, UpdateInjectionRequest>({
    mutationFn: (request) => updateInjection(workspaceId, injectionId, request),
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: injectionKeys.list(workspaceId),
      });
      queryClient.invalidateQueries({
        queryKey: injectionKeys.detail(workspaceId, injectionId),
      });
    },
  });
}

export function useCreateInjectionFile(workspaceId: string) {
  const queryClient = useQueryClient();
  return useMutation<PutInjectionResponse, Error, { name: string; file: File }>({
    mutationFn: ({ name, file }) => putInjectionFile(workspaceId, name, file),
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: injectionKeys.list(workspaceId),
      });
    },
  });
}
