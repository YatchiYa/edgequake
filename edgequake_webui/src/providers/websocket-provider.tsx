/**
 * @module WebSocketProvider
 * @description WebSocket connection context for real-time progress tracking (SPEC-149).
 *
 * @implements FEAT0724 - Real-time ingestion progress
 * @implements FEAT0865 - WebSocket connection management
 * @implements SPEC-149 - Auth-gated connect, session reset, toast hygiene
 */
'use client';

import { getTokens } from '@/lib/api/client-context';
import { getRuntimeConfig } from '@/lib/runtime-config';
import type { ProgressWebSocket } from '@/lib/websocket';
import {
  disconnectWebSocket,
  destroyWebSocketClient,
  getWebSocketClient,
  reconnectRealtime,
} from '@/lib/websocket';
import { useAuthStore, useAuthStoreHydrated } from '@/stores/use-auth-store';
import { useCostStore } from '@/stores/use-cost-store';
import { useIngestionStore } from '@/stores/use-ingestion-store';
import type { CostUpdateEvent } from '@/types/cost';
import type { IngestionFailedEvent, WebSocketProgressMessage } from '@/types/ingestion';
import { createContext, useCallback, useContext, useEffect, useRef, type ReactNode } from 'react';
import { useTranslation } from 'react-i18next';
import { toast } from 'sonner';

const TOAST_MAX_RECONNECTS = 'ws-max-reconnects';
const TOAST_CONNECTION_LOST = 'ws-connection-lost';

interface WebSocketContextValue {
  connected: boolean;
  reconnecting: boolean;
  subscribe: (trackIds: string[]) => void;
  unsubscribe: (trackIds: string[]) => void;
  cancel: (trackId: string) => void;
  connect: () => void;
  disconnect: () => void;
}

const WebSocketContext = createContext<WebSocketContextValue | null>(null);

interface WebSocketProviderProps {
  children: ReactNode;
  /** Whether to auto-connect on mount when auth allows (default: true) */
  autoConnect?: boolean;
  /** Whether WebSocket is enabled (default: true) */
  enabled?: boolean;
}

function canConnectRealtime(): boolean {
  const { authEnabled } = getRuntimeConfig();
  if (!authEnabled) return true;
  const { accessToken } = getTokens();
  return Boolean(accessToken);
}

