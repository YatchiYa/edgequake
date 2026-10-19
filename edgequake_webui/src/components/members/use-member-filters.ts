'use client';

import { useDebounce } from '@/hooks/use-debounce';
import type { MemberListParams, MemberSort, MemberStatus, OrgRole } from '@/lib/api/edgequake/members';
import { useMemo, useState } from 'react';

export const PAGE_SIZE = 25;

/** Filter/sort/page state of the directory; `params` is what goes on the wire. */
export function useMemberFilters() {
  const [search, setSearch] = useState('');
  const [role, setRole] = useState<OrgRole | 'all'>('all');
  const [status, setStatus] = useState<MemberStatus | 'all'>('all');
  const [sort, setSort] = useState<MemberSort>('name');
  const [dir, setDir] = useState<'asc' | 'desc'>('asc');
  const [page, setPage] = useState(1);
  const q = useDebounce(search.trim(), 250);

  const params: MemberListParams = useMemo(
    () => ({ q, role, status, sort, dir, page, page_size: PAGE_SIZE }),
    [q, role, status, sort, dir, page],
  );

  const resetPage = <T,>(setter: (v: T) => void) => (v: T) => {
    setter(v);
    setPage(1);
  };

  const toggleSort = (next: MemberSort) => {
    if (next === sort) setDir((d) => (d === 'asc' ? 'desc' : 'asc'));
    else {
      setSort(next);
      setDir(next === 'last_active' || next === 'joined' ? 'desc' : 'asc');
    }
    setPage(1);
  };

  const filtered = q !== '' || role !== 'all' || status !== 'all';
  const clear = () => {
    setSearch('');
    setRole('all');
    setStatus('all');
    setPage(1);
  };

  return {
    params,
    search,
    setSearch: resetPage(setSearch),
    role,
    setRole: resetPage(setRole),
    status,
    setStatus: resetPage(setStatus),
    sort,
    dir,
    toggleSort,
    page,
    setPage,
    filtered,
    clear,
  };
}

export type MemberFilters = ReturnType<typeof useMemberFilters>;
