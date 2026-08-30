/**
 * SPEC-140 — Exhaust paginated `{ items, total, offset, limit }` list endpoints.
 *
 * LAW-140-3: a context switcher must follow pages until `accumulated >= total`.
 * Safety cap prevents infinite loops if a buggy server keeps returning full pages
 * with a lying `total`.
 */

export const SELECTOR_PAGE_LIMIT = 100;
export const FETCH_ALL_PAGES_MAX = 50;

export type PaginatedList<T> = {
  items: T[];
  total: number;
  offset?: number;
  limit?: number;
};

/**
 * Fetch successive pages until the catalog is complete (or the safety cap).
 */
export async function fetchAllPages<T>(
  fetchPage: (offset: number, limit: number) => Promise<PaginatedList<T>>,
  pageLimit: number = SELECTOR_PAGE_LIMIT,
): Promise<T[]> {
  const all: T[] = [];
  let offset = 0;
  const cap = Math.max(1, pageLimit);

  for (let page = 0; page < FETCH_ALL_PAGES_MAX; page += 1) {
    const { items, total } = await fetchPage(offset, cap);
    const batch = Array.isArray(items) ? items : [];
    all.push(...batch);
    const honestTotal = Number.isFinite(total) ? total : all.length;
    if (batch.length === 0) break;
    if (all.length >= honestTotal) break;
    if (batch.length < cap) break;
    offset += batch.length;
  }
  return all;
}

/**
 * SPEC-141 — Exhaust 1-based `{ items, total }` lists (task queue `page` /
 * `page_size`). Same safety cap as `fetchAllPages`.
 */
export async function fetchAllPagesByIndex<T>(
  fetchPage: (page: number, pageSize: number) => Promise<PaginatedList<T>>,
  pageSize: number = SELECTOR_PAGE_LIMIT,
): Promise<T[]> {
  const all: T[] = [];
  const cap = Math.max(1, pageSize);

  for (let page = 1; page <= FETCH_ALL_PAGES_MAX; page += 1) {
    const { items, total } = await fetchPage(page, cap);
    const batch = Array.isArray(items) ? items : [];
    all.push(...batch);
    const honestTotal = Number.isFinite(total) ? total : all.length;
    if (batch.length === 0) break;
    if (all.length >= honestTotal) break;
    if (batch.length < cap) break;
  }
  return all;
}
