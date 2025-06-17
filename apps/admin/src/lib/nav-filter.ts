import { ServiceRole } from '@bitsaccoserver/types';
import type { NavItemConfig } from '@/types/nav';

/**
 * Check if user has access to a navigation item based on role requirements
 */
function hasNavItemAccess(
  item: NavItemConfig,
  userRole?: ServiceRole,
): boolean {
  // If no user role, deny access
  if (!userRole) {
    return false;
  }

  // If item requires a specific role, check exact match
  if (item.requiredRole) {
    return userRole === item.requiredRole;
  }

  // If item has allowed roles, check if user role is in the list
  if (item.allowedRoles) {
    return item.allowedRoles.includes(userRole);
  }

  // If no role restrictions, allow access
  return true;
}

/**
 * Filter navigation items based on user's service role
 */
export function filterNavItemsByRole(
  items: NavItemConfig[],
  userRole?: ServiceRole,
): NavItemConfig[] {
  return (
    items
      .filter((item) => hasNavItemAccess(item, userRole))
      .map((item) => ({
        ...item,
        // Recursively filter sub-items if they exist
        items: item.items
          ? filterNavItemsByRole(item.items, userRole)
          : undefined,
      }))
      // Remove items that have no accessible sub-items
      .filter((item) => !item.items || item.items.length > 0)
  );
}
