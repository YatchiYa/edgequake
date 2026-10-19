'use client';

import { MembersPage } from '@/components/members/members-page';
import { Suspense } from 'react';

export default function Page() {
  return (
    <Suspense fallback={null}>
      <MembersPage />
    </Suspense>
  );
}
