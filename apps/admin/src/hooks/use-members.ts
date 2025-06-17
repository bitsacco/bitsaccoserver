import { useState, useCallback, useEffect } from 'react';
import { Member, ServiceRole } from '@bitsaccoserver/types';
import { membersClient } from '@/lib/members/client';
import { logger } from '@/lib/default-logger';

interface UseMembersOptions {
  page?: number;
  limit?: number;
  search?: string;
  roleFilter?: ServiceRole | null;
  sortBy?: 'name' | 'email' | 'createdAt' | 'updatedAt';
  sortOrder?: 'asc' | 'desc';
  initialMembers?: Member[];
}

interface UseMembersResult {
  members: Member[];
  totalCount: number;
  isLoading: boolean;
  error: string | null;
  refetch: () => Promise<void>;
  search: (term: string) => void;
  filterByRole: (role: ServiceRole | null) => void;
  setPage: (page: number) => void;
  setLimit: (limit: number) => void;
  setSort: (
    sortBy: 'name' | 'email' | 'createdAt' | 'updatedAt',
    sortOrder: 'asc' | 'desc',
  ) => void;
}

export function useMembers({
  page = 0,
  limit = 10,
  search: initialSearch = '',
  roleFilter: initialRoleFilter = null,
  sortBy: initialSortBy = 'createdAt' as const,
  sortOrder: initialSortOrder = 'desc',
  initialMembers = [],
}: UseMembersOptions = {}): UseMembersResult {
  const [members, setMembers] = useState<Member[]>(initialMembers);
  const [totalCount, setTotalCount] = useState<number>(0);
  const [isLoading, setIsLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);
  const [queryParams, setQueryParams] = useState({
    page,
    limit,
    search: initialSearch,
    roleFilter: initialRoleFilter,
    sortBy: initialSortBy,
    sortOrder: initialSortOrder,
  });

  const fetchMembers = useCallback(async () => {
    setIsLoading(true);
    setError(null);

    try {
      const response = await membersClient.getMembers(queryParams);
      setMembers(response.members);
      setTotalCount(response.total);
      logger.debug(
        `Loaded ${response.members.length} of ${response.total} members`,
      );
    } catch (err) {
      logger.error('Error fetching members:', err);
      setError('Failed to fetch members. Please try again.');
    } finally {
      setIsLoading(false);
    }
  }, [queryParams]);

  // Fetch members on initial render and when params change
  useEffect(() => {
    fetchMembers();
  }, [fetchMembers]);

  const search = useCallback((term: string) => {
    setQueryParams((prev) => ({
      ...prev,
      search: term,
      page: 0, // Reset to first page when searching
    }));
  }, []);

  const filterByRole = useCallback((role: ServiceRole | null) => {
    setQueryParams((prev) => ({
      ...prev,
      roleFilter: role,
      page: 0, // Reset to first page when filtering
    }));
  }, []);

  const setPage = useCallback((newPage: number) => {
    setQueryParams((prev) => ({
      ...prev,
      page: newPage,
    }));
  }, []);

  const setLimit = useCallback((newLimit: number) => {
    setQueryParams((prev) => ({
      ...prev,
      limit: newLimit,
      page: 0, // Reset to first page when changing limit
    }));
  }, []);

  const setSort = useCallback(
    (
      sortBy: 'name' | 'email' | 'createdAt' | 'updatedAt',
      sortOrder: 'asc' | 'desc',
    ) => {
      setQueryParams((prev) => ({
        ...prev,
        sortBy,
        sortOrder,
      }));
    },
    [],
  );

  return {
    members,
    totalCount,
    isLoading,
    error,
    refetch: fetchMembers,
    search,
    filterByRole,
    setPage,
    setLimit,
    setSort,
  };
}
