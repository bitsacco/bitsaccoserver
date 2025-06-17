import * as React from 'react';
import type { Metadata } from 'next';
import Stack from '@mui/material/Stack';
import Typography from '@mui/material/Typography';

import { config } from '@/config';
import { AccountSection } from '@/components/dashboard/settings/account-section';
import { UpdatePasswordForm } from '@/components/dashboard/settings/update-password-form';
import { Notifications } from '@/components/dashboard/settings/notifications';

export const metadata = {
  title: `Settings | Dashboard | ${config.site.name}`,
} satisfies Metadata;

export default function Page(): React.JSX.Element {
  return (
    <Stack spacing={3}>
      <div>
        <Typography variant="h4">Settings</Typography>
      </div>
      <AccountSection />
      <UpdatePasswordForm />
      <Notifications />
    </Stack>
  );
}
