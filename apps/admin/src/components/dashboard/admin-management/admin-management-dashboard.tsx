'use client';

import * as React from 'react';
import Button from '@mui/material/Button';
import Stack from '@mui/material/Stack';
import Typography from '@mui/material/Typography';
import Snackbar from '@mui/material/Snackbar';
import Alert from '@mui/material/Alert';
import { Download as DownloadIcon } from '@phosphor-icons/react/dist/ssr/Download';
import { Plus as PlusIcon } from '@phosphor-icons/react/dist/ssr/Plus';
import { Upload as UploadIcon } from '@phosphor-icons/react/dist/ssr/Upload';
import CircularProgress from '@mui/material/CircularProgress';

import { User, ServiceRole } from '@bitsaccoserver/types';
import { useAdmins } from '@/hooks/use-admins';
import { adminsClient } from '@/lib/admins/client';
import { AdminsFilters } from '@/components/dashboard/admin-management/admins-filters';
import { AdminsTable } from '@/components/dashboard/admin-management/admins-table';
import { AdminForm } from '@/components/dashboard/admin-management/admin-form';
import { DeleteConfirmDialog } from '@/components/dashboard/member-management/delete-confirm-dialog';
import { logger } from '@/lib/default-logger';
import { useUser } from '@/hooks/use-user';

// Simple loading component
function LoadingState() {
  return (
    <Stack
      spacing={3}
      alignItems="center"
      justifyContent="center"
      sx={{ py: 8 }}
    >
      <CircularProgress />
      <Typography>Loading admin data...</Typography>
    </Stack>
  );
}

// ErrorState component
function ErrorState({ message }: { message: string }) {
  return (
    <Stack
      spacing={3}
      alignItems="center"
      justifyContent="center"
      sx={{ py: 8 }}
    >
      <Alert severity="error" sx={{ width: '100%', maxWidth: 500 }}>
        {message}
      </Alert>
    </Stack>
  );
}

