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
 * Enhanced authenticated user with dual-scope context
 */
export interface AuthenticatedUser {
  // Basic user info
  userId: string;
  sub: string; // JWT subject identifier (same as userId but from token)
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
  user: AuthenticatedUser;
  organizationId?: string;
  chamaId?: string;
  scope: PermissionScope;
  apiKeyId?: string; // For API key authentication
}

export interface AuthMetricData {
  action: string;
  userId?: string;
  success: boolean;
  duration: number;
  errorType?: string;
}
