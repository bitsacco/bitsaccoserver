'use client';

import * as React from 'react';
import { useState } from 'react';
import Card from '@mui/material/Card';
import CardContent from '@mui/material/CardContent';
import CardHeader from '@mui/material/CardHeader';
import Stack from '@mui/material/Stack';
import Table from '@mui/material/Table';
import TableBody from '@mui/material/TableBody';
import TableCell from '@mui/material/TableCell';
import TableContainer from '@mui/material/TableContainer';
import TableHead from '@mui/material/TableHead';
import TableRow from '@mui/material/TableRow';
import Typography from '@mui/material/Typography';
import Button from '@mui/material/Button';
import CircularProgress from '@mui/material/CircularProgress';
import Alert from '@mui/material/Alert';
import Chip from '@mui/material/Chip';
import IconButton from '@mui/material/IconButton';
import Menu from '@mui/material/Menu';
import MenuItem from '@mui/material/MenuItem';
import FormControl from '@mui/material/FormControl';
import InputLabel from '@mui/material/InputLabel';
import Select from '@mui/material/Select';
import { ArrowClockwise as ArrowClockwiseIcon } from '@phosphor-icons/react/dist/ssr/ArrowClockwise';
import { DotsThree as DotsThreeIcon } from '@phosphor-icons/react/dist/ssr/DotsThree';

import { ServiceRole } from '@bitsaccoserver/types';
import { useAdminMembers } from '@/hooks/use-admin-members';
import { useUser } from '@/hooks/use-user';
import type { AdminMember } from '@/lib/admin-members/client';

