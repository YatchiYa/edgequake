/**
 * Accessible sortable column header (WAI-ARIA APG sortable table pattern).
 *
 * - `aria-sort` on the `th`
 * - interactive control is a full-cell `button`
 * - sort glyph is aria-hidden (name comes from button label)
 */
'use client';

import { TableHead } from '@/components/ui/table';
import {
  ariaSortForColumn,
  type SortDirection,
  type SortField,
} from '@/lib/documents/document-sort';
import { ArrowDown, ArrowUp, ArrowUpDown } from 'lucide-react';
import { cn } from '@/lib/utils';

export interface SortableColumnHeaderProps {
  field: SortField;
  label: string;
  activeField: SortField;
  direction: SortDirection;
  onSort: (field: SortField) => void;
  className?: string;
  align?: 'left' | 'center' | 'right';
}

export function SortableColumnHeader({
  field,
  label,
  activeField,
  direction,
  onSort,
  className,
  align = 'left',
}: SortableColumnHeaderProps) {
  const ariaSort = ariaSortForColumn(field, activeField, direction);
  const isActive = field === activeField;

  return (
    <TableHead
      scope="col"
      aria-sort={ariaSort === 'none' ? undefined : ariaSort}
      className={cn(
        align === 'center' && 'text-center',
        align === 'right' && 'text-right',
        className,
      )}
      data-testid={`sort-header-${field}`}
    >
      <button
        type="button"
        className={cn(
          'inline-flex w-full min-w-0 items-center gap-1 rounded-sm px-0 py-0.5',
          'text-left font-medium text-muted-foreground hover:text-foreground',
          'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring',
          align === 'center' && 'justify-center text-center',
          align === 'right' && 'justify-end text-right',
          isActive && 'text-foreground',
        )}
        onClick={() => onSort(field)}
        aria-label={
          isActive
            ? `${label}, sorted ${direction === 'asc' ? 'ascending' : 'descending'}. Activate to reverse.`
            : `Sort by ${label}`
        }
        title={label}
      >
        <span className="truncate">{label}</span>
        <span className="inline-flex shrink-0 opacity-70" aria-hidden="true">
          {isActive ? (
            direction === 'asc' ? (
              <ArrowUp className="h-3.5 w-3.5" />
            ) : (
              <ArrowDown className="h-3.5 w-3.5" />
            )
          ) : (
            <ArrowUpDown className="h-3.5 w-3.5 opacity-50" />
          )}
        </span>
      </button>
    </TableHead>
  );
}

export default SortableColumnHeader;
