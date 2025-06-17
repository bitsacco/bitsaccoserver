export const paths = {
  home: '/',
  auth: {
    signIn: '/auth/sign-in',
    signUp: '/auth/sign-up',
    resetPassword: '/auth/reset-password',
    verify: '/auth/verify',
  },
  dashboard: {
    overview: '/dashboard',
    adminManagement: '/dashboard/admin-management',
    memberManagement: '/dashboard/member-management',
    settings: '/dashboard/settings',
  },
  errors: { notFound: '/errors/not-found' },
} as const;
