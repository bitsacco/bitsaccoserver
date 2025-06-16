import { ServiceRole } from '@bitsaccoserver/types';
import { fetchWithAuth } from '@/lib/fetch-with-auth';

/**
 * Admin member management types based on server admin endpoints
 */
export interface AdminMember {
  memberId: string;
  email: string;
  serviceRole: ServiceRole;
  status: 'active' | 'inactive' | 'suspended';
  lastLoginAt: Date;
  createdAt: Date;
}

export interface AdminMemberListResponse {
  members: AdminMember[];
  total: number;
  limit: number;
  offset: number;
}

export interface AdminMemberListParams {
  role?: ServiceRole;
  status?: 'active' | 'inactive' | 'suspended';
  limit?: number;
  offset?: number;
}

/**
 * Fetch system members from admin endpoint
 */
export async function fetchAdminMembers(
  params: AdminMemberListParams = {},
): Promise<AdminMemberListResponse> {
  const queryParams = new URLSearchParams();

  if (params.role) {
    queryParams.append('role', params.role);
  }

  if (params.status) {
    queryParams.append('status', params.status);
  }

  if (params.limit) {
    queryParams.append('limit', params.limit.toString());
  }

  if (params.offset) {
    queryParams.append('offset', params.offset.toString());
  }

  try {
    const response = await fetchWithAuth(
      `/admin/members?${queryParams.toString()}`,
    );

    if (!response.ok) {
      throw new Error(`Failed to fetch admin members: ${response.statusText}`);
    }

    return await response.json();
  } catch (error) {
    console.error('Error fetching admin members:', error);
    throw error;
  }
}

/**
 * Update member service role (SYSTEM_ADMIN only)
 */
export async function updateMemberRole(
  memberId: string,
  serviceRole: ServiceRole,
): Promise<{ success: boolean }> {
  try {
    const response = await fetchWithAuth(`/admin/members/${memberId}/role`, {
      method: 'PUT',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ serviceRole }),
    });

    if (!response.ok) {
      throw new Error(`Failed to update member role: ${response.statusText}`);
    }

    return await response.json();
  } catch (error) {
    console.error('Error updating member role:', error);
    throw error;
  }
}

/**
 * Update member status (ADMIN+)
 */
export async function updateMemberStatus(
  memberId: string,
  status: 'active' | 'inactive' | 'suspended',
  reason?: string,
): Promise<{ success: boolean }> {
  try {
    const response = await fetchWithAuth(`/admin/members/${memberId}/status`, {
      method: 'PUT',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ status, reason }),
    });

    if (!response.ok) {
      throw new Error(`Failed to update member status: ${response.statusText}`);
    }

    return await response.json();
  } catch (error) {
    console.error('Error updating member status:', error);
    throw error;
  }
}
