'use client';

import * as React from 'react';
import { useSearchParams } from 'next/navigation';
import { zodResolver } from '@hookform/resolvers/zod';
import Alert from '@mui/material/Alert';
import Button from '@mui/material/Button';
import FormControl from '@mui/material/FormControl';
import FormHelperText from '@mui/material/FormHelperText';
import InputLabel from '@mui/material/InputLabel';
import OutlinedInput from '@mui/material/OutlinedInput';
import Stack from '@mui/material/Stack';
import Typography from '@mui/material/Typography';
import { Eye as EyeIcon } from '@phosphor-icons/react/dist/ssr/Eye';
import { EyeSlash as EyeSlashIcon } from '@phosphor-icons/react/dist/ssr/EyeSlash';
import { Controller, useForm } from 'react-hook-form';
import { z as zod } from 'zod';

import { authClient } from '@/lib/auth/client';

const forgotPasswordSchema = zod.object({
  email: zod.string().min(1, { message: 'Email is required' }).email(),
});

const resetPasswordSchema = zod
  .object({
    token: zod.string().min(1, { message: 'Reset token is required' }),
    newPassword: zod
      .string()
      .min(8, { message: 'Password must be at least 8 characters' })
      .regex(/^(?=.*[a-z])(?=.*[A-Z])(?=.*\d)(?=.*[^\w\s])[^\s]*$/, {
        message:
          'Password must contain at least one uppercase letter, one lowercase letter, one number, and one special character',
      }),
    confirmPassword: zod
      .string()
      .min(1, { message: 'Please confirm your password' }),
  })
  .refine((data) => data.newPassword === data.confirmPassword, {
    message: 'Passwords do not match',
    path: ['confirmPassword'],
  });

type ForgotPasswordValues = zod.infer<typeof forgotPasswordSchema>;
type ResetPasswordValues = zod.infer<typeof resetPasswordSchema>;

const forgotPasswordDefaults = { email: '' } satisfies ForgotPasswordValues;

