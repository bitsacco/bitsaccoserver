import { Request } from 'express';
import {
  ServiceRole,
  GroupMembership,
  PermissionScope,
  Permission,
} from './permissions.types';

export interface JwtPayload {
  sub: string;
  email: string;
  email_verified?: boolean;
  given_name?: string;
  family_name?: string;
  preferred_username?: string;
  name?: string;
  iat?: number;
  exp?: number;
  aud?: string;
  iss?: string;
}

/**
 * Enhanced authenticated member with dual-scope context
 */
export interface AuthenticatedMember {
  // Basic member info
  memberId: string;
  sub: string; // JWT subject identifier (same as memberId but from token)
  email: string;
  authMethod: 'jwt' | 'api-key';

  // Service-level permissions
  serviceRole: ServiceRole;
  servicePermissions: Permission[];

  // Current context
  currentOrganizationId?: string;
  currentChamaId?: string;
  currentScope: PermissionScope;

  // All group memberships
  groupMemberships: GroupMembership[];

  // Resolved permissions for current context
  contextPermissions: Permission[];

  // Legacy alias for contextPermissions (for backward compatibility)
  permissions?: Permission[];
}

export interface AuthenticatedRequest extends Request {
  member: AuthenticatedMember;
  organizationId?: string;
  chamaId?: string;
  scope: PermissionScope;
  apiKeyId?: string; // For API key authentication
}

export interface AuthMetricData {
  action: string;
  memberId?: string;
  success: boolean;
  duration: number;
  errorType?: string;
}
