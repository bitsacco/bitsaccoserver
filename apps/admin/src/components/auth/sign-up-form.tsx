'use client';

import * as React from 'react';
import RouterLink from 'next/link';
import { useRouter } from 'next/navigation';
import { zodResolver } from '@hookform/resolvers/zod';
import Alert from '@mui/material/Alert';
import Button from '@mui/material/Button';
import Checkbox from '@mui/material/Checkbox';
import FormControl from '@mui/material/FormControl';
import FormControlLabel from '@mui/material/FormControlLabel';
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

const schema = zod
  .object({
    email: zod.string().min(1, { message: 'Email is required' }).email(),
    firstName: zod.string().min(1, { message: 'First name is required' }),
    lastName: zod.string().min(1, { message: 'Last name is required' }),
    phoneNumber: zod.string().optional(),
    password: zod
      .string()
      .min(8, { message: 'Password must be at least 8 characters' })
      .regex(/^(?=.*[a-z])(?=.*[A-Z])(?=.*\d)(?=.*[^\w\s])[^\s]*$/, {
        message:
          'Password must contain at least one uppercase letter, one lowercase letter, one number, and one special character',
      }),
    confirmPassword: zod
      .string()
      .min(1, { message: 'Please confirm your password' }),
    terms: zod
      .boolean()
      .refine((value) => value, 'You must accept the terms and conditions'),
  })
  .refine((data) => data.password === data.confirmPassword, {
    message: 'Passwords do not match',
    path: ['confirmPassword'],
  });

type Values = zod.infer<typeof schema>;

const defaultValues = {
  email: '',
  firstName: '',
  lastName: '',
  phoneNumber: '',
  password: '',
  confirmPassword: '',
  terms: false,
} satisfies Values;

export function SignUpForm(): React.JSX.Element {
  const router = useRouter();

  const [showPassword, setShowPassword] = React.useState<boolean>(false);
  const [showConfirmPassword, setShowConfirmPassword] =
    React.useState<boolean>(false);
  const [isPending, setIsPending] = React.useState<boolean>(false);
  const [registrationSuccess, setRegistrationSuccess] =
    React.useState<boolean>(false);

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
        const { error } = await authClient.signUp({
          email: values.email,
          firstName: values.firstName,
          lastName: values.lastName,
          password: values.password,
          phoneNumber: values.phoneNumber || undefined,
        });

        if (error) {
          setError('root', { type: 'server', message: error });
          setIsPending(false);
          return;
        }

        setRegistrationSuccess(true);
        setIsPending(false);
      } catch (err) {
        console.error('Registration error:', err);
        setError('root', {
          type: 'server',
          message: 'An unexpected error occurred. Please try again.',
        });
        setIsPending(false);
      }
    },
    [setError],
  );

  if (registrationSuccess) {
    return (
      <Stack spacing={3}>
        <Stack spacing={1}>
          <Typography variant="h4">Registration Successful!</Typography>
          <Typography color="text.secondary" variant="body2">
            Your account has been created successfully. Please check your email
            to verify your account before signing in.
          </Typography>
        </Stack>
        <Alert color="success">
          A verification email has been sent to your email address. Please click
          the link in the email to verify your account.
        </Alert>
        <Button
          component={RouterLink}
          href={paths.auth.signIn}
          variant="contained"
        >
          Go to Sign In
        </Button>
      </Stack>
    );
  }

  return (
    <Stack spacing={3}>
      <Stack spacing={1}>
        <Typography variant="h4">Sign up</Typography>
        <Typography color="text.secondary" variant="body2">
          Already have an account?{' '}
          <Link
            component={RouterLink}
            href={paths.auth.signIn}
            underline="hover"
            variant="subtitle2"
          >
            Sign in
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
          <Stack direction="row" spacing={2}>
            <Controller
              control={control}
              name="firstName"
              render={({ field }) => (
                <FormControl error={Boolean(errors.firstName)} fullWidth>
                  <InputLabel>First name</InputLabel>
                  <OutlinedInput {...field} label="First name" />
                  {errors.firstName ? (
                    <FormHelperText>{errors.firstName.message}</FormHelperText>
                  ) : null}
                </FormControl>
              )}
            />
            <Controller
              control={control}
              name="lastName"
              render={({ field }) => (
                <FormControl error={Boolean(errors.lastName)} fullWidth>
                  <InputLabel>Last name</InputLabel>
                  <OutlinedInput {...field} label="Last name" />
                  {errors.lastName ? (
                    <FormHelperText>{errors.lastName.message}</FormHelperText>
                  ) : null}
                </FormControl>
              )}
            />
          </Stack>
          <Controller
            control={control}
            name="phoneNumber"
            render={({ field }) => (
              <FormControl error={Boolean(errors.phoneNumber)}>
                <InputLabel>Phone number (optional)</InputLabel>
                <OutlinedInput {...field} label="Phone number (optional)" />
                {errors.phoneNumber ? (
                  <FormHelperText>{errors.phoneNumber.message}</FormHelperText>
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
          <Controller
            control={control}
            name="confirmPassword"
            render={({ field }) => (
              <FormControl error={Boolean(errors.confirmPassword)}>
                <InputLabel>Confirm password</InputLabel>
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
                  label="Confirm password"
                  type={showConfirmPassword ? 'text' : 'password'}
                />
                {errors.confirmPassword ? (
                  <FormHelperText>
                    {errors.confirmPassword.message}
                  </FormHelperText>
                ) : null}
              </FormControl>
            )}
          />
          <Controller
            control={control}
            name="terms"
            render={({ field }) => (
              <div>
                <FormControlLabel
                  control={<Checkbox {...field} />}
                  label={
                    <React.Fragment>
                      I have read the <Link>terms and conditions</Link>
                    </React.Fragment>
                  }
                />
                {errors.terms ? (
                  <FormHelperText error>{errors.terms.message}</FormHelperText>
                ) : null}
              </div>
            )}
          />
          {errors.root ? (
            <Alert color="error">{errors.root.message}</Alert>
          ) : null}
          <Button disabled={isPending} type="submit" variant="contained">
            {isPending ? 'Creating Account...' : 'Sign up'}
          </Button>
        </Stack>
      </form>
      <Alert color="info">
        After registration, you will receive an email verification link. You
        must verify your email before you can sign in.
      </Alert>
    </Stack>
  );
}
