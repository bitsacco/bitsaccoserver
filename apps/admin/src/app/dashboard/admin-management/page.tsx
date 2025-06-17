import * as React from 'react';
import type { Metadata } from 'next';
import { AdminManagementDashboard } from '@/components/dashboard/admin-management/admin-management-dashboard';
import { PermissionGuard } from '@/components/auth/permission-guard';
import { ServiceRole } from '@bitsaccoserver/types';

import { config } from '@/config';

export const metadata = {
  title: `Admin Management | Dashboard | ${config.site.name}`,
} satisfies Metadata;

export default function Page(): React.JSX.Element {
  return (
    <PermissionGuard requiredRole={ServiceRole.SYSTEM_ADMIN}>
      <AdminManagementDashboard />
    </PermissionGuard>
  );
}
