import { useState, useCallback, useEffect } from 'react';
import { User, ServiceRole } from '@bitsaccoserver/types';
import { adminsClient } from '@/lib/admins/client';
import { logger } from '@/lib/default-logger';

interface UseAdminsOptions {
  page?: number;
  limit?: number;
  search?: string;
  sortBy?: 'name' | 'email' | 'createdAt' | 'updatedAt';
  sortOrder?: 'asc' | 'desc';
  initialAdmins?: User[];
}

interface UseAdminsResult {
  admins: User[];
  totalCount: number;
  isLoading: boolean;
  error: string | null;
  refetch: () => Promise<void>;
  search: (term: string) => void;
  setPage: (page: number) => void;
  setLimit: (limit: number) => void;
  setSort: (
    sortBy: 'name' | 'email' | 'createdAt' | 'updatedAt',
    sortOrder: 'asc' | 'desc',
  ) => void;
}

export function useAdmins({
  page = 0,
  limit = 10,
  search: initialSearch = '',
  sortBy: initialSortBy = 'createdAt' as const,
  sortOrder: initialSortOrder = 'desc',
  initialAdmins = [],
}: UseAdminsOptions = {}): UseAdminsResult {
  const [admins, setAdmins] = useState<User[]>(initialAdmins);
  const [totalCount, setTotalCount] = useState<number>(0);
  const [isLoading, setIsLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);
  const [queryParams, setQueryParams] = useState({
    page,
    limit,
    search: initialSearch,
    sortBy: initialSortBy,
    sortOrder: initialSortOrder,
  });

  const fetchAdmins = useCallback(async () => {
    setIsLoading(true);
    setError(null);

    try {
      const response = await adminsClient.getAdmins(queryParams);
      setAdmins(response.admins);
      setTotalCount(response.total);
      logger.debug(
        `Loaded ${response.admins.length} of ${response.total} admins`,
      );
    } catch (err) {
      logger.error('Error fetching admins:', err);
      setError('Failed to fetch admins. Please try again.');
    } finally {
      setIsLoading(false);
    }
  }, [queryParams]);

  useEffect(() => {
    fetchAdmins();
  }, [fetchAdmins]);

  const search = useCallback((term: string) => {
    setQueryParams((prev) => ({
      ...prev,
      search: term,
      page: 0, // Reset to first page when searching
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
    admins,
    totalCount,
    isLoading,
    error,
    refetch: fetchAdmins,
    search,
    setPage,
    setLimit,
    setSort,
  };
}
