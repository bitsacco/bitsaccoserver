import { Injectable } from '@nestjs/common';
import {
  ServiceRole,
  GroupRole,
  Permission,
  PermissionScope,
  GroupMembership,
  AuthenticatedMember,
  ROLE_PERMISSIONS,
  ROLE_HIERARCHY,
} from '..';

/**
 * Service for managing and resolving SACCO permissions
 */
@Injectable()
export class PermissionService {
  /**
   * Get all permissions for a service role
   */
  getServiceRolePermissions(role: ServiceRole): Permission[] {
    const permissions = new Set<Permission>();

    // Add direct permissions
    const directPermissions = ROLE_PERMISSIONS[role] || [];
    directPermissions.forEach((p) => permissions.add(p));

    // Add inherited permissions
    const inheritedRoles = ROLE_HIERARCHY[role] || [];
    inheritedRoles.forEach((inheritedRole) => {
      const inheritedPermissions = ROLE_PERMISSIONS[inheritedRole] || [];
      inheritedPermissions.forEach((p) => permissions.add(p));
    });

    return Array.from(permissions);
  }

  /**
   * Get all permissions for a group role
   */
  getGroupRolePermissions(role: GroupRole): Permission[] {
    const permissions = new Set<Permission>();

    // Add direct permissions
    const directPermissions = ROLE_PERMISSIONS[role] || [];
    directPermissions.forEach((p) => permissions.add(p));

    // Add inherited permissions
    const inheritedRoles = ROLE_HIERARCHY[role] || [];
    inheritedRoles.forEach((inheritedRole) => {
      const inheritedPermissions = ROLE_PERMISSIONS[inheritedRole] || [];
      inheritedPermissions.forEach((p) => permissions.add(p));
    });

    return Array.from(permissions);
  }

  /**
   * Check if a service role inherits from another service role
   */
  serviceRoleInheritsFrom(
    memberRole: ServiceRole,
    checkRole: ServiceRole,
  ): boolean {
    if (memberRole === checkRole) return true;

    const inheritedRoles = ROLE_HIERARCHY[memberRole] || [];
    return inheritedRoles.includes(checkRole);
  }

  /**
   * Check if a group role inherits from another group role
   */
  groupRoleInheritsFrom(memberRole: GroupRole, checkRole: GroupRole): boolean {
    if (memberRole === checkRole) return true;

    const inheritedRoles = ROLE_HIERARCHY[memberRole] || [];
    return inheritedRoles.includes(checkRole);
  }

  /**
   * Resolve all effective permissions for a member in a specific context
   */
  resolveUserPermissions(
    member: AuthenticatedMember,
    scope: PermissionScope,
    organizationId?: string,
    chamaId?: string,
  ): Permission[] {
    const permissions = new Set<Permission>();

    // Always include service-level permissions
    const servicePermissions = this.getServiceRolePermissions(
      member.serviceRole,
    );
    servicePermissions.forEach((p) => permissions.add(p));

    // Add context-specific permissions based on scope
    if (scope !== PermissionScope.GLOBAL) {
      const relevantMemberships = this.getRelevantMemberships(
        member.groupMemberships,
        scope,
        organizationId,
        chamaId,
      );

      relevantMemberships.forEach((membership) => {
        if (membership.isActive) {
          // Add group role permissions
          const groupPermissions = this.getGroupRolePermissions(
            membership.role,
          );
          groupPermissions.forEach((p) => permissions.add(p));

          // Add custom permissions
          membership.permissions.forEach((p) => permissions.add(p));
        }
      });
    }

    return Array.from(permissions);
  }

  /**
   * Get relevant group memberships for a specific scope
   */
  private getRelevantMemberships(
    memberships: GroupMembership[],
    scope: PermissionScope,
    organizationId?: string,
    chamaId?: string,
  ): GroupMembership[] {
    return memberships.filter((membership) => {
      if (scope === PermissionScope.ORGANIZATION) {
        return (
          membership.groupId === organizationId &&
          membership.groupType === 'organization'
        );
      }
      if (scope === PermissionScope.CHAMA) {
        return (
          membership.groupId === chamaId && membership.groupType === 'chama'
        );
      }
      return false;
    });
  }

  /**
   * Check if member has specific permission in given scope
   */
  memberHasPermission(
    member: AuthenticatedMember,
    permission: Permission,
    scope: PermissionScope,
    organizationId?: string,
    chamaId?: string,
  ): boolean {
    const memberPermissions = this.resolveUserPermissions(
      member,
      scope,
      organizationId,
      chamaId,
    );
    return memberPermissions.includes(permission);
  }

