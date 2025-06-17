'use client';

import * as React from 'react';
import Box from '@mui/material/Box';
import Card from '@mui/material/Card';
import Divider from '@mui/material/Divider';
import FormControl from '@mui/material/FormControl';
import InputAdornment from '@mui/material/InputAdornment';
import InputLabel from '@mui/material/InputLabel';
import MenuItem from '@mui/material/MenuItem';
import OutlinedInput from '@mui/material/OutlinedInput';
import Select from '@mui/material/Select';
import Stack from '@mui/material/Stack';
import { MagnifyingGlass as MagnifyingGlassIcon } from '@phosphor-icons/react/dist/ssr/MagnifyingGlass';

interface AdminsFiltersProps {
  onSearch?: (term: string) => void;
  onSort?: (
    sortBy: 'name' | 'email' | 'createdAt' | 'updatedAt',
    sortOrder: 'asc' | 'desc',
  ) => void;
}

export function AdminsFilters({
  onSearch,
  onSort,
}: AdminsFiltersProps): React.JSX.Element {
  const [searchTerm, setSearchTerm] = React.useState<string>('');
  const [sortBy, setSortBy] = React.useState<
    'name' | 'email' | 'createdAt' | 'updatedAt'
  >('createdAt');
  const [sortOrder, setSortOrder] = React.useState<'asc' | 'desc'>('desc');

  const handleSearchChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    const value = event.target.value;
    setSearchTerm(value);
    onSearch?.(value);
  };

  const handleSortByChange = (event: any) => {
    const value = event.target.value as
      | 'name'
      | 'email'
      | 'createdAt'
      | 'updatedAt';
    setSortBy(value);
    onSort?.(value, sortOrder);
  };

  const handleSortOrderChange = (event: any) => {
    const value = event.target.value as 'asc' | 'desc';
    setSortOrder(value);
    onSort?.(sortBy, value);
  };

  return (
    <Card sx={{ p: 2 }}>
      <Stack spacing={2}>
        <Stack
          direction="row"
          spacing={2}
          sx={{ alignItems: 'center', flexWrap: 'wrap' }}
        >
          <Box sx={{ flex: '1 1 auto', minWidth: '240px' }}>
            <FormControl fullWidth>
              <OutlinedInput
                value={searchTerm}
                onChange={handleSearchChange}
                placeholder="Search admins..."
                startAdornment={
                  <InputAdornment position="start">
                    <MagnifyingGlassIcon fontSize="var(--icon-fontSize-md)" />
                  </InputAdornment>
                }
                sx={{ maxWidth: '500px' }}
              />
            </FormControl>
          </Box>

          <FormControl sx={{ minWidth: '120px' }}>
            <InputLabel>Sort by</InputLabel>
            <Select
              value={sortBy}
              onChange={handleSortByChange}
              label="Sort by"
            >
              <MenuItem value="name">Name</MenuItem>
              <MenuItem value="email">Email</MenuItem>
              <MenuItem value="createdAt">Created</MenuItem>
              <MenuItem value="updatedAt">Updated</MenuItem>
            </Select>
          </FormControl>

          <FormControl sx={{ minWidth: '120px' }}>
            <InputLabel>Order</InputLabel>
            <Select
              value={sortOrder}
              onChange={handleSortOrderChange}
              label="Order"
            >
              <MenuItem value="asc">Ascending</MenuItem>
              <MenuItem value="desc">Descending</MenuItem>
            </Select>
          </FormControl>
        </Stack>
      </Stack>
    </Card>
  );
}
