'use client';

/**
 * @module workspace-members-card
 * @description Members of the current tenant: list (whole tenant or one workspace), add,
 * change role, remove. Actions the caller may not perform are hidden or disabled; the
 * server re-checks every rule (owner grants, last owner).
 */

import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
} from '@/components/ui/alert-dialog';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Checkbox } from '@/components/ui/checkbox';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table';
import { ApiRequestError } from '@/lib/api/client';
import {
  addTenantMember,
  assignableRoles,
  canEditMember,
  isLastOwner,
  listTenantMembers,
  membersInScope,
  removeTenantMember,
  updateTenantMember,
  type MemberRole,
} from '@/lib/api/members';
import { createUser, listUsers } from '@/lib/api/users';
import { useAuthStore } from '@/stores/use-auth-store';
import { useTenantStore } from '@/stores/use-tenant-store';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { Loader2, Plus, Trash2, Users } from 'lucide-react';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { toast } from 'sonner';

const ALL = 'all';
const errMsg = (e: unknown) => (e instanceof Error ? e.message : String(e));

export function WorkspaceMembersCard() {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const { selectedTenantId, selectedWorkspaceId, workspaces } = useTenantStore();
  const user = useAuthStore((s) => s.user);
  const isPlatformAdmin = user?.role === 'admin' || user?.roles?.includes('admin') || false;
  const [scope, setScope] = useState<string>(selectedWorkspaceId ?? ALL);
  const [adding, setAdding] = useState(false);

  const queryKey = ['tenant-members', selectedTenantId];
  const members = useQuery({
    queryKey,
    queryFn: () => listTenantMembers(selectedTenantId!),
    enabled: Boolean(selectedTenantId),
    retry: false,
  });
  const refresh = () => queryClient.invalidateQueries({ queryKey });

  const roleMutation = useMutation({
    mutationFn: ({ id, role }: { id: string; role: MemberRole }) =>
      updateTenantMember(selectedTenantId!, id, role),
    onSuccess: () => {
      toast.success(t('members.roleUpdated', 'Role updated'));
      refresh();
    },
    onError: (e) => toast.error(t('members.updateFailed', 'Update failed: {{error}}', { error: errMsg(e) })),
  });
  const removeMutation = useMutation({
    mutationFn: (id: string) => removeTenantMember(selectedTenantId!, id),
    onSuccess: () => {
      toast.success(t('members.removed', 'Member removed'));
      refresh();
    },
    onError: (e) => toast.error(t('members.removeFailed', 'Remove failed: {{error}}', { error: errMsg(e) })),
  });

  if (!selectedTenantId) return null;

  const forbidden = members.error instanceof ApiRequestError && members.error.status === 403;
  const canManageOwners = members.data?.can_manage_owners ?? false;
  const all = members.data?.items ?? [];
  const rows = membersInScope(all, scope);
  const roleLabel = (r: MemberRole) => t(`members.roles.${r}`, r);

  return (
    <Card data-testid="workspace-members-card">
      <CardHeader className="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
        <div className="space-y-1">
          <CardTitle className="flex items-center gap-2 text-base">
            <Users className="h-4 w-4" aria-hidden="true" />
            {t('members.title', 'Members')}
          </CardTitle>
          <CardDescription className="text-xs">
            {t('members.description', 'Who can access this tenant and its workspaces.')}
          </CardDescription>
        </div>
        {!forbidden && members.data && (
          <div className="flex items-center gap-2">
            <Select value={scope} onValueChange={setScope}>
              <SelectTrigger className="h-8 w-[200px] text-xs" aria-label={t('members.scope', 'Scope')}>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value={ALL}>{t('members.scopeTenant', 'Whole tenant')}</SelectItem>
                {workspaces.map((w) => (
                  <SelectItem key={w.id} value={w.id}>
                    {w.name}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            <Button size="sm" variant="outline" onClick={() => setAdding(true)}>
              <Plus className="mr-1 h-3.5 w-3.5" aria-hidden="true" />
              {t('members.add', 'Add member')}
            </Button>
          </div>
        )}
      </CardHeader>
      <CardContent className="px-0 pb-0">
        {forbidden ? (
          <p className="px-6 pb-6 text-sm text-muted-foreground">
            {t('members.forbidden', 'Only tenant owners/admins and platform admins can manage members.')}
          </p>
        ) : members.isLoading ? (
          <div className="flex justify-center py-8" role="status">
            <Loader2 className="h-5 w-5 animate-spin text-muted-foreground" aria-hidden="true" />
          </div>
        ) : members.error ? (
          <p className="px-6 pb-6 text-sm text-destructive">{errMsg(members.error)}</p>
        ) : rows.length === 0 ? (
          <p className="px-6 pb-6 text-sm text-muted-foreground">{t('members.empty', 'No members.')}</p>
        ) : (
          <div className="border-t">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead className="pl-6">{t('members.user', 'User')}</TableHead>
                  <TableHead>{t('members.scope', 'Scope')}</TableHead>
                  <TableHead className="w-[140px]">{t('members.role', 'Role')}</TableHead>
                  <TableHead className="w-[48px] pr-6">
                    <span className="sr-only">{t('members.actions', 'Actions')}</span>
                  </TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {rows.map((m) => {
                  const name = m.username ?? m.user_id;
                  const editable = canEditMember(m, canManageOwners);
                  const lastOwner = isLastOwner(all, m);
                  return (
                    <TableRow key={m.membership_id}>
                      <TableCell className="pl-6">
                        <div className="text-sm font-medium">{name}</div>
                        {m.email && <div className="text-xs text-muted-foreground">{m.email}</div>}
                      </TableCell>
                      <TableCell className="text-xs">
                        {m.workspace_id
                          ? (m.workspace_name ?? m.workspace_id)
                          : t('members.allWorkspaces', 'All workspaces')}
                      </TableCell>
                      <TableCell>
                        <Select
                          value={m.role}
                          disabled={!editable || lastOwner || roleMutation.isPending}
                          onValueChange={(role) =>
                            roleMutation.mutate({ id: m.membership_id, role: role as MemberRole })
                          }
                        >
                          <SelectTrigger
                            className="h-7 w-[120px] text-xs"
                            aria-label={t('members.roleFor', 'Role for {{name}}', { name })}
                          >
                            <SelectValue />
                          </SelectTrigger>
                          <SelectContent>
                            {/* Keep the current role selectable even if the caller cannot grant it. */}
                            {[...new Set<MemberRole>([...assignableRoles(canManageOwners), m.role])].map((r) => (
                              <SelectItem key={r} value={r} className="text-xs">
                                {roleLabel(r)}
                              </SelectItem>
                            ))}
                          </SelectContent>
                        </Select>
                      </TableCell>
                      <TableCell className="pr-6 text-right">
                        <RemoveButton
                          name={name}
                          disabled={!editable || lastOwner}
                          title={lastOwner ? t('members.lastOwner', 'The last owner cannot be removed') : undefined}
                          onConfirm={() => removeMutation.mutate(m.membership_id)}
                        />
                      </TableCell>
                    </TableRow>
                  );
                })}
              </TableBody>
            </Table>
          </div>
        )}
      </CardContent>
      {adding && (
        <AddMemberDialog
          tenantId={selectedTenantId}
          defaultWorkspace={scope}
          canManageOwners={canManageOwners}
          isPlatformAdmin={isPlatformAdmin}
          onClose={() => setAdding(false)}
          onAdded={refresh}
        />
      )}
    </Card>
  );
}

function RemoveButton({
  name,
  disabled,
  title,
  onConfirm,
}: {
  name: string;
  disabled: boolean;
  title?: string;
  onConfirm: () => void;
}) {
  const { t } = useTranslation();
  return (
    <AlertDialog>
      <AlertDialogTrigger asChild>
        <Button
          variant="ghost"
          size="icon"
          className="h-7 w-7 text-muted-foreground hover:text-destructive"
          disabled={disabled}
          title={title}
          aria-label={t('members.removeLabel', 'Remove {{name}}', { name })}
        >
          <Trash2 className="h-3.5 w-3.5" aria-hidden="true" />
        </Button>
      </AlertDialogTrigger>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>{t('members.removeTitle', 'Remove member?')}</AlertDialogTitle>
          <AlertDialogDescription>
            {t('members.removeConfirm', '{{name}} immediately loses access granted by this membership.', { name })}
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel>{t('common.cancel', 'Cancel')}</AlertDialogCancel>
          <AlertDialogAction
            onClick={onConfirm}
            className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
          >
            {t('members.remove', 'Remove')}
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}

function AddMemberDialog({
  tenantId,
  defaultWorkspace,
  canManageOwners,
  isPlatformAdmin,
  onClose,
  onAdded,
}: {
  tenantId: string;
  defaultWorkspace: string;
  canManageOwners: boolean;
  isPlatformAdmin: boolean;
  onClose: () => void;
  onAdded: () => void;
}) {
  const { t } = useTranslation();
  const workspaces = useTenantStore((s) => s.workspaces);
  const [login, setLogin] = useState('');
  const [role, setRole] = useState<MemberRole>('member');
  const [workspace, setWorkspace] = useState(defaultWorkspace);
  const [createNew, setCreateNew] = useState(false);
  const [newUser, setNewUser] = useState({ username: '', email: '', password: '' });

  // Suggestions for platform admins (GET /users is admin-only).
  const users = useQuery({
    queryKey: ['users', 'member-picker'],
    queryFn: () => listUsers(1, 100),
    enabled: isPlatformAdmin,
    retry: false,
  });

  const submit = useMutation({
    mutationFn: async () => {
      let user = login.trim();
      if (createNew) {
        await createUser({ ...newUser, role: 'user' });
        user = newUser.username.trim();
      }
      return addTenantMember(tenantId, {
        user,
        role,
        workspace_id: workspace === ALL ? null : workspace,
      });
    },
    onSuccess: () => {
      toast.success(t('members.added', 'Member added'));
      onAdded();
      onClose();
    },
    onError: (e) => toast.error(t('members.addFailed', 'Add failed: {{error}}', { error: errMsg(e) })),
  });

  const valid = createNew
    ? Boolean(newUser.username.trim() && newUser.email.trim() && newUser.password)
    : Boolean(login.trim());

  return (
    <Dialog open onOpenChange={(open) => !open && onClose()}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{t('members.add', 'Add member')}</DialogTitle>
          <DialogDescription>
            {t('members.addDescription', 'Grant an existing user access to the whole tenant or to one workspace.')}
          </DialogDescription>
        </DialogHeader>
        <form
          className="grid gap-3"
          onSubmit={(e) => {
            e.preventDefault();
            if (valid && !submit.isPending) submit.mutate();
          }}
        >
          {isPlatformAdmin && (
            <div className="flex items-center gap-2">
              <Checkbox
                id="member-create-user"
                checked={createNew}
                onCheckedChange={(v) => setCreateNew(v === true)}
              />
              <Label htmlFor="member-create-user" className="text-xs font-normal">
                {t('members.createUser', 'Create a new user')}
              </Label>
            </div>
          )}
          {createNew ? (
            <>
              <Field id="member-new-username" label={t('members.username', 'Username')} value={newUser.username}
                onChange={(username) => setNewUser((u) => ({ ...u, username }))} />
              <Field id="member-new-email" type="email" label={t('members.email', 'Email')} value={newUser.email}
                onChange={(email) => setNewUser((u) => ({ ...u, email }))} />
              <Field id="member-new-password" type="password" autoComplete="new-password"
                label={t('members.password', 'Password')} value={newUser.password}
                onChange={(password) => setNewUser((u) => ({ ...u, password }))} />
            </>
          ) : (
            <>
              <Field id="member-login" label={t('members.userLogin', 'Username or email')} value={login}
                onChange={setLogin} list="member-user-options" />
              <datalist id="member-user-options">
                {users.data?.users.map((u) => (
                  <option key={u.user_id} value={u.username}>
                    {u.email}
                  </option>
                ))}
              </datalist>
            </>
          )}
          <div className="grid gap-1">
            <Label className="text-xs">{t('members.role', 'Role')}</Label>
            <Select value={role} onValueChange={(v) => setRole(v as MemberRole)}>
              <SelectTrigger className="h-8 text-sm">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {assignableRoles(canManageOwners).map((r) => (
                  <SelectItem key={r} value={r}>
                    {t(`members.roles.${r}`, r)}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
          <div className="grid gap-1">
            <Label className="text-xs">{t('members.scope', 'Scope')}</Label>
            <Select value={workspace} onValueChange={setWorkspace}>
              <SelectTrigger className="h-8 text-sm">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value={ALL}>{t('members.allWorkspaces', 'All workspaces')}</SelectItem>
                {workspaces.map((w) => (
                  <SelectItem key={w.id} value={w.id}>
                    {w.name}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
          <DialogFooter>
            <Button type="button" variant="outline" size="sm" onClick={onClose}>
              {t('common.cancel', 'Cancel')}
            </Button>
            <Button type="submit" size="sm" disabled={!valid || submit.isPending}>
              {submit.isPending && <Loader2 className="mr-1.5 h-3.5 w-3.5 animate-spin" aria-hidden="true" />}
              {t('members.add', 'Add member')}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}

function Field({
  id,
  label,
  value,
  onChange,
  type = 'text',
  autoComplete = 'off',
  list,
}: {
  id: string;
  label: string;
  value: string;
  onChange: (v: string) => void;
  type?: string;
  autoComplete?: string;
  list?: string;
}) {
  return (
    <div className="grid gap-1">
      <Label htmlFor={id} className="text-xs">
        {label}
      </Label>
      <Input
        id={id}
        type={type}
        value={value}
        list={list}
        autoComplete={autoComplete}
        className="h-8 text-sm"
        onChange={(e) => onChange(e.target.value)}
      />
    </div>
  );
}
