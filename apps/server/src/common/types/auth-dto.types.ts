// Authentication DTOs and request types

export interface LoginDto {
  email: string;
  password: string;
  rememberMe?: boolean;
  organizationId?: string;
}

export interface RegisterDto {
  email: string;
  password: string;
  firstName: string;
  lastName: string;
  phoneNumber?: string;
  organizationId?: string;
  invitationCode?: string;
}

export interface RefreshTokenDto {
  refreshToken: string;
}

export interface LogoutDto {
  refreshToken?: string;
  logoutAllDevices?: boolean;
}

export interface ResetPasswordDto {
  token: string;
  newPassword: string;
  confirmPassword: string;
}

export interface ChangePasswordDto {
  currentPassword: string;
  newPassword: string;
  confirmPassword: string;
}

export interface ForgotPasswordDto {
  email: string;
  organizationId?: string;
}

export interface VerifyEmailDto {
  token: string;
  email: string;
}

export interface ResendVerificationDto {
  email: string;
  organizationId?: string;
}

// Note: CreateApiKeyDto and UpdateApiKeyDto moved to dto folder to avoid conflicts
