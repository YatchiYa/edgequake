'use client';

import { fetchSetupStatus, type SetupStatus } from '@/lib/api/setup';
import { useTenantStore } from '@/stores/use-tenant-store';
import { useQuery } from '@tanstack/react-query';
import { useEffect } from 'react';

/**
 * SPEC-101 — Poll setup status and wire `needsOnboarding` in the tenant store.
 */
export function useSetupStatus() {
  const setNeedsOnboarding = useTenantStore((s) => s.setNeedsOnboarding);

  const query = useQuery<SetupStatus>({
    queryKey: ['setup', 'status'],
    queryFn: fetchSetupStatus,
    staleTime: 30_000,
    retry: 1,
  });

  useEffect(() => {
    if (query.data) {
      setNeedsOnboarding(query.data.needs_setup);
    }
  }, [query.data, setNeedsOnboarding]);

  return query;
}
