'use client';

import * as React from 'react';
import Avatar from '@mui/material/Avatar';
import Button from '@mui/material/Button';
import Card from '@mui/material/Card';
import CardActions from '@mui/material/CardActions';
import CardContent from '@mui/material/CardContent';
import Divider from '@mui/material/Divider';
import Stack from '@mui/material/Stack';
import Typography from '@mui/material/Typography';
import Chip from '@mui/material/Chip';

import { useUser } from '@/hooks/use-user';
import { ServiceRole } from '@bitsaccoserver/types';
import { getUserDisplayName, getUserInitial } from '@/lib/utils/user';

export function AccountInfo(): React.JSX.Element {
  const { user } = useUser();

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

  const getRoleLabel = (role: ServiceRole) => {
    switch (role) {
      case ServiceRole.SYSTEM_ADMIN:
        return 'System Administrator';
      case ServiceRole.ADMIN:
        return 'Administrator';
      case ServiceRole.MEMBER:
        return 'Member';
      default:
        return 'Unknown';
    }
  };

  if (!user) {
    return (
      <Card>
        <CardContent>
          <Typography
            variant="body2"
            color="text.secondary"
            sx={{ textAlign: 'center' }}
          >
            Loading account information...
          </Typography>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card>
      <CardContent>
        <Stack spacing={2} sx={{ alignItems: 'center' }}>
          <div>
            <Avatar
              src={user.avatar || user.profilePicture}
              sx={{ height: '80px', width: '80px' }}
            >
              {getUserInitial(user)}
            </Avatar>
          </div>
          <Stack spacing={1} sx={{ textAlign: 'center' }}>
            <Typography variant="h5">{getUserDisplayName(user)}</Typography>
            <Typography color="text.secondary" variant="body2">
              {user.email}
            </Typography>
            {user.phone && (
              <Typography color="text.secondary" variant="body2">
                {user.phone}
              </Typography>
            )}
            <Chip
              label={getRoleLabel(user.serviceRole)}
              color={getRoleColor(user.serviceRole) as any}
              size="small"
            />
            {user.emailVerified && (
              <Chip
                label="Email Verified"
                color="success"
                size="small"
                variant="outlined"
              />
            )}
          </Stack>
        </Stack>
      </CardContent>
      <Divider />
      <CardActions>
        <Button fullWidth variant="text">
          Upload picture
        </Button>
      </CardActions>
    </Card>
  );
}
