/**
 * SPEC-120: operation resource client (transparent alias over tasks).
 */

import { api } from "../client";
import type { OperationCancelResponse, TaskResponse } from "@/types";

export async function getOperation(id: string): Promise<TaskResponse> {
  return api.get<TaskResponse>(`/operations/${id}`);
}

export async function cancelOperation(
  id: string,
): Promise<OperationCancelResponse> {
  return api.post<OperationCancelResponse>(`/operations/${id}/cancel`);
}

export async function getOperationEvents(id: string): Promise<unknown[]> {
  return api.get<unknown[]>(`/operations/${id}/events`);
}

export type { OperationCancelResponse };
