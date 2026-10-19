'use client';

import { AccessLinkPage } from '@/components/members/access-link-page';
import { useParams } from 'next/navigation';

export default function AccessPage() {
  const { token } = useParams<{ token: string }>();
  return <AccessLinkPage token={token} />;
}