export function WebSocketProvider({
  children,
  autoConnect = true,
  enabled = true,
}: WebSocketProviderProps) {
  const clientRef = useRef<ProgressWebSocket | null>(null);
  const { t } = useTranslation();
  const hasHydrated = useAuthStoreHydrated();
  const isAuthenticated = useAuthStore((s) => s.isAuthenticated);
  const accessToken = useAuthStore((s) => s.accessToken);

  const { updateFromMessage, setWsConnected, setWsReconnecting, setWsMaxReconnectsReached } =
    useIngestionStore();
  const { updateIngestionCost } = useCostStore();

  const handleMessage = useCallback(
    (message: WebSocketProgressMessage | CostUpdateEvent) => {
      if (message.type === 'ingestion_failed') {
        const failedEvent = message as IngestionFailedEvent;
        console.error('[WebSocket] Ingestion failed:', {
          track_id: failedEvent.track_id,
          document_id: failedEvent.document_id,
          stage: failedEvent.stage,
          error: failedEvent.error,
        });

        toast.error(
          t('websocket.ingestionFailed', 'Document processing failed'),
          {
            duration: 10000,
            description: t('websocket.ingestionFailedDesc', 'Stage: {{stage}}', {
              stage: failedEvent.stage,
            }),
            action: failedEvent.error.recoverable
              ? {
                  label: t('websocket.retry', 'Retry'),
                  onClick: () => {
                    console.log('[WebSocket] Retry requested for:', failedEvent.track_id);
                  },
                }
              : undefined,
          },
        );
      }

      updateFromMessage(message);

      if (message.type === 'cost_update') {
        const costMessage = message as CostUpdateEvent;
        updateIngestionCost(costMessage.track_id, costMessage.cumulative_cost_usd);
      }
    },
    [updateFromMessage, updateIngestionCost, t],
  );

  // Bind listeners once; connect only when session allows.
  useEffect(() => {
    if (!enabled) return;

    const client = getWebSocketClient();
    clientRef.current = client;

    const unsubConnected = client.on('connected', () => {
      setWsConnected(true);
      setWsReconnecting(false);
      toast.dismiss(TOAST_MAX_RECONNECTS);
      toast.dismiss(TOAST_CONNECTION_LOST);
      if (useIngestionStore.getState().wsMaxReconnectsReached) {
        setWsMaxReconnectsReached(false);
        toast.success(t('websocket.connectionRestored', 'Connection restored'), {
          description: t(
            'websocket.connectionRestoredDesc',
            'Real-time updates are back online.',
          ),
          duration: 3000,
        });
      }
    });

    const unsubDisconnected = client.on('disconnected', () => {
      setWsConnected(false);
      toast.warning(t('websocket.connectionLost', 'Connection lost'), {
        id: TOAST_CONNECTION_LOST,
        description: t('websocket.connectionLostDesc', 'Attempting to reconnect...'),
        duration: 5000,
      });
    });

    const unsubReconnecting = client.on('reconnecting', () => {
      setWsReconnecting(true);
    });

    const unsubMaxReconnects = client.on('max_reconnects_reached', () => {
      setWsReconnecting(false);
      setWsMaxReconnectsReached(true);
      console.warn('[WebSocketProvider] Max reconnection attempts reached');
      toast.error(t('websocket.unableToReconnect', 'Unable to reconnect'), {
        id: TOAST_MAX_RECONNECTS,
        description: t(
          'websocket.unableToReconnectDesc',
          'Real-time updates unavailable. Click to retry.',
        ),
        duration: Infinity,
        action: {
          label: t('websocket.retry', 'Retry'),
          onClick: () => {
            setWsMaxReconnectsReached(false);
            clientRef.current = reconnectRealtime();
          },
        },
      });
    });

    const unsubProgress = client.on('progress', (message) => {
      handleMessage(message as WebSocketProgressMessage);
    });

    const unsubPdfProgress = client.on('pdf_progress', (message) => {
      handleMessage(message as WebSocketProgressMessage);
    });

    const unsubStatusSnapshot = client.on('status_snapshot', (message) => {
      handleMessage(message as WebSocketProgressMessage);
    });

    return () => {
      unsubConnected();
      unsubDisconnected();
      unsubReconnecting();
      unsubMaxReconnects();
      unsubProgress();
      unsubPdfProgress();
      unsubStatusSnapshot();
    };
  }, [
    enabled,
    handleMessage,
    setWsConnected,
    setWsReconnecting,
    setWsMaxReconnectsReached,
    t,
  ]);

  // Auth-gated connect + rebuild on token change (LAW-149-1/2).
  const lastTokenRef = useRef<string | null | undefined>(undefined);
  useEffect(() => {
    if (!enabled || !autoConnect) return;
    if (!hasHydrated) return;

    const { authEnabled } = getRuntimeConfig();
    if (authEnabled && (!isAuthenticated || !canConnectRealtime())) {
      lastTokenRef.current = null;
      disconnectWebSocket();
      clientRef.current = getWebSocketClient();
      setWsConnected(false);
      return;
    }

    if (!canConnectRealtime() && authEnabled) return;

    const token = getTokens().accessToken ?? null;
    // Avoid tearing down a healthy socket when deps re-fire with the same token.
    if (lastTokenRef.current === token && clientRef.current?.connected) {
      return;
    }
    lastTokenRef.current = token;
    clientRef.current = reconnectRealtime();
  }, [
    enabled,
    autoConnect,
    hasHydrated,
    isAuthenticated,
    accessToken,
    setWsConnected,
  ]);

  useEffect(() => {
    return () => {
      destroyWebSocketClient();
    };
  }, []);

  const subscribe = useCallback((trackIds: string[]) => {
    const client = getWebSocketClient();
    clientRef.current = client;
    client.subscribe(trackIds);
  }, []);

  const unsubscribe = useCallback((trackIds: string[]) => {
    clientRef.current?.unsubscribe(trackIds);
  }, []);

  const cancel = useCallback((trackId: string) => {
    clientRef.current?.cancel(trackId);
  }, []);

  const connect = useCallback(() => {
    clientRef.current = reconnectRealtime();
  }, []);

  const disconnect = useCallback(() => {
    clientRef.current?.disconnect();
  }, []);

  const storeConnected = useIngestionStore((s) => s.wsConnected);
  const storeReconnecting = useIngestionStore((s) => s.wsReconnecting);

  const value: WebSocketContextValue = {
    connected: storeConnected,
    reconnecting: storeReconnecting,
    subscribe,
    unsubscribe,
    cancel,
    connect,
    disconnect,
  };

  return (
    <WebSocketContext.Provider value={value}>{children}</WebSocketContext.Provider>
  );
}

export function useWebSocketContext(): WebSocketContextValue {
  const context = useContext(WebSocketContext);
  if (!context) {
    throw new Error('useWebSocketContext must be used within a WebSocketProvider');
  }
  return context;
}

export function useWebSocketStatus(): { connected: boolean; reconnecting: boolean } {
  const { wsConnected, wsReconnecting } = useIngestionStore();
  return { connected: wsConnected, reconnecting: wsReconnecting };
}
