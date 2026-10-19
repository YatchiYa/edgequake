'use client';

import { InviteAccept } from '@/components/members/invite-accept';
import { useParams } from 'next/navigation';

export default function InvitePage() {
  const { token } = useParams<{ token: string }>();
  return <InviteAccept token={token} />;
}
