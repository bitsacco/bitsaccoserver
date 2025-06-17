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
import dayjs from 'dayjs';

import { useSelection } from '@/hooks/use-selection';
import { Member, ServiceRole } from '@bitsaccoserver/types';
import { useUser } from '@/hooks/use-user';

interface MembersTableProps {
  count: number;
  page: number;
  rows: Member[];
  rowsPerPage: number;
  isLoading?: boolean;
  error?: string | null;
  onPageChange: (page: number) => void;
  onRowsPerPageChange: (rowsPerPage: number) => void;
  onEdit?: (member: Member) => void;
  onDelete?: (member: Member) => void;
  onCopyId?: (id: string) => void;
}

export function MembersTable({
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
}: MembersTableProps): React.JSX.Element {
  const { user: currentUser } = useUser();
  const rowIds = React.useMemo(() => {
    return rows.map((member) => member.id);
  }, [rows]);

  const { deselectAll, deselectOne, selectAll, selectOne, selected } =
    useSelection(rowIds);

  const selectedSome =
    (selected?.size ?? 0) > 0 && (selected?.size ?? 0) < rows.length;
  const selectedAll = rows.length > 0 && selected?.size === rows.length;

  const [menuAnchor, setMenuAnchor] = React.useState<{
    element: HTMLElement;
    memberId: string;
  } | null>(null);

  const handleMenuOpen = (
    event: React.MouseEvent<HTMLElement>,
    memberId: string,
  ) => {
    setMenuAnchor({ element: event.currentTarget, memberId });
  };

  const handleMenuClose = () => {
    setMenuAnchor(null);
  };

  const handleEdit = (member: Member) => {
    handleMenuClose();
    onEdit?.(member);
  };

  const handleDelete = (member: Member) => {
    handleMenuClose();
    onDelete?.(member);
  };

  const handleCopyId = (memberId: string) => {
    handleMenuClose();
    onCopyId?.(memberId);
  };

  const canEditMember = (member: Member) => {
    if (!currentUser) return false;
    // System Admins and Admins can edit members
    return (
      currentUser.serviceRole === ServiceRole.SYSTEM_ADMIN ||
      currentUser.serviceRole === ServiceRole.ADMIN
    );
  };

  const canDeleteMember = (member: Member) => {
    if (!currentUser) return false;
    // Only System Admins can delete members
    return currentUser.serviceRole === ServiceRole.SYSTEM_ADMIN;
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
              <TableCell>Member</TableCell>
              <TableCell>Email</TableCell>
              <TableCell>Phone</TableCell>
              <TableCell>Created</TableCell>
              <TableCell>Status</TableCell>
              <TableCell align="right">Actions</TableCell>
            </TableRow>
          </TableHead>
          <TableBody>
            {rows.map((member) => {
              const isSelected = selected?.has(member.id);

              return (
                <TableRow hover key={member.id} selected={isSelected}>
                  <TableCell padding="checkbox">
                    <Checkbox
                      checked={isSelected}
                      onChange={(event) => {
                        if (event.target.checked) {
                          selectOne(member.id);
                        } else {
                          deselectOne(member.id);
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
                      <Avatar
                        src={member.avatar}
                        sx={{ width: 36, height: 36 }}
                      >
                        {member.name?.charAt(0)?.toUpperCase() ||
                          `${member.firstName?.charAt(0) || ''}${member.lastName?.charAt(0) || ''}`.toUpperCase()}
                      </Avatar>
                      <Stack sx={{ minWidth: 0 }}>
                        <Typography variant="subtitle2" noWrap>
                          {member.name ||
                            `${member.firstName || ''} ${member.lastName || ''}`.trim()}
                        </Typography>
                        <Typography
                          color="text.secondary"
                          variant="body2"
                          noWrap
                        >
                          ID: {member.id}
                        </Typography>
                      </Stack>
                    </Stack>
                  </TableCell>
                  <TableCell>
                    <Typography variant="body2">{member.email}</Typography>
                    {member.emailVerified && (
                      <Typography variant="caption" color="success.main">
                        Verified
                      </Typography>
                    )}
                  </TableCell>
                  <TableCell>
                    <Typography variant="body2">
                      {member.phone || '-'}
                    </Typography>
                  </TableCell>
                  <TableCell>
                    {member.createdAt
                      ? dayjs(member.createdAt).format('MMM D, YYYY')
                      : 'N/A'}
                  </TableCell>
                  <TableCell>
                    <Chip label="Active" color="success" size="small" />
                  </TableCell>
                  <TableCell align="right">
                    <IconButton onClick={(e) => handleMenuOpen(e, member.id)}>
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
        <MenuItem onClick={() => handleCopyId(menuAnchor?.memberId || '')}>
          <CopyIcon size={16} style={{ marginRight: 8 }} />
          Copy ID
        </MenuItem>
        {menuAnchor &&
          canEditMember(
            rows.find((member) => member.id === menuAnchor.memberId)!,
          ) && (
            <MenuItem
              onClick={() =>
                handleEdit(
                  rows.find((member) => member.id === menuAnchor.memberId)!,
                )
              }
            >
              <PencilIcon size={16} style={{ marginRight: 8 }} />
              Edit
            </MenuItem>
          )}
        {menuAnchor &&
          canDeleteMember(
            rows.find((member) => member.id === menuAnchor.memberId)!,
          ) && (
            <MenuItem
              onClick={() =>
                handleDelete(
                  rows.find((member) => member.id === menuAnchor.memberId)!,
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