export function AdminManagementDashboard(): React.JSX.Element {
  const { user, isLoading: isUserLoading, error: userError } = useUser();

  // Check for errors and loading state
  if (isUserLoading) {
    return <LoadingState />;
  }

  if (userError) {
    return <ErrorState message={userError} />;
  }

  // Check if user exists and is System Admin
  if (!user || user.serviceRole !== ServiceRole.SYSTEM_ADMIN) {
    return (
      <ErrorState message="Access denied. System Administrator role required." />
    );
  }

  // Admins state management with custom hook
  const {
    admins,
    totalCount,
    isLoading: isLoadingAdmins,
    error: adminsError,
    refetch,
    search,
    setPage,
    setLimit,
    setSort,
  } = useAdmins();

  // UI state
  const [page, setPageLocal] = React.useState(0);
  const [rowsPerPage, setRowsPerPageLocal] = React.useState(10);

  // Dialog state
  const [adminFormOpen, setAdminFormOpen] = React.useState(false);
  const [selectedAdmin, setSelectedAdmin] = React.useState<User | null>(null);
  const [deleteDialogOpen, setDeleteDialogOpen] = React.useState(false);
  const [adminToDelete, setAdminToDelete] = React.useState<User | null>(null);

  // Operation state
  const [formSubmitting, setFormSubmitting] = React.useState(false);
  const [deleteLoading, setDeleteLoading] = React.useState(false);
  const [formError, setFormError] = React.useState<string | null>(null);
  const [deleteError, setDeleteError] = React.useState<string | null>(null);

  // Notification state
  const [notification, setNotification] = React.useState<{
    open: boolean;
    message: string;
    severity: 'success' | 'error' | 'info' | 'warning';
  }>({
    open: false,
    message: '',
    severity: 'success',
  });

  // Handle copying admin ID
  const handleCopyAdminId = (id: string) => {
    navigator.clipboard.writeText(id);
    setNotification({
      open: true,
      message: 'Admin ID copied to clipboard',
      severity: 'success',
    });
  };

  // Handlers for pagination
  const handlePageChange = (newPage: number) => {
    setPageLocal(newPage);
    setPage(newPage);
  };

  const handleRowsPerPageChange = (newRowsPerPage: number) => {
    setRowsPerPageLocal(newRowsPerPage);
    setLimit(newRowsPerPage);
  };

  // Handlers for search and sort
  const handleSearch = (term: string) => {
    search(term);
  };

  const handleSort = (
    field: 'name' | 'email' | 'createdAt' | 'updatedAt',
    order: 'asc' | 'desc',
  ) => {
    setSort(field, order);
  };

  // Admin form handlers
  const handleAddAdmin = () => {
    setSelectedAdmin(null);
    setFormError(null);
    setAdminFormOpen(true);
  };

  const handleEditAdmin = (admin: User) => {
    setSelectedAdmin(admin);
    setFormError(null);
    setAdminFormOpen(true);
  };

  const handleAdminFormClose = () => {
    setAdminFormOpen(false);
  };

  const handleAdminFormSubmit = async (data: any) => {
    setFormSubmitting(true);
    setFormError(null);

    try {
      if (selectedAdmin) {
        // Update existing admin
        await adminsClient.updateAdmin(selectedAdmin.id, data);

        setNotification({
          open: true,
          message: 'Admin updated successfully',
          severity: 'success',
        });
      } else {
        // Create new admin
        await adminsClient.createAdmin(data);

        setNotification({
          open: true,
          message: 'Admin created successfully',
          severity: 'success',
        });
      }

      setAdminFormOpen(false);
      refetch();
    } catch (err) {
      logger.error('Admin form submission error:', err);
      setFormError('An unexpected error occurred. Please try again.');
    } finally {
      setFormSubmitting(false);
    }
  };

  // Delete admin handlers
  const handleDeleteClick = (admin: User) => {
    setAdminToDelete(admin);
    setDeleteError(null);
    setDeleteDialogOpen(true);
  };

  const handleDeleteClose = () => {
    setDeleteDialogOpen(false);
  };

  const handleDeleteConfirm = async () => {
    if (!adminToDelete) return;

    setDeleteLoading(true);
    setDeleteError(null);

    try {
      await adminsClient.deleteAdmin(adminToDelete.id);

      setNotification({
        open: true,
        message: 'Admin deleted successfully',
        severity: 'success',
      });

      setDeleteDialogOpen(false);
      refetch();
    } catch (err) {
      logger.error('Delete admin error:', err);
      setDeleteError('An unexpected error occurred. Please try again.');
    } finally {
      setDeleteLoading(false);
    }
  };

  // Export handlers
  const handleExport = () => {
    if (admins.length === 0) {
      setNotification({
        open: true,
        message: 'No admins to export',
        severity: 'info',
      });
      return;
    }

    try {
      // Simple JSON export
      const exportData = JSON.stringify(admins, null, 2);
      const blob = new Blob([exportData], { type: 'application/json' });
      const url = URL.createObjectURL(blob);

      const link = document.createElement('a');
      link.href = url;
      link.download = `admins-export-${new Date().toISOString().split('T')[0]}.json`;
      document.body.appendChild(link);
      link.click();

      setTimeout(() => {
        URL.revokeObjectURL(url);
        document.body.removeChild(link);
      }, 100);

      setNotification({
        open: true,
        message: `${admins.length} admins exported successfully`,
        severity: 'success',
      });
    } catch (err) {
      logger.error('Export admins error:', err);
      setNotification({
        open: true,
        message: 'Failed to export admins',
        severity: 'error',
      });
    }
  };

  const handleCloseNotification = () => {
    setNotification((prev) => ({ ...prev, open: false }));
  };

  return (
    <Stack spacing={3}>
      <Stack direction="row" spacing={3}>
        <Stack spacing={1} sx={{ flex: '1 1 auto' }}>
          <Typography variant="h4">Admin Management</Typography>
          <Typography color="text.secondary">
            Manage System Administrators and their permissions
          </Typography>
          <Stack direction="row" spacing={1} sx={{ alignItems: 'center' }}>
            <Button
              color="inherit"
              startIcon={<DownloadIcon fontSize="var(--icon-fontSize-md)" />}
              onClick={handleExport}
            >
              Export
            </Button>
          </Stack>
        </Stack>
        <div>
          <Button
            startIcon={<PlusIcon fontSize="var(--icon-fontSize-md)" />}
            variant="contained"
            onClick={handleAddAdmin}
          >
            Add Admin
          </Button>
        </div>
      </Stack>

      <AdminsFilters onSearch={handleSearch} onSort={handleSort} />

      <AdminsTable
        count={totalCount}
        page={page}
        rows={admins}
        rowsPerPage={rowsPerPage}
        isLoading={isLoadingAdmins}
        error={adminsError}
        onPageChange={handlePageChange}
        onRowsPerPageChange={handleRowsPerPageChange}
        onEdit={handleEditAdmin}
        onDelete={handleDeleteClick}
        onCopyId={handleCopyAdminId}
      />

      {/* Admin Add/Edit Form */}
      <AdminForm
        open={adminFormOpen}
        onClose={handleAdminFormClose}
        onSubmit={handleAdminFormSubmit}
        admin={selectedAdmin}
        isLoading={formSubmitting}
        error={formError}
      />

      {/* Delete Confirmation Dialog */}
      <DeleteConfirmDialog
        open={deleteDialogOpen}
        onClose={handleDeleteClose}
        onConfirm={handleDeleteConfirm}
        isLoading={deleteLoading}
        error={deleteError}
        title="Delete Admin"
        description="Are you sure you want to delete this administrator? This action cannot be undone and will revoke all administrative privileges."
        itemName={
          adminToDelete?.name ||
          `${adminToDelete?.firstName} ${adminToDelete?.lastName}`
        }
      />

      {/* Notification Snackbar */}
      <Snackbar
        open={notification.open}
        autoHideDuration={5000}
        onClose={handleCloseNotification}
        anchorOrigin={{ vertical: 'bottom', horizontal: 'right' }}
      >
        <Alert
          onClose={handleCloseNotification}
          severity={notification.severity}
          sx={{ width: '100%' }}
        >
          {notification.message}
        </Alert>
      </Snackbar>
    </Stack>
  );
}
