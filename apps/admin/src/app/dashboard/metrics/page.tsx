import * as React from 'react';
import type { Metadata } from 'next';
import { AdminMetricsDashboard } from '@/components/dashboard/admin-metrics/admin-metrics-dashboard';

import { config } from '@/config';

export const metadata = {
  title: `Admin Metrics | Dashboard | ${config.site.name}`,
} satisfies Metadata;

export default function Page(): React.JSX.Element {
  return <AdminMetricsDashboard />;
}
