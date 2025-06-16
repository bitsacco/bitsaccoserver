'use client';

import * as React from 'react';
import RouterLink from 'next/link';
import { useRouter } from 'next/navigation';
import { zodResolver } from '@hookform/resolvers/zod';
import Alert from '@mui/material/Alert';
import Button from '@mui/material/Button';
import FormControl from '@mui/material/FormControl';
import FormHelperText from '@mui/material/FormHelperText';
import InputLabel from '@mui/material/InputLabel';
import Link from '@mui/material/Link';
import OutlinedInput from '@mui/material/OutlinedInput';
import Stack from '@mui/material/Stack';
import Typography from '@mui/material/Typography';
import { Eye as EyeIcon } from '@phosphor-icons/react/dist/ssr/Eye';
import { EyeSlash as EyeSlashIcon } from '@phosphor-icons/react/dist/ssr/EyeSlash';
import { Controller, useForm } from 'react-hook-form';
import { z as zod } from 'zod';

import { paths } from '@/paths';
import { authClient } from '@/lib/auth/client';
import { useUser } from '@/hooks/use-user';
import { ServiceRole } from '@bitsaccoserver/types';

const schema = zod.object({
  email: zod.string().min(1, { message: 'Email is required' }).email(),
  password: zod.string().min(1, { message: 'Password is required' }),
});

type Values = zod.infer<typeof schema>;

const defaultValues = {
  email: '',
  password: '',
} satisfies Values;

export function SignInForm(): React.JSX.Element {
  const router = useRouter();

  const { checkSession } = useUser();

  const [showPassword, setShowPassword] = React.useState<boolean>();
  const [isPending, setIsPending] = React.useState<boolean>(false);

  const {
    control,
    handleSubmit,
    setError,
    formState: { errors },
  } = useForm<Values>({ defaultValues, resolver: zodResolver(schema) });

  const onSubmit = React.useCallback(
    async (values: Values): Promise<void> => {
      setIsPending(true);

      try {
        console.log('Attempting login with email:', values.email);
        const { data, error } = await authClient.signIn({
          email: values.email,
          password: values.password,
        });

        if (error) {
          const errorMessage =
            error.includes('Invalid email or password') ||
            error.includes('Invalid credentials')
              ? 'Invalid email or password. Please check your credentials and try again.'
              : error;

          setError('root', { type: 'server', message: errorMessage });
          setIsPending(false);
          return;
        }

        console.log(
          'Login successful, checking service role:',
          data?.serviceRole,
        );

        // Check if the user has admin or system admin service role
        if (data?.serviceRole) {
          const hasAdminRole =
            data.serviceRole === ServiceRole.ADMIN ||
            data.serviceRole === ServiceRole.SYSTEM_ADMIN;

          if (!hasAdminRole) {
            console.error('User does not have admin service role');
            setError('root', {
              type: 'server',
              message:
                "You don't have permission to access this dashboard. Only administrators can log in.",
            });

            // Sign out the user since they don't have permission
            await authClient.signOut();
            setIsPending(false);
            return;
          }

          console.log(
            'User has admin service role, proceeding with authentication',
          );
        } else {
          console.error('User has no service role defined');
          setError('root', {
            type: 'server',
            message:
              'Unable to determine your access level. Please contact an administrator.',
          });
          await authClient.signOut();
          setIsPending(false);
          return;
        }

        // Refresh the auth state
        console.log('Refreshing auth state...');
        await checkSession?.();
        console.log('Auth state refreshed, redirecting to dashboard');

        // Explicitly redirect to dashboard after successful authentication
        router.push(paths.dashboard.overview);
      } catch (err) {
        console.error('Login error:', err);
        setError('root', {
          type: 'server',
          message: 'An unexpected error occurred. Please try again.',
        });
        setIsPending(false);
      }
    },
    [checkSession, router, setError],
  );

  return (
    <Stack spacing={4}>
      <Stack spacing={1}>
        <Typography variant="h4">Sign in</Typography>
        <Typography color="text.secondary" variant="body2">
          Don&apos;t have an account?{' '}
          <Link
            component={RouterLink}
            href={paths.auth.signUp}
            underline="hover"
            variant="subtitle2"
          >
            Sign up
          </Link>
        </Typography>
      </Stack>
      <form onSubmit={handleSubmit(onSubmit)}>
        <Stack spacing={3}>
          <Controller
            control={control}
            name="email"
            render={({ field }) => (
              <FormControl error={Boolean(errors.email)}>
                <InputLabel>Email address</InputLabel>
                <OutlinedInput {...field} label="Email address" type="email" />
                {errors.email ? (
                  <FormHelperText>{errors.email.message}</FormHelperText>
                ) : null}
              </FormControl>
            )}
          />
          <Controller
            control={control}
            name="password"
            render={({ field }) => (
              <FormControl error={Boolean(errors.password)}>
                <InputLabel>Password</InputLabel>
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
                  label="Password"
                  type={showPassword ? 'text' : 'password'}
                />
                {errors.password ? (
                  <FormHelperText>{errors.password.message}</FormHelperText>
                ) : null}
              </FormControl>
            )}
          />
          <div>
            <Link
              component={RouterLink}
              href={paths.auth.resetPassword}
              variant="subtitle2"
            >
              Forgot password?
            </Link>
          </div>
          {errors.root ? (
            <Alert color="error">{errors.root.message}</Alert>
          ) : null}
          <Button disabled={isPending} type="submit" variant="contained">
            Sign in
          </Button>
        </Stack>
      </form>
    </Stack>
  );
}
