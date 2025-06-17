'use client';

import * as React from 'react';
import Avatar from '@mui/material/Avatar';
import Box from '@mui/material/Box';
import Card from '@mui/material/Card';
import Checkbox from '@mui/material/Checkbox';
import Divider from '@mui/material/Divider';
import IconButton from '@mui/material/IconButton';
import Stack from '@mui/material/Stack';
import Table from '@mui/material/Table';
import TableBody from '@mui/material/TableBody';
import TableCell from '@mui/material/TableCell';
import TableHead from '@mui/material/TableHead';
import TablePagination from '@mui/material/TablePagination';
import TableRow from '@mui/material/TableRow';
import Typography from '@mui/material/Typography';
import Menu from '@mui/material/Menu';
import MenuItem from '@mui/material/MenuItem';
import CircularProgress from '@mui/material/CircularProgress';
import Chip from '@mui/material/Chip';
import { Pencil as PencilIcon } from '@phosphor-icons/react/dist/ssr/Pencil';
import { DotsThree as DotsThreeIcon } from '@phosphor-icons/react/dist/ssr/DotsThree';
import { Trash as TrashIcon } from '@phosphor-icons/react/dist/ssr/Trash';
import { CopySimple as CopyIcon } from '@phosphor-icons/react/dist/ssr/CopySimple';
import { Shield as ShieldIcon } from '@phosphor-icons/react/dist/ssr/Shield';
import { ShieldCheck as ShieldCheckIcon } from '@phosphor-icons/react/dist/ssr/ShieldCheck';
import dayjs from 'dayjs';

import { useSelection } from '@/hooks/use-selection';
import { User, ServiceRole } from '@bitsaccoserver/types';
import { useUser } from '@/hooks/use-user';

interface AdminsTableProps {
  count: number;
  page: number;
  rows: User[];
  rowsPerPage: number;
  isLoading?: boolean;
  error?: string | null;
  onPageChange: (page: number) => void;
  onRowsPerPageChange: (rowsPerPage: number) => void;
  onEdit?: (admin: User) => void;
  onDelete?: (admin: User) => void;
  onCopyId?: (id: string) => void;
}

