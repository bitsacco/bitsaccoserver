'use client';

import * as React from 'react';
import Alert from '@mui/material/Alert';
import Box from '@mui/material/Box';
import Button from '@mui/material/Button';
import Card from '@mui/material/Card';
import CardContent from '@mui/material/CardContent';
import Stack from '@mui/material/Stack';
import Typography from '@mui/material/Typography';
import { Shield as ShieldIcon } from '@phosphor-icons/react/dist/ssr/Shield';
import { User as UserIcon } from '@phosphor-icons/react/dist/ssr/User';

import { ServiceRole } from '@bitsaccoserver/types';
import { useUser } from '@/hooks/use-user';

interface PermissionGuardProps {
  children: React.ReactNode;
  requiredRole?: ServiceRole;
  allowedRoles?: ServiceRole[];
  fallback?: React.ReactNode;
}

function InsufficientPermissionsError({
  requiredRole,
  allowedRoles,
  userRole,
}: {
  requiredRole?: ServiceRole;
  allowedRoles?: ServiceRole[];
  userRole?: ServiceRole;
}) {
  const getRequiredRoleText = () => {
    if (requiredRole) {
      return requiredRole === ServiceRole.SYSTEM_ADMIN
        ? 'Super Admin'
        : 'Admin';
    }
    if (allowedRoles) {
      const roleNames = allowedRoles.map((role) =>
        role === ServiceRole.SYSTEM_ADMIN
          ? 'Super Admin'
          : role === ServiceRole.ADMIN
            ? 'Admin'
            : 'Member',
      );
      return roleNames.join(' or ');
    }
    return 'Admin';
  };

  const getCurrentRoleText = () => {
    if (!userRole) return 'Unknown';
    return userRole === ServiceRole.SYSTEM_ADMIN
      ? 'Super Admin'
      : userRole === ServiceRole.ADMIN
        ? 'Admin'
        : 'Member';
  };

  return (
    <Box
      sx={{
        display: 'flex',
        justifyContent: 'center',
        alignItems: 'center',
        minHeight: '60vh',
        p: 3,
      }}
    >
      <Card sx={{ maxWidth: 500, width: '100%' }}>
        <CardContent>
          <Stack spacing={3} alignItems="center" textAlign="center">
            <Box
              sx={{
                display: 'flex',
                justifyContent: 'center',
                alignItems: 'center',
                width: 64,
                height: 64,
                borderRadius: '50%',
                bgcolor: 'error.light',
                color: 'error.contrastText',
              }}
            >
              <ShieldIcon size={32} />
            </Box>

            <Stack spacing={1}>
              <Typography variant="h5" color="error">
                Insufficient Permissions
              </Typography>
              <Typography color="text.secondary">
                You don't have the required permissions to access this page.
              </Typography>
            </Stack>

            <Alert severity="info" sx={{ width: '100%' }}>
              <Stack spacing={1}>
                <Typography variant="body2">
                  <strong>Required role:</strong> {getRequiredRoleText()}
                </Typography>
                <Typography variant="body2">
                  <strong>Your current role:</strong> {getCurrentRoleText()}
                </Typography>
              </Stack>
            </Alert>

            <Stack spacing={2} sx={{ width: '100%' }}>
              <Typography variant="body2" color="text.secondary">
                To access this information or perform these actions, please
                contact a {getRequiredRoleText()}
                who can either provide you with the information you need or
                upgrade your account permissions.
              </Typography>

              <Box
                sx={{
                  display: 'flex',
                  gap: 1,
                  justifyContent: 'center',
                  flexWrap: 'wrap',
                }}
              >
                <Button
                  variant="outlined"
                  startIcon={<UserIcon />}
                  onClick={() => window.history.back()}
                >
                  Go Back
                </Button>
                <Button variant="contained" href="/dashboard">
                  Return to Dashboard
                </Button>
              </Box>
            </Stack>
          </Stack>
        </CardContent>
      </Card>
    </Box>
  );
}

export function PermissionGuard({
  children,
  requiredRole,
  allowedRoles,
  fallback,
}: PermissionGuardProps): React.JSX.Element {
  const { user, isLoading, error } = useUser();

  // Show loading state while checking user
  if (isLoading) {
    return (
      <Box sx={{ display: 'flex', justifyContent: 'center', p: 3 }}>
        <Typography>Loading...</Typography>
      </Box>
    );
  }

  // Show error if user fetch failed
  if (error || !user) {
    return (
      <Box sx={{ display: 'flex', justifyContent: 'center', p: 3 }}>
        <Alert severity="error">
          Unable to verify user permissions. Please sign in again.
        </Alert>
      </Box>
    );
  }

  // Check permissions
  const hasPermission = () => {
    if (!user.serviceRole) return false;

    // Check required role (exact match or higher)
    if (requiredRole) {
      if (requiredRole === ServiceRole.SYSTEM_ADMIN) {
        return user.serviceRole === ServiceRole.SYSTEM_ADMIN;
      }
      if (requiredRole === ServiceRole.ADMIN) {
        return (
          user.serviceRole === ServiceRole.SYSTEM_ADMIN ||
          user.serviceRole === ServiceRole.ADMIN
        );
      }
      return true; // For MEMBER role, everyone has access
    }

    // Check allowed roles (explicit list)
    if (allowedRoles && allowedRoles.length > 0) {
      return allowedRoles.includes(user.serviceRole);
    }

    // Default: allow access if no restrictions specified
    return true;
  };

  if (!hasPermission()) {
    return fallback ? (
      <>{fallback}</>
    ) : (
      <InsufficientPermissionsError
        requiredRole={requiredRole}
        allowedRoles={allowedRoles}
        userRole={user.serviceRole}
      />
    );
  }

  return <>{children}</>;
}
