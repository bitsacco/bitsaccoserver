import type { NavItemConfig } from '@/types/nav';
import { ServiceRole } from '@bitsaccoserver/types';
import { paths } from '@/paths';

export const navItems = [
  {
    key: 'overview',
    title: 'System Overview',
    href: paths.dashboard.overview,
    icon: 'home',
    allowedRoles: [ServiceRole.ADMIN, ServiceRole.SYSTEM_ADMIN],
  },
  {
    key: 'admin-management',
    title: 'Admin Management',
    href: paths.dashboard.adminManagement,
    icon: 'shield',
    requiredRole: ServiceRole.SYSTEM_ADMIN,
  },
  {
    key: 'member-management',
    title: 'Member Management',
    href: paths.dashboard.memberManagement,
    icon: 'users',
    allowedRoles: [ServiceRole.ADMIN, ServiceRole.SYSTEM_ADMIN],
  },
  // {
  //   key: 'metrics',
  //   title: 'Metrics & Analytics',
  //   href: '/dashboard/metrics',
  //   icon: 'chart',
  //   allowedRoles: [ServiceRole.ADMIN, ServiceRole.SYSTEM_ADMIN],
  // },
  {
    key: 'settings',
    title: 'Settings',
    href: paths.dashboard.settings,
    icon: 'gear',
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
    separator: 'before',
  },
] satisfies NavItemConfig[];
