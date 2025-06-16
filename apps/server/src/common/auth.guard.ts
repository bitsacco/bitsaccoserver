import {
  Injectable,
  ExecutionContext,
  UnauthorizedException,
  ForbiddenException,
  CanActivate,
} from '@nestjs/common';
import { Reflector } from '@nestjs/core';
import { Observable } from 'rxjs';
import {
  AuthenticatedRequest,
  AuthenticatedMember,
  Permission,
  PermissionScope,
  ServiceRole,
  GroupRole,
  GroupMembership,
  ROLE_PERMISSIONS,
  ROLE_HIERARCHY,
} from './types';

/**
 * Enhanced SACCO authentication and authorization guard
 * Supports dual-scope permissions: service-level and group-level
 */
@Injectable()
export class AuthGuard implements CanActivate {
  constructor(private reflector: Reflector) {}

  canActivate(
    context: ExecutionContext,
  ): boolean | Promise<boolean> | Observable<boolean> {
    const request = context.switchToHttp().getRequest<AuthenticatedRequest>();
    const member = request.member;

    if (!member) {
      throw new UnauthorizedException('Authentication required');
    }

    // Get required permissions from decorator
    const requiredPermissions =
      this.reflector.getAllAndOverride<Permission[]>('permissions', [
        context.getHandler(),
        context.getClass(),
      ]) || [];

    // Get required scope from decorator
    const requiredScope =
      this.reflector.getAllAndOverride<PermissionScope>('scope', [
        context.getHandler(),
        context.getClass(),
      ]) || PermissionScope.GLOBAL;

    // Get required role from decorator
    const requiredRole = this.reflector.getAllAndOverride<
      ServiceRole | GroupRole
    >('role', [context.getHandler(), context.getClass()]);

    // Resolve member context and permissions
    const enhancedUser = this.enhanceUserContext(member, request);
    request.member = enhancedUser;

    // Check role requirement
    if (
      requiredRole &&
      !this.hasRole(enhancedUser, requiredRole, requiredScope)
    ) {
      throw new ForbiddenException(
        `Insufficient role: ${requiredRole} required for ${requiredScope} scope`,
      );
    }

    // Check permission requirements
    if (requiredPermissions.length > 0) {
      const hasPermissions = this.hasPermissions(
        enhancedUser,
        requiredPermissions,
        requiredScope,
      );

      if (!hasPermissions) {
        throw new ForbiddenException(
          `Insufficient permissions: ${requiredPermissions.join(', ')} required`,
        );
      }
    }

    return true;
  }

  /**
   * Enhance member context with resolved permissions and current scope
   */
  private enhanceUserContext(
    member: any,
    request: AuthenticatedRequest,
  ): AuthenticatedMember {
    // Determine current context from request
    const organizationId =
      request.params?.organizationId ||
      (request.query?.organizationId as string);
    const chamaId =
      request.params?.chamaId || (request.query?.chamaId as string);

    // Determine current scope
    let currentScope = PermissionScope.GLOBAL;
    if (chamaId) {
      currentScope = PermissionScope.CHAMA;
    } else if (organizationId) {
      currentScope = PermissionScope.ORGANIZATION;
    }

    // Mock group memberships (would be loaded from database in real implementation)
    const groupMemberships: GroupMembership[] = member.groupMemberships || [];

    // Resolve permissions for current context
    const contextPermissions = this.resolveContextPermissions(
      member.serviceRole,
      groupMemberships,
      currentScope,
      organizationId,
      chamaId,
    );

    return {
      memberId: member.sub || member.memberId,
      sub: member.sub || member.memberId,
      email: member.email,
      authMethod: member.authMethod,
      serviceRole: member.serviceRole || ServiceRole.MEMBER,
      servicePermissions: ROLE_PERMISSIONS[member.serviceRole] || [],
      currentOrganizationId: organizationId,
      currentChamaId: chamaId,
      currentScope,
      groupMemberships,
      contextPermissions,
      permissions: contextPermissions, // Legacy alias
    };
  }

