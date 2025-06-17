import * as React from 'react';
import RouterLink from 'next/link';
import Box from '@mui/material/Box';
import Stack from '@mui/material/Stack';
import Typography from '@mui/material/Typography';

import { paths } from '@/paths';
import { DynamicLogo } from '@/components/core/logo';

export interface LayoutProps {
  children: React.ReactNode;
}

export function Layout({ children }: LayoutProps): React.JSX.Element {
  return (
    <Box
      sx={{
        display: { xs: 'flex', lg: 'grid' },
        flexDirection: 'column',
        gridTemplateColumns: '1fr 1fr',
        minHeight: '100%',
      }}
    >
      <Box sx={{ display: 'flex', flex: '1 1 auto', flexDirection: 'column' }}>
        <Box
          component={RouterLink}
          href={paths.home}
          sx={{ p: 3, display: 'flex', flexDirection: 'row' }}
        >
          <DynamicLogo colorDark="light" colorLight="dark" height={80} />
        </Box>
        <Box
          sx={{
            alignItems: 'center',
            display: 'flex',
            flex: '1 1 auto',
            justifyContent: 'center',
            p: 3,
          }}
        >
          <Box sx={{ maxWidth: '450px', width: '100%' }}>{children}</Box>
        </Box>
      </Box>
      <Box
        sx={{
          alignItems: 'center',
          background:
            'radial-gradient(50% 50% at 50% 50%, #122647 0%, #090E23 100%)',
          color: 'var(--mui-palette-common-white)',
          display: { xs: 'none', lg: 'flex' },
          justifyContent: 'center',
          p: 3,
        }}
      >
        <Stack spacing={3}>
          <Stack spacing={1}>
            <Typography
              color="inherit"
              sx={{ fontSize: '32px', lineHeight: '32px', textAlign: 'center' }}
              variant="h1"
            >
              <Box component="span" sx={{ color: '#15b79e' }}>
                Admin Dashboard
              </Box>
            </Typography>
            <Typography
              align="center"
              variant="subtitle1"
              sx={{ fontSize: '22px', lineHeight: '24px', textAlign: 'center' }}
            >
              Gateway to Community Service
            </Typography>
          </Stack>
          <Box sx={{ display: 'flex', justifyContent: 'center' }}>
            <Box
              component="img"
              alt="Widgets"
              src="/assets/svg/bitsaccoserver.svg"
              sx={{
                height: 'auto',
                width: '100%',
                maxWidth: '600px',
                opacity: '5%',
              }}
            />
          </Box>
        </Stack>
      </Box>
    </Box>
  );
}
