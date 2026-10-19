'use client';

import { Avatar, AvatarFallback } from '@/components/ui/avatar';
import { avatarTone, displayName, initials } from '@/lib/members/format';
import { cn } from '@/lib/utils';
import { MailPlus } from 'lucide-react';

/** Initials on a stable tint; a pending invitation shows an envelope on a dashed ring. */
export function MemberAvatar({
  member,
  className,
}: {
  member: { username?: string | null; email: string; kind: 'member' | 'invitation' };
  className?: string;
}) {
  const name = displayName(member);
  if (member.kind === 'invitation') {
    return (
      <Avatar className={cn('size-9 border border-dashed border-muted-foreground/40', className)}>
        <AvatarFallback className="bg-transparent text-muted-foreground">
          <MailPlus className="size-4" aria-hidden />
        </AvatarFallback>
      </Avatar>
    );
  }
  return (
    <Avatar className={cn('size-9', className)}>
      <AvatarFallback className={cn('text-xs font-semibold', avatarTone(member.email))}>
        {initials(name)}
      </AvatarFallback>
    </Avatar>
  );
}
