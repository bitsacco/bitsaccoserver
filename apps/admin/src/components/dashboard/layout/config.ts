import type { NavItemConfig } from '@/types/nav';
import { ServiceRole } from '@bitsaccoserver/types';
import { paths } from '@/paths';

export const navItems = [
  {
    key: 'overview',
    title: 'Overview',
    href: paths.dashboard.overview,
    icon: 'chart-line',
    allowedRoles: [ServiceRole.ADMIN, ServiceRole.SYSTEM_ADMIN],
  },
  {
    key: 'membership',
    title: 'Member Management',
    href: paths.dashboard.membership,
    icon: 'users',
    allowedRoles: [ServiceRole.ADMIN, ServiceRole.SYSTEM_ADMIN],
  },
  {
    key: 'metrics',
    title: 'Metrics & Analytics',
    href: '/dashboard/metrics',
    icon: 'chart-bar',
    allowedRoles: [ServiceRole.ADMIN, ServiceRole.SYSTEM_ADMIN],
  },
  {
    key: 'health',
    title: 'System Health',
    href: '/dashboard/health',
    icon: 'heart',
    allowedRoles: [ServiceRole.ADMIN, ServiceRole.SYSTEM_ADMIN],
  },
  {
    key: 'settings',
    title: 'Settings',
    href: paths.dashboard.settings,
    icon: 'gear-six',
    items: [
      {
        key: 'general-settings',
        title: 'General',
        href: paths.dashboard.settings,
        allowedRoles: [ServiceRole.ADMIN, ServiceRole.SYSTEM_ADMIN],
      },
      {
        key: 'system-config',
        title: 'System Configuration',
        href: '/dashboard/settings/system',
        requiredRole: ServiceRole.SYSTEM_ADMIN,
      },
      {
        key: 'integrations',
        title: 'Integrations',
        href: '/dashboard/settings/integrations',
        allowedRoles: [ServiceRole.ADMIN, ServiceRole.SYSTEM_ADMIN],
      },
    ],
  },
  {
    key: 'account',
    title: 'Account',
    href: paths.dashboard.account,
    icon: 'user',
    allowedRoles: [ServiceRole.ADMIN, ServiceRole.SYSTEM_ADMIN],
  },
] satisfies NavItemConfig[];
