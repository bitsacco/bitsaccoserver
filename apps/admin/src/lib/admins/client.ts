'use client';

import {
  User,
  ServiceRole,
  CreateMemberRequest,
  UpdateMemberRequest,
} from '@bitsaccoserver/types';
import { logger } from '@/lib/default-logger';
import { fetchWithAuth } from '@/lib/fetch-with-auth';

// Admin-specific interfaces
export interface AdminListParams {
  page?: number;
  limit?: number;
  search?: string;
  sortBy?: 'name' | 'email' | 'createdAt' | 'updatedAt';
  sortOrder?: 'asc' | 'desc';
}

export interface AdminListResponse {
  admins: User[];
  total: number;
  page: number;
  limit: number;
  totalPages: number;
}

export interface CreateAdminRequest
  extends Omit<CreateMemberRequest, 'serviceRole'> {
  serviceRole: ServiceRole.ADMIN | ServiceRole.SYSTEM_ADMIN;
}

export interface UpdateAdminRequest
  extends Omit<UpdateMemberRequest, 'serviceRole'> {
  serviceRole?:
    | ServiceRole.ADMIN
    | ServiceRole.SYSTEM_ADMIN
    | ServiceRole.MEMBER;
}

class AdminsClient {
  private baseUrl = '/api/admin/members';

  async getAdmins(params: AdminListParams = {}): Promise<AdminListResponse> {
    try {
      const searchParams = new URLSearchParams();

      // Add pagination
      if (params.limit) searchParams.set('limit', params.limit.toString());
      if (params.page) {
        // Convert page to offset (page is 1-based, offset is 0-based)
        const offset = (params.page - 1) * (params.limit || 10);
        searchParams.set('offset', offset.toString());
      }

      // Add filters
      if (params.search) searchParams.set('search', params.search);
      if (params.sortBy) searchParams.set('sortBy', params.sortBy);
      if (params.sortOrder) searchParams.set('sortOrder', params.sortOrder);

      // Don't set role filter here - we'll filter client-side for admin roles

      const url = `${this.baseUrl}?${searchParams.toString()}`;
      const response = await fetchWithAuth(url);

      if (!response.ok) {
        throw new Error(`Failed to fetch admins: ${response.statusText}`);
      }

      const data = await response.json();

      // Backend returns { members: [], total: number, limit: number, offset: number }
      if (data.members) {
        // Filter for admin roles only
        const adminMembers = data.members
          .map(this.normalizeAdmin.bind(this))
          .filter(
            (member: any) =>
              member.serviceRole === ServiceRole.ADMIN ||
              member.serviceRole === ServiceRole.SYSTEM_ADMIN,
          );

        const limit = data.limit || params.limit || 10;
        const offset = data.offset || 0;
        return {
          admins: adminMembers,
          total: adminMembers.length,
          page: Math.floor(offset / limit) + 1,
          limit: limit,
          totalPages: Math.ceil(adminMembers.length / limit),
        };
      } else {
        // Handle unexpected format
        logger.warn('Unexpected admins response format:', data);
        return {
          admins: [],
          total: 0,
          page: 1,
          limit: 50,
          totalPages: 0,
        };
      }
    } catch (error) {
      logger.error('Error fetching admins:', error);
      throw error;
    }
  }

  async getAdmin(id: string): Promise<User> {
    try {
      const response = await fetchWithAuth(`${this.baseUrl}/${id}`);

      if (!response.ok) {
        throw new Error(`Failed to fetch admin: ${response.statusText}`);
      }

      const data = await response.json();
      return this.normalizeAdmin(data);
    } catch (error) {
      logger.error(`Error fetching admin ${id}:`, error);
      throw error;
    }
  }

  async createAdmin(adminData: CreateAdminRequest): Promise<User> {
    try {
      const response = await fetchWithAuth(this.baseUrl, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          ...adminData,
          serviceRole: adminData.serviceRole || ServiceRole.ADMIN,
        }),
      });

      if (!response.ok) {
        const errorData = await response.json().catch(() => ({}));
        throw new Error(
          errorData.message || `Failed to create admin: ${response.statusText}`,
        );
      }

      const data = await response.json();
      return this.normalizeAdmin(data);
    } catch (error) {
      logger.error('Error creating admin:', error);
      throw error;
    }
  }

  async updateAdmin(id: string, updates: UpdateAdminRequest): Promise<User> {
    try {
      const response = await fetchWithAuth(`${this.baseUrl}/${id}`, {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(updates),
      });

      if (!response.ok) {
        const errorData = await response.json().catch(() => ({}));
        throw new Error(
          errorData.message || `Failed to update admin: ${response.statusText}`,
        );
      }

      const data = await response.json();
      return this.normalizeAdmin(data);
    } catch (error) {
      logger.error(`Error updating admin ${id}:`, error);
      throw error;
    }
  }

  async deleteAdmin(id: string): Promise<void> {
    try {
      const response = await fetchWithAuth(`${this.baseUrl}/${id}`, {
        method: 'DELETE',
        headers: {
          'Content-Type': 'application/json',
        },
      });

      if (!response.ok) {
        const errorData = await response.json().catch(() => ({}));
        throw new Error(
          errorData.message || `Failed to delete admin: ${response.statusText}`,
        );
      }
    } catch (error) {
      logger.error(`Error deleting admin ${id}:`, error);
      throw error;
    }
  }

  async promoteToAdmin(userId: string): Promise<User> {
    try {
      // Use the updateAdmin method to change the role
      return await this.updateAdmin(userId, { serviceRole: ServiceRole.ADMIN });
    } catch (error) {
      logger.error(`Error promoting user ${userId} to admin:`, error);
      throw error;
    }
  }

  async demoteAdmin(adminId: string): Promise<User> {
    try {
      // Use the updateAdmin method to change the role
      return await this.updateAdmin(adminId, {
        serviceRole: ServiceRole.MEMBER,
      });
    } catch (error) {
      logger.error(`Error demoting admin ${adminId}:`, error);
      throw error;
    }
  }

  private normalizeAdmin(user: any): User {
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
    };
  }
}

// Export singleton instance
export const adminsClient = new AdminsClient();

// Export class for convenience
export { AdminsClient };