  /**
   * Check if member has all required permissions in given scope
   */
  memberHasAllPermissions(
    member: AuthenticatedMember,
    permissions: Permission[],
    scope: PermissionScope,
    organizationId?: string,
    chamaId?: string,
  ): boolean {
    const memberPermissions = this.resolveUserPermissions(
      member,
      scope,
      organizationId,
      chamaId,
    );
    return permissions.every((permission) =>
      memberPermissions.includes(permission),
    );
  }

  /**
   * Check if member has any of the required permissions in given scope
   */
  memberHasAnyPermission(
    member: AuthenticatedMember,
    permissions: Permission[],
    scope: PermissionScope,
    organizationId?: string,
    chamaId?: string,
  ): boolean {
    const memberPermissions = this.resolveUserPermissions(
      member,
      scope,
      organizationId,
      chamaId,
    );
    return permissions.some((permission) =>
      memberPermissions.includes(permission),
    );
  }

  /**
   * Get member's effective role in a specific context
   */
  getUserEffectiveRole(
    member: AuthenticatedMember,
    scope: PermissionScope,
    organizationId?: string,
    chamaId?: string,
  ): { serviceRole: ServiceRole; groupRole?: GroupRole } {
    const result = { serviceRole: member.serviceRole };

    if (scope !== PermissionScope.GLOBAL) {
      const relevantMemberships = this.getRelevantMemberships(
        member.groupMemberships,
        scope,
        organizationId,
        chamaId,
      );

      // Get the highest privilege role
      const activeRoles = relevantMemberships
        .filter((m) => m.isActive)
        .map((m) => m.role);

      if (activeRoles.length > 0) {
        // Sort roles by hierarchy and take the highest
        const sortedRoles = this.sortRolesByHierarchy(activeRoles);
        result['groupRole'] = sortedRoles[0];
      }
    }

    return result;
  }

  /**
   * Sort group roles by hierarchy (highest privilege first)
   */
  private sortRolesByHierarchy(roles: GroupRole[]): GroupRole[] {
    const roleHierarchyOrder = {
      [GroupRole.ORG_ADMIN]: 1,
      [GroupRole.CHAMA_ADMIN]: 2,
      [GroupRole.ORG_MEMBER]: 3,
      [GroupRole.CHAMA_MEMBER]: 4,
      [GroupRole.VIEWER]: 5,
    };

    return roles.sort((a, b) => roleHierarchyOrder[a] - roleHierarchyOrder[b]);
  }

  /**
   * Create a new group membership
   */
  createGroupMembership(
    memberId: string,
    groupId: string,
    groupType: 'organization' | 'chama',
    role: GroupRole,
    invitedBy?: string,
    customPermissions: Permission[] = [],
  ): GroupMembership {
    const basePermissions = this.getGroupRolePermissions(role);
    const allPermissions = [...basePermissions, ...customPermissions];

    return {
      groupId,
      groupType,
      role,
      permissions: allPermissions,
      scope:
        groupType === 'organization'
          ? PermissionScope.ORGANIZATION
          : PermissionScope.CHAMA,
      isActive: true,
      joinedAt: new Date(),
      invitedBy,
    };
  }

  /**
   * Validate permission compatibility
   * Ensures permissions are appropriate for the given scope
   */
  validatePermissionScope(
    permission: Permission,
    scope: PermissionScope,
  ): boolean {
    const systemPermissions = [
      Permission.SYSTEM_CONFIG,
      Permission.SYSTEM_MONITOR,
      Permission.SYSTEM_BACKUP,
    ];

    const organizationPermissions = [
      Permission.ORG_CREATE,
      Permission.ORG_READ,
      Permission.ORG_UPDATE,
      Permission.ORG_DELETE,
      Permission.ORG_SETTINGS,
    ];

    const chamaPermissions = [
      Permission.CHAMA_CREATE,
      Permission.CHAMA_READ,
      Permission.CHAMA_UPDATE,
      Permission.CHAMA_DELETE,
      Permission.CHAMA_INVITE,
    ];

    switch (scope) {
      case PermissionScope.GLOBAL:
        return systemPermissions.includes(permission);
      case PermissionScope.ORGANIZATION:
        return (
          organizationPermissions.includes(permission) ||
          systemPermissions.includes(permission)
        );
      case PermissionScope.CHAMA:
        return (
          chamaPermissions.includes(permission) ||
          organizationPermissions.includes(permission) ||
          systemPermissions.includes(permission)
        );
      case PermissionScope.PERSONAL:
        return true; // All permissions can be used in personal scope
      default:
        return false;
    }
  }
}
