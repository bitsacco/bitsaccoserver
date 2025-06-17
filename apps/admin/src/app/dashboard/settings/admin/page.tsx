import * as React from 'react';
import type { Metadata } from 'next';
import { AdminSettingsDashboard } from '@/components/dashboard/admin-settings/admin-settings-dashboard';
import { PermissionGuard } from '@/components/auth/permission-guard';
import { ServiceRole } from '@bitsaccoserver/types';

import { config } from '@/config';

export const metadata = {
  title: `Admin Settings | Dashboard | ${config.site.name}`,
} satisfies Metadata;

export default function Page(): React.JSX.Element {
  return (
    <PermissionGuard requiredRole={ServiceRole.SYSTEM_ADMIN}>
      <AdminSettingsDashboard />
    </PermissionGuard>
  );
}
