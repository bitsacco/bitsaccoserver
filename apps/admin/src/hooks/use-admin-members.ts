import { useState, useEffect, useCallback, useRef } from 'react';
import { ServiceRole } from '@bitsaccoserver/types';
import {
  fetchAdminMembers,
  updateMemberRole,
  updateMemberStatus,
  AdminMember,
  AdminMemberListParams,
} from '@/lib/admin-members/client';

interface UseAdminMembersOptions extends AdminMemberListParams {
  autoRefresh?: boolean;
  refreshInterval?: number;
}

interface UseAdminMembersResult {
  members: AdminMember[];
  totalCount: number;
  isLoading: boolean;
  error: Error | null;
  refresh: () => Promise<void>;
  updateRole: (memberId: string, role: ServiceRole) => Promise<void>;
  updateStatus: (
    memberId: string,
    status: 'active' | 'inactive' | 'suspended',
    reason?: string,
  ) => Promise<void>;
}

/**
 * Custom hook for admin member management
 */
export function useAdminMembers(
  options: UseAdminMembersOptions = {},
): UseAdminMembersResult {
  const [members, setMembers] = useState<AdminMember[]>([]);
  const [totalCount, setTotalCount] = useState<number>(0);
  const [isLoading, setIsLoading] = useState<boolean>(true);
  const [error, setError] = useState<Error | null>(null);

  const {
    autoRefresh = false,
    refreshInterval = 30000,
    ...queryParams
  } = options;

  // Use ref to store latest query params to avoid dependency issues
  const queryParamsRef = useRef(queryParams);
  queryParamsRef.current = queryParams;

  const fetchData = useCallback(async () => {
    try {
      setIsLoading(true);
      setError(null);

      const response = await fetchAdminMembers(queryParamsRef.current);
      setMembers(response.members);
      setTotalCount(response.total);
    } catch (err) {
      setError(err instanceof Error ? err : new Error('Unknown error'));
    } finally {
      setIsLoading(false);
    }
  }, []); // No dependencies - we use ref for latest values

  const handleUpdateRole = useCallback(
    async (memberId: string, role: ServiceRole) => {
      try {
        await updateMemberRole(memberId, role);
        // Refresh the data after successful update
        await fetchData();
      } catch (err) {
        setError(
          err instanceof Error
            ? err
            : new Error('Failed to update member role'),
        );
        throw err;
      }
    },
    [fetchData],
  );

  const handleUpdateStatus = useCallback(
    async (
      memberId: string,
      status: 'active' | 'inactive' | 'suspended',
      reason?: string,
    ) => {
      try {
        await updateMemberStatus(memberId, status, reason);
        // Refresh the data after successful update
        await fetchData();
      } catch (err) {
        setError(
          err instanceof Error
            ? err
            : new Error('Failed to update member status'),
        );
        throw err;
      }
    },
    [fetchData],
  );

  // Initial fetch and re-fetch when query params change
  useEffect(() => {
    fetchData();
  }, [
    queryParams.role,
    queryParams.status,
    queryParams.limit,
    queryParams.offset,
    fetchData,
  ]);

  // Auto-refresh functionality
  useEffect(() => {
    if (autoRefresh && refreshInterval > 0) {
      const intervalId = setInterval(fetchData, refreshInterval);
      return () => clearInterval(intervalId);
    }
  }, [autoRefresh, refreshInterval, fetchData]);

  return {
    members,
    totalCount,
    isLoading,
    error,
    refresh: fetchData,
    updateRole: handleUpdateRole,
    updateStatus: handleUpdateStatus,
  };
}
