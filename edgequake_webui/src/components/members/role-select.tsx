'use client';

import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuLabel, DropdownMenuSeparator, DropdownMenuTrigger } from '@/components/ui/dropdown-menu';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import type { OrgRole } from '@/lib/api/edgequake/members';
import { ROLE_ICON } from '@/lib/members/role-meta';
import { cn } from '@/lib/utils';
import { Check, ChevronDown, ShieldCheck } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { RoleBadge } from './role-badge';
import { useRoleDescription, useRoleLabel } from './use-role-label';

export interface RoleSelectProps {
  value: OrgRole;
  options: OrgRole[];
  onChange: (role: OrgRole) => void;
  /** Translated reason shown on hover when the control is locked. */
  lockedReason?: string | null;
  /** IdP slug when the role is re-synced at every sign-in. */
  managedBy?: string | null;
  disabled?: boolean;
  testId?: string;
}

/** Role pill that opens a menu of assignable roles (each with its one-line description). */
export function RoleSelect({ value, options, onChange, lockedReason, managedBy, disabled, testId }: RoleSelectProps) {
  const { t } = useTranslation();
  const label = useRoleLabel();
  const describe = useRoleDescription();
  const locked = !!lockedReason || disabled;

  if (locked) {
    const pill = (
      <span className="inline-flex" data-testid={testId} data-locked="true">
        <RoleBadge role={value} />
      </span>
    );
    if (!lockedReason) return pill;
    return (
      <Tooltip>
        <TooltipTrigger asChild>
          <span tabIndex={0} className="inline-flex rounded-full focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring">
            {pill}
          </span>
        </TooltipTrigger>
        <TooltipContent>{lockedReason}</TooltipContent>
      </Tooltip>
    );
  }

  return (
    <DropdownMenu>
      <DropdownMenuTrigger
        data-testid={testId}
        aria-label={t('members.role.change', 'Change role (currently {{role}})', { role: label(value) })}
        className="group inline-flex items-center gap-1 rounded-full focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
      >
        <RoleBadge role={value} className="transition-colors group-hover:border-foreground/30" />
        <ChevronDown className="size-3 text-muted-foreground group-data-[state=open]:rotate-180" aria-hidden />
      </DropdownMenuTrigger>
      <DropdownMenuContent align="start" className="w-72">
        <DropdownMenuLabel className="text-xs font-normal text-muted-foreground">
          {t('members.role.menuTitle', 'Organization role')}
        </DropdownMenuLabel>
        {options.map((r) => {
          const Icon = ROLE_ICON[r];
          return (
            <DropdownMenuItem
              key={r}
              data-testid={`role-option-${r}`}
              onSelect={() => r !== value && onChange(r)}
              className="items-start gap-2.5 py-2"
            >
              <Icon className="mt-0.5 size-4 text-muted-foreground" aria-hidden />
              <span className="min-w-0 flex-1">
                <span className="block text-sm font-medium">{label(r)}</span>
                <span className="block text-xs text-muted-foreground">{describe(r)}</span>
              </span>
              <Check className={cn('mt-0.5 size-4', r === value ? 'opacity-100' : 'opacity-0')} aria-hidden />
            </DropdownMenuItem>
          );
        })}
        {managedBy && (
          <>
            <DropdownMenuSeparator />
            <p className="flex items-start gap-1.5 px-2 py-1.5 text-xs text-muted-foreground">
              <ShieldCheck className="mt-0.5 size-3.5 shrink-0 text-teal-600" aria-hidden />
              {t(
                'members.role.ssoPinNote',
                'This role follows {{provider}}. Changing it here pins it, and the identity provider will stop overriding it.',
                { provider: managedBy },
              )}
            </p>
          </>
        )}
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
