import * as React from 'react';
import type { Metadata } from 'next';
import { HealthDashboard } from '@/components/dashboard/health/health-dashboard';

import { config } from '@/config';

export const metadata = {
  title: `System Health | Dashboard | ${config.site.name}`,
} satisfies Metadata;

export default function Page(): React.JSX.Element {
  return <HealthDashboard />;
}
