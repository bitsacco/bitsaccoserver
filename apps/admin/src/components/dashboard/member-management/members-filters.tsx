'use client';

import * as React from 'react';
import Card from '@mui/material/Card';
import InputAdornment from '@mui/material/InputAdornment';
import OutlinedInput from '@mui/material/OutlinedInput';
import Button from '@mui/material/Button';
import Stack from '@mui/material/Stack';
import MenuItem from '@mui/material/MenuItem';
import Select, { SelectChangeEvent } from '@mui/material/Select';
import InputLabel from '@mui/material/InputLabel';
import FormControl from '@mui/material/FormControl';
import Box from '@mui/material/Box';
import { MagnifyingGlass as MagnifyingGlassIcon } from '@phosphor-icons/react/dist/ssr/MagnifyingGlass';
import { FunnelSimple as FilterIcon } from '@phosphor-icons/react/dist/ssr/FunnelSimple';
import { X as ClearIcon } from '@phosphor-icons/react/dist/ssr/X';

import { ServiceRole } from '@bitsaccoserver/types';
import { useUser } from '@/hooks/use-user';
import { isSuperAdmin } from '@/lib/members/client';

interface MembersFiltersProps {
  onSearch: (value: string) => void;
  onSort: (field: string, order: 'asc' | 'desc') => void;
  onFilterRole?: (role: ServiceRole | null) => void;
  sortField?: string;
  sortOrder?: 'asc' | 'desc';
  selectedRole?: ServiceRole | null;
}

export function MembersFilters({
  onSearch,
  onSort,
  onFilterRole,
  sortField = 'createdAt',
  sortOrder = 'desc',
  selectedRole = null,
}: MembersFiltersProps): React.JSX.Element {
  // Get current user and check if they're a super admin
  const { user } = useUser();
  const currentUserIsSuperAdmin = isSuperAdmin(user);

  const [searchValue, setSearchValue] = React.useState('');
  const [showFilters, setShowFilters] = React.useState(false);
  const [localSortField, setLocalSortField] = React.useState(sortField);
  const [localSortOrder, setLocalSortOrder] = React.useState<'asc' | 'desc'>(
    sortOrder,
  );
  const [localSelectedRole, setLocalSelectedRole] =
    React.useState<ServiceRole | null>(selectedRole);

  const handleSearch = (event: React.ChangeEvent<HTMLInputElement>) => {
    const newValue = event.target.value;
    setSearchValue(newValue);
    onSearch(newValue);
  };

  const handleClearSearch = () => {
    setSearchValue('');
    onSearch('');
  };

  const handleSortFieldChange = (event: SelectChangeEvent) => {
    const newSortField = event.target.value;
    setLocalSortField(newSortField);
    onSort(newSortField, localSortOrder);
  };

  const handleSortOrderChange = (event: SelectChangeEvent) => {
    const newSortOrder = event.target.value as 'asc' | 'desc';
    setLocalSortOrder(newSortOrder);
    onSort(localSortField, newSortOrder);
  };

  const handleRoleFilterChange = (event: SelectChangeEvent) => {
    const value = event.target.value;
    // If "all" is selected, set to null, otherwise use the ServiceRole value
    const newRole = value === 'all' ? null : (value as ServiceRole);

    // Prevent non-super-admins from filtering by SYSTEM_ADMIN role
    if (!currentUserIsSuperAdmin && newRole === ServiceRole.SYSTEM_ADMIN) {
      console.warn('Regular admins cannot filter by System Admin role');
      return;
    }

    setLocalSelectedRole(newRole);
    if (onFilterRole) {
      onFilterRole(newRole);
    }
  };

  const toggleFilters = () => {
    setShowFilters(!showFilters);
  };

  return (
    <Card sx={{ p: 2 }}>
      <Stack spacing={2}>
        <Stack direction="row" spacing={2} alignItems="center">
          <Box sx={{ position: 'relative', width: '100%', maxWidth: '500px' }}>
            <OutlinedInput
              value={searchValue}
              onChange={handleSearch}
              fullWidth
              placeholder="Find member by name, phone or id"
              startAdornment={
                <InputAdornment position="start">
                  <MagnifyingGlassIcon fontSize="var(--icon-fontSize-md)" />
                </InputAdornment>
              }
              endAdornment={
                searchValue && (
                  <InputAdornment position="end">
                    <Button
                      onClick={handleClearSearch}
                      color="inherit"
                      size="small"
                      sx={{ minWidth: 'auto', p: 0.5 }}
                    >
                      <ClearIcon fontSize="var(--icon-fontSize-md)" />
                    </Button>
                  </InputAdornment>
                )
              }
            />
          </Box>
          <Button
            color="inherit"
            onClick={toggleFilters}
            startIcon={<FilterIcon fontSize="var(--icon-fontSize-md)" />}
          >
            Filters
          </Button>
        </Stack>

        {showFilters && (
          <Stack direction={{ xs: 'column', sm: 'row' }} spacing={2}>
            <FormControl sx={{ minWidth: 200 }}>
              <InputLabel id="sort-field-label">Sort By</InputLabel>
              <Select
                labelId="sort-field-label"
                value={localSortField}
                onChange={handleSortFieldChange}
                label="Sort By"
                size="small"
              >
                <MenuItem value="name">Name</MenuItem>
                <MenuItem value="email">Email</MenuItem>
                <MenuItem value="phone">Phone</MenuItem>
                <MenuItem value="createdAt">Date Joined</MenuItem>
              </Select>
            </FormControl>

            <FormControl sx={{ minWidth: 120 }}>
              <InputLabel id="sort-order-label">Order</InputLabel>
              <Select
                labelId="sort-order-label"
                value={localSortOrder}
                onChange={handleSortOrderChange}
                label="Order"
                size="small"
              >
                <MenuItem value="asc">Ascending</MenuItem>
                <MenuItem value="desc">Descending</MenuItem>
              </Select>
            </FormControl>

            <FormControl sx={{ minWidth: 150 }}>
              <InputLabel id="role-filter-label">Role</InputLabel>
              <Select
                labelId="role-filter-label"
                value={localSelectedRole === null ? 'all' : localSelectedRole}
                onChange={handleRoleFilterChange}
                label="Role"
                size="small"
              >
                <MenuItem value="all">All Roles</MenuItem>
                <MenuItem value={ServiceRole.MEMBER}>Member</MenuItem>
                <MenuItem value={ServiceRole.ADMIN}>Admin</MenuItem>
                {currentUserIsSuperAdmin && (
                  <MenuItem value={ServiceRole.SYSTEM_ADMIN}>
                    System Admin
                  </MenuItem>
                )}
              </Select>
            </FormControl>
          </Stack>
        )}
      </Stack>
    </Card>
  );
}
