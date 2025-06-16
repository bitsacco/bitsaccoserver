'use client';

import * as React from 'react';
import { useSearchParams } from 'next/navigation';
import {
  Stack,
  CircularProgress,
  Typography,
  Alert,
  Button,
} from '@mui/material';
import RouterLink from 'next/link';

import { authClient } from '@/lib/auth/client';
import { paths } from '@/paths';

function VerifyContent(): React.JSX.Element {
  const searchParams = useSearchParams();
  const token = searchParams.get('token');

  const [isVerifying, setIsVerifying] = React.useState<boolean>(true);
  const [verificationResult, setVerificationResult] = React.useState<{
    success: boolean;
    message: string;
  } | null>(null);

  React.useEffect(() => {
    const verifyEmail = async () => {
      if (!token) {
        setVerificationResult({
          success: false,
          message: 'Invalid verification link. No token provided.',
        });
        setIsVerifying(false);
        return;
      }

      try {
        const { error } = await authClient.verifyEmail({ token });

        if (error) {
          setVerificationResult({
            success: false,
            message: error,
          });
        } else {
          setVerificationResult({
            success: true,
            message:
              'Your email has been successfully verified! You can now sign in.',
          });
        }
      } catch (err) {
        console.error('Email verification error:', err);
        setVerificationResult({
          success: false,
          message: 'An unexpected error occurred during verification.',
        });
      } finally {
        setIsVerifying(false);
      }
    };

    verifyEmail();
  }, [token]);

  if (isVerifying) {
    return (
      <Stack spacing={3} alignItems="center">
        <CircularProgress />
        <Typography variant="h5">Verifying your email...</Typography>
        <Typography variant="body2" color="text.secondary" align="center">
          Please wait while we verify your email address.
        </Typography>
      </Stack>
    );
  }

  if (!verificationResult) {
    return (
      <Stack spacing={3}>
        <Alert severity="error">
          Something went wrong during verification.
        </Alert>
      </Stack>
    );
  }

  return (
    <Stack spacing={4}>
      <Typography variant="h4" align="center">
        Email Verification
      </Typography>

      <Alert severity={verificationResult.success ? 'success' : 'error'}>
        {verificationResult.message}
      </Alert>

      {verificationResult.success ? (
        <Button
          component={RouterLink}
          href={paths.auth.signIn}
          variant="contained"
          size="large"
        >
          Go to Sign In
        </Button>
      ) : (
        <Stack spacing={2}>
          <Button
            component={RouterLink}
            href={paths.auth.signUp}
            variant="outlined"
          >
            Back to Sign Up
          </Button>
          <Typography variant="body2" color="text.secondary" align="center">
            If you continue to have issues, please contact support.
          </Typography>
        </Stack>
      )}
    </Stack>
  );
}

// This ensures Next.js doesn't try to statically generate this dynamic page
export const dynamic = 'force-dynamic';

export default function VerifyPage(): React.JSX.Element {
  return (
    <Stack
      spacing={4}
      sx={{
        mx: 'auto',
        px: 3,
        py: 8,
        width: '100%',
        maxWidth: 400,
      }}
    >
      <React.Suspense fallback={<CircularProgress />}>
        <VerifyContent />
      </React.Suspense>
    </Stack>
  );
}
