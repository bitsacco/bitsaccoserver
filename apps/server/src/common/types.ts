import { Request } from 'express';
import { OrganizationMemberDocument } from './schemas';

export enum UserRole {
  DEVELOPER = 'developer',
  ADMIN = 'admin',
}

export interface AuthenticatedUser {
  sub: string;
  email: string;
  authMethod: 'jwt' | 'api-key';
  permissions?: string[];
  // Legacy compatibility fields
  userId?: string;
  keyId?: string;
  roles?: string[];
  organizationId?: string;
}

export interface AuthenticatedRequest extends Request {
  user: AuthenticatedUser;
  organizationId?: string;
  apiKeyId?: string;
  organizationMembership?: OrganizationMemberDocument;
}

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

export enum ServiceStatus {
  ACTIVE = 'active',
  DEPRECATED = 'deprecated',
  MAINTENANCE = 'maintenance',
}