export function AdminMembersCard(): React.JSX.Element {
  const [roleFilter, setRoleFilter] = useState<ServiceRole | ''>('');
  const [statusFilter, setStatusFilter] = useState<
    'active' | 'inactive' | 'suspended' | ''
  >('');
  const [actionMenuAnchor, setActionMenuAnchor] = useState<null | HTMLElement>(
    null,
  );
  const [selectedMember, setSelectedMember] = useState<AdminMember | null>(
    null,
  );

  const { user } = useUser();

  // Memoize query parameters to prevent unnecessary re-renders
  const queryParams = React.useMemo(
    () => ({
      role: roleFilter || undefined,
      status: statusFilter || undefined,
      limit: 10, // Small limit since system admins are few
      offset: 0,
    }),
    [roleFilter, statusFilter],
  );

  const {
    members,
    totalCount,
    isLoading,
    error,
    refresh,
    updateRole,
    updateStatus,
  } = useAdminMembers(queryParams);

  const isSystemAdmin = user?.serviceRole === ServiceRole.SYSTEM_ADMIN;

  const handleActionClick = (
    event: React.MouseEvent<HTMLElement>,
    member: AdminMember,
  ) => {
    setSelectedMember(member);
    setActionMenuAnchor(event.currentTarget);
  };

  const handleActionClose = () => {
    setActionMenuAnchor(null);
    setSelectedMember(null);
  };

  const handleRoleChange = async (newRole: ServiceRole) => {
    if (selectedMember && isSystemAdmin) {
      try {
        await updateRole(selectedMember.memberId, newRole);
        handleActionClose();
      } catch (error) {
        console.error('Failed to update role:', error);
      }
    }
  };

  const handleStatusChange = async (
    newStatus: 'active' | 'inactive' | 'suspended',
  ) => {
    if (selectedMember) {
      try {
        await updateStatus(
          selectedMember.memberId,
          newStatus,
          `Status changed by admin`,
        );
        handleActionClose();
      } catch (error) {
        console.error('Failed to update status:', error);
      }
    }
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'active':
        return 'success';
      case 'inactive':
        return 'default';
      case 'suspended':
        return 'error';
      default:
        return 'default';
    }
  };

  const getRoleColor = (role: ServiceRole) => {
    switch (role) {
      case ServiceRole.SYSTEM_ADMIN:
        return 'error';
      case ServiceRole.ADMIN:
        return 'warning';
      case ServiceRole.MEMBER:
        return 'info';
      default:
        return 'default';
    }
  };

  // Don't show this card to non-admins
  if (!isSystemAdmin && user?.serviceRole !== ServiceRole.ADMIN) {
    return <></>;
  }

  return (
    <Card>
      <CardHeader
        title="System Administration"
        subheader="Manage system administrators and their permissions"
        action={
          <Button
            startIcon={
              <ArrowClockwiseIcon fontSize="var(--icon-fontSize-md)" />
            }
            variant="outlined"
            size="small"
            onClick={refresh}
            disabled={isLoading}
          >
            Refresh
          </Button>
        }
      />
      <CardContent>
        <Stack spacing={2}>
          {/* Filters */}
          <Stack direction="row" spacing={2} alignItems="center">
            <FormControl size="small" sx={{ minWidth: 120 }}>
              <InputLabel>Role</InputLabel>
              <Select
                value={roleFilter}
                onChange={(e) =>
                  setRoleFilter(e.target.value as ServiceRole | '')
                }
                label="Role"
              >
                <MenuItem value="">All Roles</MenuItem>
                <MenuItem value={ServiceRole.SYSTEM_ADMIN}>
                  System Admin
                </MenuItem>
                <MenuItem value={ServiceRole.ADMIN}>Admin</MenuItem>
                <MenuItem value={ServiceRole.MEMBER}>Member</MenuItem>
              </Select>
            </FormControl>

            <FormControl size="small" sx={{ minWidth: 120 }}>
              <InputLabel>Status</InputLabel>
              <Select
                value={statusFilter}
                onChange={(e) => setStatusFilter(e.target.value as any)}
                label="Status"
              >
                <MenuItem value="">All Status</MenuItem>
                <MenuItem value="active">Active</MenuItem>
                <MenuItem value="inactive">Inactive</MenuItem>
                <MenuItem value="suspended">Suspended</MenuItem>
              </Select>
            </FormControl>

            <Typography
              variant="body2"
              color="text.secondary"
              sx={{ ml: 'auto' }}
            >
              Total: {totalCount} members
            </Typography>
          </Stack>

          {/* Loading state */}
          {isLoading && (
            <Stack alignItems="center" sx={{ py: 2 }}>
              <CircularProgress size={24} />
            </Stack>
          )}

          {/* Error state */}
          {error && (
            <Alert severity="error">
              Error loading members: {error.message}
            </Alert>
          )}

          {/* Members table */}
          {!isLoading && !error && (
            <TableContainer>
              <Table size="small">
                <TableHead>
                  <TableRow>
                    <TableCell>Email</TableCell>
                    <TableCell>Service Role</TableCell>
                    <TableCell>Status</TableCell>
                    <TableCell>Last Login</TableCell>
                    <TableCell align="right">Actions</TableCell>
                  </TableRow>
                </TableHead>
                <TableBody>
                  {members.map((member) => (
                    <TableRow key={member.memberId}>
                      <TableCell>
                        <Typography variant="body2">{member.email}</Typography>
                      </TableCell>
                      <TableCell>
                        <Chip
                          label={member.serviceRole}
                          size="small"
                          color={getRoleColor(member.serviceRole) as any}
                        />
                      </TableCell>
                      <TableCell>
                        <Chip
                          label={member.status}
                          size="small"
                          color={getStatusColor(member.status) as any}
                        />
                      </TableCell>
                      <TableCell>
                        <Typography variant="body2" color="text.secondary">
                          {member.lastLoginAt
                            ? new Date(member.lastLoginAt).toLocaleDateString()
                            : 'Never'}
                        </Typography>
                      </TableCell>
                      <TableCell align="right">
                        <IconButton
                          size="small"
                          onClick={(e) => handleActionClick(e, member)}
                          disabled={
                            !isSystemAdmin &&
                            member.serviceRole === ServiceRole.SYSTEM_ADMIN
                          }
                        >
                          <DotsThreeIcon />
                        </IconButton>
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </TableContainer>
          )}
        </Stack>

        {/* Action menu */}
        <Menu
          anchorEl={actionMenuAnchor}
          open={Boolean(actionMenuAnchor)}
          onClose={handleActionClose}
        >
          {isSystemAdmin && selectedMember && (
            <>
              <MenuItem disabled>Change Role</MenuItem>
              {Object.values(ServiceRole).map((role) => (
                <MenuItem
                  key={role}
                  onClick={() => handleRoleChange(role)}
                  disabled={selectedMember.serviceRole === role}
                  sx={{ pl: 3 }}
                >
                  {role}
                </MenuItem>
              ))}
            </>
          )}

          <MenuItem disabled>Change Status</MenuItem>
          {['active', 'inactive', 'suspended'].map((status) => (
            <MenuItem
              key={status}
              onClick={() => handleStatusChange(status as any)}
              disabled={selectedMember?.status === status}
              sx={{ pl: 3 }}
            >
              {status}
            </MenuItem>
          ))}
        </Menu>
      </CardContent>
    </Card>
  );
}