export function ResetPasswordForm(): React.JSX.Element {
  const searchParams = useSearchParams();
  const token = searchParams.get('token');

  const [isPending, setIsPending] = React.useState<boolean>(false);
  const [isSubmitted, setIsSubmitted] = React.useState<boolean>(false);
  const [isResetComplete, setIsResetComplete] = React.useState<boolean>(false);
  const [showPassword, setShowPassword] = React.useState<boolean>(false);
  const [showConfirmPassword, setShowConfirmPassword] =
    React.useState<boolean>(false);

  const {
    control: forgotControl,
    handleSubmit: handleForgotSubmit,
    setError: setForgotError,
    formState: { errors: forgotErrors },
  } = useForm<ForgotPasswordValues>({
    defaultValues: forgotPasswordDefaults,
    resolver: zodResolver(forgotPasswordSchema),
  });

  const {
    control: resetControl,
    handleSubmit: handleResetSubmit,
    setError: setResetError,
    formState: { errors: resetErrors },
  } = useForm<ResetPasswordValues>({
    defaultValues: { token: token || '', newPassword: '', confirmPassword: '' },
    resolver: zodResolver(resetPasswordSchema),
  });

  const onForgotSubmit = React.useCallback(
    async (values: ForgotPasswordValues): Promise<void> => {
      setIsPending(true);

      try {
        const { error } = await authClient.forgotPassword({
          email: values.email,
        });

        if (error) {
          setForgotError('root', { type: 'server', message: error });
          setIsPending(false);
          return;
        }

        setIsPending(false);
        setIsSubmitted(true);
      } catch (err) {
        console.error('Forgot password error:', err);
        setForgotError('root', {
          type: 'server',
          message: 'An unexpected error occurred. Please try again.',
        });
        setIsPending(false);
      }
    },
    [setForgotError],
  );

  const onResetSubmit = React.useCallback(
    async (values: ResetPasswordValues): Promise<void> => {
      setIsPending(true);

      try {
        const { error } = await authClient.resetPassword({
          token: values.token,
          newPassword: values.newPassword,
        });

        if (error) {
          setResetError('root', { type: 'server', message: error });
          setIsPending(false);
          return;
        }

        setIsPending(false);
        setIsResetComplete(true);
      } catch (err) {
        console.error('Reset password error:', err);
        setResetError('root', {
          type: 'server',
          message: 'An unexpected error occurred. Please try again.',
        });
        setIsPending(false);
      }
    },
    [setResetError],
  );

  if (isResetComplete) {
    return (
      <Stack spacing={4}>
        <Typography variant="h5">Password Reset Successful</Typography>
        <Alert severity="success">
          Your password has been successfully reset. You can now sign in with
          your new password.
        </Alert>
        <Button href="/auth/sign-in" variant="contained">
          Go to Sign In
        </Button>
      </Stack>
    );
  }

  if (token) {
    // Show reset password form if token is present
    return (
      <Stack spacing={4}>
        <Typography variant="h5">Reset Your Password</Typography>
        <form onSubmit={handleResetSubmit(onResetSubmit)}>
          <Stack spacing={3}>
            <Controller
              control={resetControl}
              name="token"
              render={({ field }) => (
                <FormControl error={Boolean(resetErrors.token)}>
                  <InputLabel>Reset Token</InputLabel>
                  <OutlinedInput {...field} label="Reset Token" disabled />
                  {resetErrors.token ? (
                    <FormHelperText>{resetErrors.token.message}</FormHelperText>
                  ) : null}
                </FormControl>
              )}
            />
            <Controller
              control={resetControl}
              name="newPassword"
              render={({ field }) => (
                <FormControl error={Boolean(resetErrors.newPassword)}>
                  <InputLabel>New Password</InputLabel>
                  <OutlinedInput
                    {...field}
                    endAdornment={
                      showPassword ? (
                        <EyeIcon
                          cursor="pointer"
                          fontSize="var(--icon-fontSize-md)"
                          onClick={(): void => {
                            setShowPassword(false);
                          }}
                        />
                      ) : (
                        <EyeSlashIcon
                          cursor="pointer"
                          fontSize="var(--icon-fontSize-md)"
                          onClick={(): void => {
                            setShowPassword(true);
                          }}
                        />
                      )
                    }
                    label="New Password"
                    type={showPassword ? 'text' : 'password'}
                  />
                  {resetErrors.newPassword ? (
                    <FormHelperText>
                      {resetErrors.newPassword.message}
                    </FormHelperText>
                  ) : null}
                </FormControl>
              )}
            />
            <Controller
              control={resetControl}
              name="confirmPassword"
              render={({ field }) => (
                <FormControl error={Boolean(resetErrors.confirmPassword)}>
                  <InputLabel>Confirm New Password</InputLabel>
                  <OutlinedInput
                    {...field}
                    endAdornment={
                      showConfirmPassword ? (
                        <EyeIcon
                          cursor="pointer"
                          fontSize="var(--icon-fontSize-md)"
                          onClick={(): void => {
                            setShowConfirmPassword(false);
                          }}
                        />
                      ) : (
                        <EyeSlashIcon
                          cursor="pointer"
                          fontSize="var(--icon-fontSize-md)"
                          onClick={(): void => {
                            setShowConfirmPassword(true);
                          }}
                        />
                      )
                    }
                    label="Confirm New Password"
                    type={showConfirmPassword ? 'text' : 'password'}
                  />
                  {resetErrors.confirmPassword ? (
                    <FormHelperText>
                      {resetErrors.confirmPassword.message}
                    </FormHelperText>
                  ) : null}
                </FormControl>
              )}
            />
            {resetErrors.root ? (
              <Alert color="error">{resetErrors.root.message}</Alert>
            ) : null}
            <Button disabled={isPending} type="submit" variant="contained">
              {isPending ? 'Resetting Password...' : 'Reset Password'}
            </Button>
          </Stack>
        </form>
      </Stack>
    );
  }

  if (isSubmitted) {
    return (
      <Stack spacing={4}>
        <Typography variant="h5">Reset Password Email Sent</Typography>
        <Alert severity="success">
          If an account with that email address exists, we've sent a password
          reset link to your email. Please check your email and follow the
          instructions to reset your password.
        </Alert>
        <Typography variant="body2" color="text.secondary">
          Didn't receive the email? Check your spam folder or try again.
        </Typography>
        <Button variant="outlined" onClick={() => setIsSubmitted(false)}>
          Try Again
        </Button>
      </Stack>
    );
  }

  return (
    <Stack spacing={4}>
      <Typography variant="h5">Forgot Password</Typography>
      <Typography variant="body2" color="text.secondary">
        Enter your email address and we'll send you a link to reset your
        password.
      </Typography>
      <form onSubmit={handleForgotSubmit(onForgotSubmit)}>
        <Stack spacing={3}>
          <Controller
            control={forgotControl}
            name="email"
            render={({ field }) => (
              <FormControl error={Boolean(forgotErrors.email)}>
                <InputLabel>Email address</InputLabel>
                <OutlinedInput {...field} label="Email address" type="email" />
                {forgotErrors.email ? (
                  <FormHelperText>{forgotErrors.email.message}</FormHelperText>
                ) : null}
              </FormControl>
            )}
          />
          {forgotErrors.root ? (
            <Alert color="error">{forgotErrors.root.message}</Alert>
          ) : null}
          <Button disabled={isPending} type="submit" variant="contained">
            {isPending ? 'Sending...' : 'Send Reset Link'}
          </Button>
        </Stack>
      </form>
    </Stack>
  );
}
