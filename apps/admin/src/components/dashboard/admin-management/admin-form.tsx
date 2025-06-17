'use client';

import * as React from 'react';
import Button from '@mui/material/Button';
import Dialog from '@mui/material/Dialog';
import DialogActions from '@mui/material/DialogActions';
import DialogContent from '@mui/material/DialogContent';
import DialogTitle from '@mui/material/DialogTitle';
import FormControl from '@mui/material/FormControl';
import FormHelperText from '@mui/material/FormHelperText';
import InputLabel from '@mui/material/InputLabel';
import OutlinedInput from '@mui/material/OutlinedInput';
import Select from '@mui/material/Select';
import MenuItem from '@mui/material/MenuItem';
import Stack from '@mui/material/Stack';
import Alert from '@mui/material/Alert';
import Typography from '@mui/material/Typography';
import { Controller, useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { z as zod } from 'zod';

import { User, ServiceRole } from '@bitsaccoserver/types';
import { useUser } from '@/hooks/use-user';

// Define the schema for admin form validation
const adminSchema = zod.object({
  firstName: zod.string().min(1, { message: 'First name is required' }),
  lastName: zod.string().min(1, { message: 'Last name is required' }),
  email: zod
    .string()
    .min(1, { message: 'Email is required' })
    .email('Invalid email format'),
  phone: zod.string().optional(),
  serviceRole: zod.enum([ServiceRole.ADMIN, ServiceRole.SYSTEM_ADMIN], {
    message: 'Please select a valid admin role',
  }),
});

type AdminFormValues = zod.infer<typeof adminSchema>;

export interface AdminFormProps {
  open?: boolean;
  onClose?: () => void;
  onSubmit?: (values: AdminFormValues) => void;
  admin?: User | null;
  isLoading?: boolean;
  error?: string | null;
}

export function AdminForm({
  open = false,
  onClose,
  onSubmit,
  admin,
  isLoading = false,
  error,
}: AdminFormProps): React.JSX.Element {
  const { user: currentUser } = useUser();
  const isEditing = Boolean(admin);

  const {
    control,
    handleSubmit,
    formState: { errors, isSubmitting },
    reset,
    watch,
  } = useForm<AdminFormValues>({
    resolver: zodResolver(adminSchema),
    defaultValues: {
      firstName: '',
      lastName: '',
      email: '',
      phone: '',
      serviceRole: ServiceRole.ADMIN,
    },
  });

  const selectedRole = watch('serviceRole');

  // Reset form when admin changes or dialog opens/closes
  React.useEffect(() => {
    if (open) {
      if (admin) {
        reset({
          firstName: admin.firstName || '',
          lastName: admin.lastName || '',
          email: admin.email || '',
          phone: admin.phone || '',
          serviceRole: admin.serviceRole as
            | ServiceRole.ADMIN
            | ServiceRole.SYSTEM_ADMIN,
        });
      } else {
        reset({
          firstName: '',
          lastName: '',
          email: '',
          phone: '',
          serviceRole: ServiceRole.ADMIN,
        });
      }
    }
  }, [admin, open, reset]);

  const handleFormSubmit = React.useCallback(
    (values: AdminFormValues) => {
      onSubmit?.(values);
    },
    [onSubmit],
  );

  const handleCancel = () => {
    reset();
    onClose?.();
  };

  // Check if current user can assign System Admin role
  const canAssignSystemAdmin =
    currentUser?.serviceRole === ServiceRole.SYSTEM_ADMIN;

  // Check if current user can edit this admin
  const canEdit = React.useMemo(() => {
    if (!currentUser || !admin) return true; // Allow for new admin creation
    if (currentUser.serviceRole !== ServiceRole.SYSTEM_ADMIN) return false;
    if (currentUser.id === admin.id) return false; // Can't edit self
    return true;
  }, [currentUser, admin]);

  if (!canEdit) {
    return (
      <Dialog open={open} onClose={onClose} maxWidth="sm" fullWidth>
        <DialogContent>
          <Alert severity="error">
            You don't have permission to edit this administrator.
          </Alert>
        </DialogContent>
        <DialogActions>
          <Button onClick={onClose}>Close</Button>
        </DialogActions>
      </Dialog>
    );
  }

  return (
    <Dialog open={open} onClose={onClose} maxWidth="sm" fullWidth>
      <form onSubmit={handleSubmit(handleFormSubmit)}>
        <DialogTitle>
          {isEditing ? 'Edit Administrator' : 'Add New Administrator'}
        </DialogTitle>
        <DialogContent>
          <Stack spacing={2} sx={{ pt: 1 }}>
            {error && <Alert severity="error">{error}</Alert>}

            <Stack direction="row" spacing={2}>
              <Controller
                control={control}
                name="firstName"
                render={({ field }) => (
                  <FormControl error={Boolean(errors.firstName)} fullWidth>
                    <InputLabel>First Name</InputLabel>
                    <OutlinedInput {...field} label="First Name" />
                    {errors.firstName && (
                      <FormHelperText>
                        {errors.firstName.message}
                      </FormHelperText>
                    )}
                  </FormControl>
                )}
              />
              <Controller
                control={control}
                name="lastName"
                render={({ field }) => (
                  <FormControl error={Boolean(errors.lastName)} fullWidth>
                    <InputLabel>Last Name</InputLabel>
                    <OutlinedInput {...field} label="Last Name" />
                    {errors.lastName && (
                      <FormHelperText>{errors.lastName.message}</FormHelperText>
                    )}
                  </FormControl>
                )}
              />
            </Stack>

            <Controller
              control={control}
              name="email"
              render={({ field }) => (
                <FormControl error={Boolean(errors.email)} fullWidth>
                  <InputLabel>Email</InputLabel>
                  <OutlinedInput
                    {...field}
                    label="Email"
                    type="email"
                    disabled={isEditing} // Email cannot be changed when editing
                  />
                  {errors.email && (
                    <FormHelperText>{errors.email.message}</FormHelperText>
                  )}
                  {isEditing && (
                    <FormHelperText>
                      Email cannot be changed after creation
                    </FormHelperText>
                  )}
                </FormControl>
              )}
            />

            <Controller
              control={control}
              name="phone"
              render={({ field }) => (
                <FormControl fullWidth>
                  <InputLabel>Phone (Optional)</InputLabel>
                  <OutlinedInput
                    {...field}
                    label="Phone (Optional)"
                    type="tel"
                  />
                </FormControl>
              )}
            />

            <Controller
              control={control}
              name="serviceRole"
              render={({ field }) => (
                <FormControl error={Boolean(errors.serviceRole)} fullWidth>
                  <InputLabel>Admin Role</InputLabel>
                  <Select {...field} label="Admin Role">
                    <MenuItem value={ServiceRole.ADMIN}>
                      <Stack>
                        <Typography>Admin</Typography>
                        <Typography variant="caption" color="text.secondary">
                          Can manage members and view system data
                        </Typography>
                      </Stack>
                    </MenuItem>
                    {canAssignSystemAdmin && (
                      <MenuItem value={ServiceRole.SYSTEM_ADMIN}>
                        <Stack>
                          <Typography>System Admin</Typography>
                          <Typography variant="caption" color="text.secondary">
                            Full system access and admin management
                          </Typography>
                        </Stack>
                      </MenuItem>
                    )}
                  </Select>
                  {errors.serviceRole && (
                    <FormHelperText>
                      {errors.serviceRole.message}
                    </FormHelperText>
                  )}
                  {!canAssignSystemAdmin && (
                    <FormHelperText>
                      Only System Admins can assign System Admin role
                    </FormHelperText>
                  )}
                </FormControl>
              )}
            />

            {selectedRole === ServiceRole.SYSTEM_ADMIN && (
              <Alert severity="warning">
                <Typography variant="body2">
                  <strong>Warning:</strong> System Administrators have full
                  access to all system functions including user management,
                  system configuration, and sensitive data. Only assign this
                  role to trusted individuals.
                </Typography>
              </Alert>
            )}
          </Stack>
        </DialogContent>
        <DialogActions>
          <Button onClick={handleCancel} disabled={isLoading}>
            Cancel
          </Button>
          <Button
            type="submit"
            variant="contained"
            disabled={isLoading || isSubmitting}
          >
            {isLoading || isSubmitting
              ? 'Saving...'
              : isEditing
                ? 'Update'
                : 'Create'}
          </Button>
        </DialogActions>
      </form>
    </Dialog>
  );
}
