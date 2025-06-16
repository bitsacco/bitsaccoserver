import { ServiceRole, GroupMembership, Permission } from './permissions';

export interface User {
  id: string;
  email: string;
  firstName?: string;
  lastName?: string;
  name?: string;
  avatar?: string;
  phone?: string;
  npub?: string;
  emailVerified?: boolean;
  serviceRole: ServiceRole;
  groupMemberships?: GroupMembership[];
  createdAt?: Date;
  updatedAt?: Date;

  // Legacy/compatibility fields
  profilePicture?: string;
  phoneNumber?: string;

  [key: string]: unknown;
}

export interface Member extends User {
  // Additional member-specific fields
  address?: {
    street?: string;
    city?: string;
    state?: string;
    country?: string;
  };

  // Business-specific fields
  organizationId?: string;
  chamaIds?: string[];
  membershipNumber?: string;
  joinedAt?: Date;
  status?: 'active' | 'inactive' | 'suspended';
}

export interface CreateMemberRequest {
  email: string;
  firstName: string;
  lastName: string;
  phone?: string;
  serviceRole?: ServiceRole;
  organizationId?: string;
}

export interface UpdateMemberRequest {
  firstName?: string;
  lastName?: string;
  phone?: string;
  avatar?: string;
  serviceRole?: ServiceRole;
  status?: 'active' | 'inactive' | 'suspended';
}

export interface MemberListParams {
  page?: number;
  limit?: number;
  search?: string;
  serviceRole?: ServiceRole;
  organizationId?: string;
  chamaId?: string;
  status?: 'active' | 'inactive' | 'suspended';
  sortBy?: 'name' | 'email' | 'createdAt' | 'updatedAt';
  sortOrder?: 'asc' | 'desc';
}

export interface MemberListResponse {
  members: Member[];
  total: number;
  page: number;
  limit: number;
  totalPages: number;
}

// Helper function types
export type RoleChecker = (user: User | null, role: ServiceRole) => boolean;
export type PermissionChecker = (
  user: User | null,
  permission: Permission,
) => boolean;
