import { describe, expect, it } from 'vitest';

import {
  assignableRoles,
  canEditMember,
  isLastOwner,
  membersInScope,
  type MemberRole,
  type TenantMember,
} from '../members';

const m = (user_id: string, role: MemberRole, workspace_id: string | null = null): TenantMember => ({
  membership_id: `${user_id}-${workspace_id ?? 'all'}`,
  user_id,
  username: user_id,
  email: null,
  role,
  workspace_id,
  workspace_name: null,
  is_active: true,
  joined_at: '2026-01-01T00:00:00Z',
});

describe('member rules', () => {
  it('last owner counts only other tenant-wide owners', () => {
    const owner = m('dave', 'owner');
    const wsOwner = m('eve', 'owner', 'ws-x');
    const members = [owner, wsOwner, m('carol', 'admin')];
    expect(isLastOwner(members, owner)).toBe(true);
    expect(isLastOwner(members, wsOwner)).toBe(false);
    expect(isLastOwner([...members, m('frank', 'owner')], owner)).toBe(false);
  });

  it('tenant admins cannot grant or touch owners', () => {
    expect(assignableRoles(false)).not.toContain('owner');
    expect(assignableRoles(true)).toContain('owner');
    expect(canEditMember(m('dave', 'owner'), false)).toBe(false);
    expect(canEditMember(m('bob', 'member'), false)).toBe(true);
  });

  it('workspace scope includes tenant-wide members', () => {
    const members = [m('alice', 'member', 'ws-x'), m('bob', 'member', 'ws-y'), m('carol', 'admin')];
    expect(membersInScope(members, 'all')).toHaveLength(3);
    expect(membersInScope(members, 'ws-x').map((x) => x.user_id)).toEqual(['alice', 'carol']);
  });
});