  /**
   * Resolve permissions for the current context
   */
  private resolveContextPermissions(
    serviceRole: ServiceRole,
    groupMemberships: GroupMembership[],
    scope: PermissionScope,
    organizationId?: string,
    chamaId?: string,
  ): Permission[] {
    const permissions = new Set<Permission>();

    // Add service-level permissions (always available)
    const servicePermissions = ROLE_PERMISSIONS[serviceRole] || [];
    servicePermissions.forEach((p) => permissions.add(p));

    // Add context-specific permissions
    if (scope === PermissionScope.GLOBAL) {
      // Global scope: only service permissions
      return Array.from(permissions);
    }

    // Find relevant group memberships for current context
    const relevantMemberships = groupMemberships.filter((membership) => {
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

    // Add permissions from group memberships
    relevantMemberships.forEach((membership) => {
      if (membership.isActive) {
        // Add direct role permissions
        const rolePermissions = ROLE_PERMISSIONS[membership.role] || [];
        rolePermissions.forEach((p) => permissions.add(p));

        // Add inherited permissions from role hierarchy
        const inheritedRoles = ROLE_HIERARCHY[membership.role] || [];
        inheritedRoles.forEach((inheritedRole) => {
          const inheritedPermissions = ROLE_PERMISSIONS[inheritedRole] || [];
          inheritedPermissions.forEach((p) => permissions.add(p));
        });

        // Add custom permissions
        membership.permissions.forEach((p) => permissions.add(p));
      }
    });

    return Array.from(permissions);
  }

  /**
   * Check if member has required role in the specified scope
   */
  private hasRole(
    member: AuthenticatedMember,
    requiredRole: ServiceRole | GroupRole,
    scope: PermissionScope,
  ): boolean {
    // Check service-level role
    if (Object.values(ServiceRole).includes(requiredRole as ServiceRole)) {
      return this.hasServiceRole(
        member.serviceRole,
        requiredRole as ServiceRole,
      );
    }

    // Check group-level role
    if (scope === PermissionScope.GLOBAL) {
      return false; // Group roles don't apply to global scope
    }

    return member.groupMemberships.some((membership) => {
      if (!membership.isActive) return false;

      // Check if this membership is relevant for current scope
      const isRelevant =
        (scope === PermissionScope.ORGANIZATION &&
          membership.groupId === member.currentOrganizationId &&
          membership.groupType === 'organization') ||
        (scope === PermissionScope.CHAMA &&
          membership.groupId === member.currentChamaId &&
          membership.groupType === 'chama');

      if (!isRelevant) return false;

      // Check direct role match
      if (membership.role === requiredRole) return true;

      // Check role hierarchy
      const inheritedRoles = ROLE_HIERARCHY[membership.role] || [];
      return inheritedRoles.includes(requiredRole as GroupRole);
    });
  }

  /**
   * Check service-level role hierarchy
   */
  private hasServiceRole(
    memberRole: ServiceRole,
    requiredRole: ServiceRole,
  ): boolean {
    if (memberRole === requiredRole) return true;

    const inheritedRoles = ROLE_HIERARCHY[memberRole] || [];
    return inheritedRoles.includes(requiredRole);
  }

  /**
   * Check if member has all required permissions in the specified scope
   */
  private hasPermissions(
    member: AuthenticatedMember,
    requiredPermissions: Permission[],
    _scope: PermissionScope,
  ): boolean {
    return requiredPermissions.every((permission) =>
      member.contextPermissions.includes(permission),
    );
  }
}

/**
 * Decorators for permission-based access control
 */
import { SetMetadata } from '@nestjs/common';

export const RequirePermissions = (...permissions: Permission[]) =>
  SetMetadata('permissions', permissions);

export const RequireRole = (role: ServiceRole | GroupRole) =>
  SetMetadata('role', role);

export const RequireScope = (scope: PermissionScope) =>
  SetMetadata('scope', scope);

/**
 * Combined decorator for common permission patterns
 */
export const SACCOAuth = (
  permissions: Permission[] = [],
  role?: ServiceRole | GroupRole,
  scope: PermissionScope = PermissionScope.GLOBAL,
) => {
  return (
    target: any,
    propertyKey?: string,
    descriptor?: PropertyDescriptor,
  ) => {
    SetMetadata('permissions', permissions)(target, propertyKey, descriptor);
    if (role) {
      SetMetadata('role', role)(target, propertyKey, descriptor);
    }
    SetMetadata('scope', scope)(target, propertyKey, descriptor);
  };
};
