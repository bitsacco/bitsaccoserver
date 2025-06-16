import {
  ServiceRole,
  Permission,
  PermissionScope,
  GroupMembership,
} from './permissions';

export interface JwtPayload {
  sub: string;
  email: string;
  firstName?: string;
  lastName?: string;
  emailVerified?: boolean;
  serviceRole?: ServiceRole;
  authMethod?: 'keycloak' | 'jwt' | 'api-key';
  iat?: number;
  exp?: number;
  aud?: string;
  iss?: string;
}

export interface LoginRequest {
  email: string;
  password: string;
}

export interface LoginResponse {
  access_token: string;
  refresh_token: string;
  expires_in: number;
  token_type: string;
  member: {
    id: string;
    email: string;
    firstName?: string;
    lastName?: string;
    emailVerified: boolean;
    serviceRole: ServiceRole;
  };
}

export interface RefreshTokenRequest {
  refresh_token: string;
}

export interface RegisterRequest {
  email: string;
  password: string;
  firstName: string;
  lastName: string;
  phoneNumber?: string;
}

export interface RegisterResponse {
  message: string;
  memberId: string;
}

export interface AuthenticatedUser {
  memberId: string;
  sub: string;
  email: string;
  firstName?: string;
  lastName?: string;
  emailVerified?: boolean;
  serviceRole: ServiceRole;
  authMethod: 'keycloak' | 'jwt' | 'api-key';

  // Context permissions
  servicePermissions: Permission[];
  currentOrganizationId?: string;
  currentChamaId?: string;
  currentScope: PermissionScope;
  groupMemberships: GroupMembership[];
  contextPermissions: Permission[];
}

export interface PasswordResetRequest {
  email: string;
}

export interface PasswordChangeRequest {
  currentPassword: string;
  newPassword: string;
}

export interface EmailVerificationRequest {
  token: string;
}
