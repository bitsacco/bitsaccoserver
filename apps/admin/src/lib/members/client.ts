'use client';

import {
  User,
  Member,
  ServiceRole,
  MemberListParams,
  MemberListResponse,
  CreateMemberRequest,
  UpdateMemberRequest,
} from '@bitsaccoserver/types';
import { logger } from '@/lib/default-logger';
import { fetchWithAuth } from '@/lib/fetch-with-auth';

// Helper function to check if user has specific role
export function hasRole(user: User | null, role: ServiceRole): boolean {
  if (!user || !user.serviceRole) {
    return false;
  }
  return user.serviceRole === role;
}

// Helper function to check if user is a super admin
export function isSuperAdmin(user: User | null): boolean {
  if (!user) return false;
  return hasRole(user, ServiceRole.SYSTEM_ADMIN);
}

// Helper function to check if user is an admin (any admin level)
export function isAdmin(user: User | null): boolean {
  if (!user) return false;
  return (
    hasRole(user, ServiceRole.SYSTEM_ADMIN) || hasRole(user, ServiceRole.ADMIN)
  );
}

class MembersClient {
  private baseUrl = '/api/admin/members';

  async getMembers(params: MemberListParams = {}): Promise<MemberListResponse> {
    try {
      const searchParams = new URLSearchParams();

      // Add pagination - backend uses limit/offset instead of page/limit
      if (params.limit) searchParams.set('limit', params.limit.toString());
      if (params.page) {
        // Convert page to offset (page is 1-based, offset is 0-based)
        const offset = (params.page - 1) * (params.limit || 10);
        searchParams.set('offset', offset.toString());
      }

      // Add filters - backend uses 'role' instead of 'serviceRole'
      if (params.search) searchParams.set('search', params.search);
      if (params.serviceRole) searchParams.set('role', params.serviceRole);
      if (params.status) searchParams.set('status', params.status);

      const url = `${this.baseUrl}?${searchParams.toString()}`;
      const response = await fetchWithAuth(url);

      if (!response.ok) {
        throw new Error(`Failed to fetch members: ${response.statusText}`);
      }

      const data = await response.json();

      // Backend returns { members: [], total: number, limit: number, offset: number }
      if (data.members) {
        // Filter out admin users - only show regular members
        const regularMembers = data.members
          .map(this.normalizeMember.bind(this))
          .filter((member: any) => member.serviceRole === ServiceRole.MEMBER);

        const limit = data.limit || params.limit || 10;
        const offset = data.offset || 0;
        return {
          members: regularMembers,
          total: regularMembers.length,
          page: Math.floor(offset / limit) + 1, // Convert offset back to page
          limit: limit,
          totalPages: Math.ceil(regularMembers.length / limit),
        };
      } else {
        // Handle unexpected format
        logger.warn('Unexpected members response format:', data);
        return {
          members: [],
          total: 0,
          page: 1,
          limit: 50,
          totalPages: 0,
        };
      }
    } catch (error) {
      logger.error('Error fetching members:', error);
      throw error;
    }
  }

  async getMember(id: string): Promise<Member> {
    try {
      const response = await fetch(`${this.baseUrl}/${id}`, {
        method: 'GET',
        headers: {
          'Content-Type': 'application/json',
        },
      });

      if (!response.ok) {
        throw new Error(`Failed to fetch member: ${response.statusText}`);
      }

      const data = await response.json();
      return this.normalizeMember(data);
    } catch (error) {
      logger.error(`Error fetching member ${id}:`, error);
      throw error;
    }
  }

  async createMember(memberData: CreateMemberRequest): Promise<Member> {
    try {
      const response = await fetch(this.baseUrl, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          ...memberData,
          serviceRole: memberData.serviceRole || ServiceRole.MEMBER,
        }),
      });

      if (!response.ok) {
        const errorData = await response.json().catch(() => ({}));
        throw new Error(
          errorData.message ||
            `Failed to create member: ${response.statusText}`,
        );
      }

      const data = await response.json();
      return this.normalizeMember(data);
    } catch (error) {
      logger.error('Error creating member:', error);
      throw error;
    }
  }

  async updateMember(
    id: string,
    updates: UpdateMemberRequest,
  ): Promise<Member> {
    try {
      const response = await fetch(`${this.baseUrl}/${id}`, {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(updates),
      });

      if (!response.ok) {
        const errorData = await response.json().catch(() => ({}));
        throw new Error(
          errorData.message ||
            `Failed to update member: ${response.statusText}`,
        );
      }

      const data = await response.json();
      return this.normalizeMember(data);
    } catch (error) {
      logger.error(`Error updating member ${id}:`, error);
      throw error;
    }
  }

  async deleteMember(id: string): Promise<void> {
    try {
      const response = await fetch(`${this.baseUrl}/${id}`, {
        method: 'DELETE',
        headers: {
          'Content-Type': 'application/json',
        },
      });

      if (!response.ok) {
        const errorData = await response.json().catch(() => ({}));
        throw new Error(
          errorData.message ||
            `Failed to delete member: ${response.statusText}`,
        );
      }
    } catch (error) {
      logger.error(`Error deleting member ${id}:`, error);
      throw error;
    }
  }

  private normalizeMember(user: any): Member {
    return {
      id: user.id || user.memberId || user.userId,
      email: user.email,
      firstName: user.firstName || user.given_name,
      lastName: user.lastName || user.family_name,
      name:
        user.name || `${user.firstName || ''} ${user.lastName || ''}`.trim(),
      avatar: user.avatar || user.profilePicture || user.profile?.avatarUrl,
      phone: user.phone || user.phoneNumber,
      npub: user.npub || user.nostr?.npub,
      emailVerified: user.emailVerified || user.email_verified || false,
      serviceRole: user.serviceRole || ServiceRole.MEMBER,
      groupMemberships: user.groupMemberships || [],
      createdAt: user.createdAt
        ? new Date(user.createdAt)
        : user.created
          ? new Date(user.created)
          : undefined,
      updatedAt: user.updatedAt
        ? new Date(user.updatedAt)
        : user.updated
          ? new Date(user.updated)
          : undefined,

      // Additional member fields
      address: user.address,
      organizationId: user.organizationId,
      chamaIds: user.chamaIds || [],
      membershipNumber: user.membershipNumber,
      joinedAt: user.joinedAt ? new Date(user.joinedAt) : undefined,
      status: user.status || 'active',
    };
  }
}

// Export singleton instance
export const membersClient = new MembersClient();

// Export individual functions for convenience
export { MembersClient };