export function AdminsTable({
  count = 0,
  rows = [],
  page = 0,
  rowsPerPage = 0,
  isLoading = false,
  error = null,
  onPageChange,
  onRowsPerPageChange,
  onEdit,
  onDelete,
  onCopyId,
}: AdminsTableProps): React.JSX.Element {
  const { user: currentUser } = useUser();
  const rowIds = React.useMemo(() => {
    return rows.map((admin) => admin.id);
  }, [rows]);

  const { deselectAll, deselectOne, selectAll, selectOne, selected } =
    useSelection(rowIds);

  const selectedSome =
    (selected?.size ?? 0) > 0 && (selected?.size ?? 0) < rows.length;
  const selectedAll = rows.length > 0 && selected?.size === rows.length;

  const [menuAnchor, setMenuAnchor] = React.useState<{
    element: HTMLElement;
    adminId: string;
  } | null>(null);

  const handleMenuOpen = (
    event: React.MouseEvent<HTMLElement>,
    adminId: string,
  ) => {
    setMenuAnchor({ element: event.currentTarget, adminId });
  };

  const handleMenuClose = () => {
    setMenuAnchor(null);
  };

  const handleEdit = (admin: User) => {
    handleMenuClose();
    onEdit?.(admin);
  };

  const handleDelete = (admin: User) => {
    handleMenuClose();
    onDelete?.(admin);
  };

  const handleCopyId = (adminId: string) => {
    handleMenuClose();
    onCopyId?.(adminId);
  };

  const getRoleColor = (role: ServiceRole) => {
    switch (role) {
      case ServiceRole.SYSTEM_ADMIN:
        return 'error';
      case ServiceRole.ADMIN:
        return 'warning';
      default:
        return 'default';
    }
  };

  const getRoleLabel = (role: ServiceRole) => {
    switch (role) {
      case ServiceRole.SYSTEM_ADMIN:
        return 'Super Admin';
      case ServiceRole.ADMIN:
        return 'Admin';
      default:
        return 'Member';
    }
  };

  const getRoleIcon = (role: ServiceRole) => {
    return role === ServiceRole.SYSTEM_ADMIN ? ShieldCheckIcon : ShieldIcon;
  };

  const canEditAdmin = (admin: User) => {
    if (!currentUser) return false;
    // System Admin can edit all admins except themselves
    // Regular admins cannot edit other admins
    return (
      currentUser.serviceRole === ServiceRole.SYSTEM_ADMIN &&
      currentUser.id !== admin.id
    );
  };

  const canDeleteAdmin = (admin: User) => {
    if (!currentUser) return false;
    // System Admin can delete regular admins, but not other System Admins or themselves
    return (
      currentUser.serviceRole === ServiceRole.SYSTEM_ADMIN &&
      admin.serviceRole === ServiceRole.ADMIN &&
      currentUser.id !== admin.id
    );
  };

  if (isLoading) {
    return (
      <Card>
        <Box sx={{ display: 'flex', justifyContent: 'center', p: 4 }}>
          <CircularProgress />
        </Box>
      </Card>
    );
  }

  if (error) {
    return (
      <Card>
        <Box sx={{ p: 3, textAlign: 'center' }}>
          <Typography color="error">{error}</Typography>
        </Box>
      </Card>
    );
  }

  return (
    <Card>
      <Box sx={{ overflowX: 'auto' }}>
        <Table sx={{ minWidth: '800px' }}>
          <TableHead>
            <TableRow>
              <TableCell padding="checkbox">
                <Checkbox
                  checked={selectedAll}
                  indeterminate={selectedSome}
                  onChange={(event) => {
                    if (event.target.checked) {
                      selectAll();
                    } else {
                      deselectAll();
                    }
                  }}
                />
              </TableCell>
              <TableCell>Admin</TableCell>
              <TableCell>Email</TableCell>
              <TableCell>Service Role</TableCell>
              <TableCell>Created</TableCell>
              <TableCell>Status</TableCell>
              <TableCell align="right">Actions</TableCell>
            </TableRow>
          </TableHead>
          <TableBody>
            {rows.map((admin) => {
              const isSelected = selected?.has(admin.id);
              const RoleIcon = getRoleIcon(admin.serviceRole);

              return (
                <TableRow hover key={admin.id} selected={isSelected}>
                  <TableCell padding="checkbox">
                    <Checkbox
                      checked={isSelected}
                      onChange={(event) => {
                        if (event.target.checked) {
                          selectOne(admin.id);
                        } else {
                          deselectOne(admin.id);
                        }
                      }}
                    />
                  </TableCell>
                  <TableCell>
                    <Stack
                      sx={{ alignItems: 'center' }}
                      direction="row"
                      spacing={2}
                    >
                      <Avatar src={admin.avatar} sx={{ width: 36, height: 36 }}>
                        {admin.name?.charAt(0)?.toUpperCase() ||
                          `${admin.firstName?.charAt(0) || ''}${admin.lastName?.charAt(0) || ''}`.toUpperCase()}
                      </Avatar>
                      <Stack sx={{ minWidth: 0 }}>
                        <Typography variant="subtitle2" noWrap>
                          {admin.name ||
                            `${admin.firstName || ''} ${admin.lastName || ''}`.trim()}
                        </Typography>
                        <Typography
                          color="text.secondary"
                          variant="body2"
                          noWrap
                        >
                          ID: {admin.id}
                        </Typography>
                      </Stack>
                    </Stack>
                  </TableCell>
                  <TableCell>
                    <Typography variant="body2">{admin.email}</Typography>
                    {admin.emailVerified && (
                      <Typography variant="caption" color="success.main">
                        Verified
                      </Typography>
                    )}
                  </TableCell>
                  <TableCell>
                    <Chip
                      icon={<RoleIcon size={16} />}
                      label={getRoleLabel(admin.serviceRole)}
                      color={getRoleColor(admin.serviceRole)}
                      size="small"
                      variant="outlined"
                    />
                  </TableCell>
                  <TableCell>
                    {admin.createdAt
                      ? dayjs(admin.createdAt).format('MMM D, YYYY')
                      : 'N/A'}
                  </TableCell>
                  <TableCell>
                    <Chip label="Active" color="success" size="small" />
                  </TableCell>
                  <TableCell align="right">
                    <IconButton onClick={(e) => handleMenuOpen(e, admin.id)}>
                      <DotsThreeIcon weight="bold" />
                    </IconButton>
                  </TableCell>
                </TableRow>
              );
            })}
          </TableBody>
        </Table>
      </Box>
      <Divider />
      <TablePagination
        component="div"
        count={count}
        onPageChange={(_, newPage) => onPageChange(newPage)}
        onRowsPerPageChange={(event) =>
          onRowsPerPageChange(parseInt(event.target.value, 10))
        }
        page={page}
        rowsPerPage={rowsPerPage}
        rowsPerPageOptions={[5, 10, 25]}
      />

      {/* Context Menu */}
      <Menu
        anchorEl={menuAnchor?.element}
        open={Boolean(menuAnchor)}
        onClose={handleMenuClose}
        anchorOrigin={{ vertical: 'bottom', horizontal: 'right' }}
        transformOrigin={{ vertical: 'top', horizontal: 'right' }}
      >
        <MenuItem onClick={() => handleCopyId(menuAnchor?.adminId || '')}>
          <CopyIcon size={16} style={{ marginRight: 8 }} />
          Copy ID
        </MenuItem>
        {menuAnchor &&
          canEditAdmin(
            rows.find((admin) => admin.id === menuAnchor.adminId)!,
          ) && (
            <MenuItem
              onClick={() =>
                handleEdit(
                  rows.find((admin) => admin.id === menuAnchor.adminId)!,
                )
              }
            >
              <PencilIcon size={16} style={{ marginRight: 8 }} />
              Edit
            </MenuItem>
          )}
        {menuAnchor &&
          canDeleteAdmin(
            rows.find((admin) => admin.id === menuAnchor.adminId)!,
          ) && (
            <MenuItem
              onClick={() =>
                handleDelete(
                  rows.find((admin) => admin.id === menuAnchor.adminId)!,
                )
              }
            >
              <TrashIcon size={16} style={{ marginRight: 8 }} />
              Delete
            </MenuItem>
          )}
      </Menu>
    </Card>
  );
}
