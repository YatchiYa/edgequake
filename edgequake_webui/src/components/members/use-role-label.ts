'use client';

import type { OrgRole } from '@/lib/api/edgequake/members';
import { ROLE_LABEL_FALLBACK } from '@/lib/members/role-meta';
import { useTranslation } from 'react-i18next';

/** Translated organization-role label (`readonly` reads "Viewer"). */
export function useRoleLabel() {
  const { t } = useTranslation();
  return (role: OrgRole) => t(`members.roles.${role}.label`, ROLE_LABEL_FALLBACK[role]);
}

export function useRoleDescription() {
  const { t } = useTranslation();
  return (role: OrgRole, fallback = '') => t(`members.roles.${role}.description`, fallback);
}
