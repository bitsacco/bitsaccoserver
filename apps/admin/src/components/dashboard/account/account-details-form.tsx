'use client';

import * as React from 'react';
import Button from '@mui/material/Button';
import Card from '@mui/material/Card';
import CardActions from '@mui/material/CardActions';
import CardContent from '@mui/material/CardContent';
import CardHeader from '@mui/material/CardHeader';
import Divider from '@mui/material/Divider';
import FormControl from '@mui/material/FormControl';
import InputLabel from '@mui/material/InputLabel';
import MenuItem from '@mui/material/MenuItem';
import OutlinedInput from '@mui/material/OutlinedInput';
import Select from '@mui/material/Select';
import Grid from '@mui/material/Unstable_Grid2';
import Typography from '@mui/material/Typography';
import Alert from '@mui/material/Alert';

import { useUser } from '@/hooks/use-user';
import { ServiceRole } from '@bitsaccoserver/types';

const states = [
  { value: 'alabama', label: 'Alabama' },
  { value: 'new-york', label: 'New York' },
  { value: 'san-francisco', label: 'San Francisco' },
  { value: 'los-angeles', label: 'Los Angeles' },
] as const;

export function AccountDetailsForm(): React.JSX.Element {
  const { user } = useUser();

  if (!user) {
    return (
      <Card>
        <CardContent>
          <Typography variant="body2" color="text.secondary">
            Loading account details...
          </Typography>
        </CardContent>
      </Card>
    );
  }

  return (
    <form
      onSubmit={(event) => {
        event.preventDefault();
      }}
    >
      <Card>
        <CardHeader
          subheader="The information can be edited"
          title="Profile Details"
        />
        <Divider />
        <CardContent>
          <Alert severity="info" sx={{ mb: 3 }}>
            Account details are currently read-only. Contact your system
            administrator to update your information.
          </Alert>
          <Grid container spacing={3}>
            <Grid md={6} xs={12}>
              <FormControl fullWidth>
                <InputLabel>First name</InputLabel>
                <OutlinedInput
                  value={user.firstName || 'Not provided'}
                  label="First name"
                  name="firstName"
                  readOnly
                />
              </FormControl>
            </Grid>
            <Grid md={6} xs={12}>
              <FormControl fullWidth>
                <InputLabel>Last name</InputLabel>
                <OutlinedInput
                  value={user.lastName || 'Not provided'}
                  label="Last name"
                  name="lastName"
                  readOnly
                />
              </FormControl>
            </Grid>
            <Grid md={6} xs={12}>
              <FormControl fullWidth>
                <InputLabel>Email address</InputLabel>
                <OutlinedInput
                  value={user.email}
                  label="Email address"
                  name="email"
                  readOnly
                />
              </FormControl>
            </Grid>
            <Grid md={6} xs={12}>
              <FormControl fullWidth>
                <InputLabel>Phone number</InputLabel>
                <OutlinedInput
                  value={user.phone || user.phoneNumber || ''}
                  label="Phone number"
                  name="phone"
                  type="tel"
                  readOnly
                />
              </FormControl>
            </Grid>
            <Grid md={6} xs={12}>
              <FormControl fullWidth>
                <InputLabel>Service Role</InputLabel>
                <OutlinedInput
                  value={user.serviceRole}
                  label="Service Role"
                  name="serviceRole"
                  readOnly
                />
              </FormControl>
            </Grid>
            <Grid md={6} xs={12}>
              <FormControl fullWidth>
                <InputLabel>Email Verified</InputLabel>
                <OutlinedInput
                  value={user.emailVerified ? 'Yes' : 'No'}
                  label="Email Verified"
                  name="emailVerified"
                  readOnly
                />
              </FormControl>
            </Grid>
            {user.createdAt && (
              <Grid md={6} xs={12}>
                <FormControl fullWidth>
                  <InputLabel>Account Created</InputLabel>
                  <OutlinedInput
                    value={new Date(user.createdAt).toLocaleDateString()}
                    label="Account Created"
                    name="createdAt"
                    readOnly
                  />
                </FormControl>
              </Grid>
            )}
            {user.updatedAt && (
              <Grid md={6} xs={12}>
                <FormControl fullWidth>
                  <InputLabel>Last Updated</InputLabel>
                  <OutlinedInput
                    value={new Date(user.updatedAt).toLocaleDateString()}
                    label="Last Updated"
                    name="updatedAt"
                    readOnly
                  />
                </FormControl>
              </Grid>
            )}
          </Grid>
        </CardContent>
      </Card>
    </form>
  );
}
