'use client';

import type { User } from '@/types/user';
import { ServiceRole } from '@bitsaccoserver/types';

// All API calls go through the Next.js API proxy
const API_URL = '/api';

// Types based on the server auth endpoints
interface AuthResponse {
  access_token: string;
  refresh_token: string;
  expires_in: number;
  token_type: string;
  member: ServerMember;
}

interface TokensResponse {
  access_token: string;
  refresh_token: string;
  expires_in: number;
  token_type: string;
}

interface ServerMember {
  id: string;
  email: string;
  firstName: string;
  lastName: string;
  emailVerified: boolean;
  phoneNumber?: string;
  serviceRole?: ServiceRole;
}

// Convert ServerMember to our app's User type
function mapServerMemberToUser(member: ServerMember): User {
  return {
    id: member.id,
    name: `${member.firstName} ${member.lastName}`.trim(),
    firstName: member.firstName,
    lastName: member.lastName,
    avatar: undefined, // No avatar in current server schema
    email: member.email,
    phone: member.phoneNumber,
    serviceRole: member.serviceRole,
  };
}

// Auth params
export interface SignUpParams {
  email: string;
  password: string;
  firstName: string;
  lastName: string;
  phoneNumber?: string;
}

export interface SignInParams {
  email: string;
  password: string;
}

export interface VerifyParams {
  token: string;
}

export interface RecoverParams {
  email: string;
}

export interface ResetPasswordParams {
  token: string;
  newPassword: string;
}

export interface ChangePasswordParams {
  currentPassword: string;
  newPassword: string;
}

class AuthClient {
  private commonHeaders = {
    'Content-Type': 'application/json',
    Accept: 'application/json',
  };

  private async fetchWithAuth(path: string, options: RequestInit = {}) {
    const token = localStorage.getItem('access-token');

    const headers = {
      ...this.commonHeaders,
      ...options.headers,
    };

    if (token) {
      (headers as unknown as any)['Authorization'] = `Bearer ${token}`;
    }

    const apiUrl = `${API_URL}${path}`;

    console.log(`Making ${options.method || 'GET'} request to: ${apiUrl}`);

    try {
      const response = await fetch(apiUrl, {
        ...options,
        headers,
        credentials: 'include', // Send cookies for cross-origin requests
      });

      if (!response.ok) {
        console.error(
          `Request failed: ${response.status} ${response.statusText}`,
          `URL: ${apiUrl}`,
          `Method: ${options.method || 'GET'}`,
        );
      }

      return response;
    } catch (error) {
      console.error(`Network error when fetching ${apiUrl}:`, error);
      throw error;
    }
  }

  private storeTokens(accessToken: string, refreshToken: string) {
    // Store tokens in localStorage for persistence
    localStorage.setItem('access-token', accessToken);
    localStorage.setItem('refresh-token', refreshToken);

    console.log('Authentication tokens stored successfully');
  }

  async signUp(params: SignUpParams): Promise<{ data?: User; error?: string }> {
    try {
      const response = await this.fetchWithAuth('/auth/register', {
        method: 'POST',
        body: JSON.stringify(params),
      });

      if (!response.ok) {
        try {
          const error = await response.json();
          return { error: error.message || 'Failed to sign up' };
        } catch (e) {
          return {
            error: `Failed to sign up: ${response.status} ${response.statusText}`,
          };
        }
      }

      const data = await response.json();
      return { data: null }; // Registration doesn't return user data, just success message
    } catch (error) {
      console.error('Sign up error:', error);
      return {
        error: 'Network error. Please check your connection and try again.',
      };
    }
  }

  async signIn(params: SignInParams): Promise<{ data?: User; error?: string }> {
    try {
      const response = await this.fetchWithAuth('/auth/login', {
        method: 'POST',
        body: JSON.stringify(params),
      });

      if (!response.ok) {
        try {
          const error = await response.json();
          return { error: error.message || 'Failed to sign in' };
        } catch (e) {
          return {
            error: `Failed to sign in: ${response.status} ${response.statusText}`,
          };
        }
      }

      const data: AuthResponse = await response.json();

      console.log('Login response:', data);

      if (data.access_token && data.refresh_token) {
        this.storeTokens(data.access_token, data.refresh_token);
      }

      return { data: mapServerMemberToUser(data.member) };
    } catch (error) {
      console.error('Sign in error:', error);
      return {
        error: 'Network error. Please check your connection and try again.',
      };
    }
  }

  async verifyEmail(params: VerifyParams): Promise<{ error?: string }> {
    try {
      const response = await this.fetchWithAuth(
        `/auth/verify-email?token=${params.token}`,
        {
          method: 'GET',
        },
      );

      if (!response.ok) {
        try {
          const error = await response.json();
          return { error: error.message || 'Failed to verify email' };
        } catch (e) {
          return {
            error: `Failed to verify email: ${response.status} ${response.statusText}`,
          };
        }
      }

      return {};
    } catch (error) {
      console.error('Email verification error:', error);
      return {
        error: 'Network error. Please check your connection and try again.',
      };
    }
  }

  async forgotPassword(params: RecoverParams): Promise<{ error?: string }> {
    try {
      const response = await this.fetchWithAuth('/auth/forgot-password', {
        method: 'POST',
        body: JSON.stringify(params),
      });

      if (!response.ok) {
        try {
          const error = await response.json();
          return {
            error: error.message || 'Failed to send password reset email',
          };
        } catch (e) {
          return {
            error: `Failed to send password reset email: ${response.status} ${response.statusText}`,
          };
        }
      }

      return {};
    } catch (error) {
      console.error('Forgot password error:', error);
      return {
        error: 'Network error. Please check your connection and try again.',
      };
    }
  }

  async resetPassword(
    params: ResetPasswordParams,
  ): Promise<{ error?: string }> {
    try {
      const response = await this.fetchWithAuth('/auth/reset-password', {
        method: 'POST',
        body: JSON.stringify(params),
      });

      if (!response.ok) {
        try {
          const error = await response.json();
          return { error: error.message || 'Failed to reset password' };
        } catch (e) {
          return {
            error: `Failed to reset password: ${response.status} ${response.statusText}`,
          };
        }
      }

      return {};
    } catch (error) {
      console.error('Reset password error:', error);
      return {
        error: 'Network error. Please check your connection and try again.',
      };
    }
  }

  async changePassword(
    params: ChangePasswordParams,
  ): Promise<{ error?: string }> {
    try {
      const response = await this.fetchWithAuth('/auth/change-password', {
        method: 'POST',
        body: JSON.stringify(params),
      });

      if (!response.ok) {
        try {
          const error = await response.json();
          return { error: error.message || 'Failed to change password' };
        } catch (e) {
          return {
            error: `Failed to change password: ${response.status} ${response.statusText}`,
          };
        }
      }

      return {};
    } catch (error) {
      console.error('Change password error:', error);
      return {
        error: 'Network error. Please check your connection and try again.',
      };
    }
  }

  async resendVerification(email: string): Promise<{ error?: string }> {
    try {
      const response = await this.fetchWithAuth('/auth/resend-verification', {
        method: 'POST',
        body: JSON.stringify({ email }),
      });

      if (!response.ok) {
        try {
          const error = await response.json();
          return {
            error: error.message || 'Failed to resend verification email',
          };
        } catch (e) {
          return {
            error: `Failed to resend verification email: ${response.status} ${response.statusText}`,
          };
        }
      }

      return {};
    } catch (error) {
      console.error('Resend verification error:', error);
      return {
        error: 'Network error. Please check your connection and try again.',
      };
    }
  }

  async refreshToken(): Promise<{ error?: string }> {
    const refreshToken = localStorage.getItem('refresh-token');

    if (!refreshToken) {
      return { error: 'No refresh token available' };
    }

    try {
      console.log('Attempting to refresh token');
      const response = await this.fetchWithAuth('/auth/refresh', {
        method: 'POST',
        body: JSON.stringify({ refresh_token: refreshToken }),
      });

      if (!response.ok) {
        console.error(
          'Token refresh failed:',
          response.status,
          response.statusText,
        );
        try {
          const error = await response.json();
          return { error: error.message || 'Failed to refresh token' };
        } catch (e) {
          return {
            error: `Failed to refresh token: ${response.status} ${response.statusText}`,
          };
        }
      }

      const data: TokensResponse = await response.json();

      if (!data.access_token || !data.refresh_token) {
        console.error('Invalid token refresh response:', data);
        return { error: 'Invalid token refresh response' };
      }

      console.log('Tokens refreshed successfully');
      this.storeTokens(data.access_token, data.refresh_token);

      return {};
    } catch (error) {
      console.error('Refresh token error:', error);
      return {
        error: 'Network error. Please check your connection and try again.',
      };
    }
  }

  async getUser(): Promise<{ data?: User | null; error?: string }> {
    const token = localStorage.getItem('access-token');

    if (!token) {
      console.log('No access token found in localStorage');
      return { data: null };
    }

    console.log('Access token found, fetching user data');

    try {
      // Use the /auth/member-info endpoint to get current user data
      const response = await this.fetchWithAuth('/auth/member-info', {
        method: 'GET',
      });

      if (response.status === 401) {
        console.log('Token expired, attempting to refresh');
        // Try to refresh the token
        const refreshResult = await this.refreshToken();
        if (refreshResult.error) {
          console.error('Token refresh failed:', refreshResult.error);
          // If refresh fails, clear tokens and return null
          this.signOut();
          return { data: null };
        }

        console.log('Token refreshed, retrying user fetch');
        // Retry with new token
        const retryResponse = await this.fetchWithAuth('/auth/member-info', {
          method: 'GET',
        });

        if (!retryResponse.ok) {
          console.error(
            'Retry failed:',
            retryResponse.status,
            retryResponse.statusText,
          );
          return { data: null };
        }

        try {
          const retryData = await retryResponse.json();
          console.log('User data retrieved after token refresh');

          return { data: mapServerMemberToUser(retryData) };
        } catch (parseError) {
          console.error('Failed to parse user data:', parseError);
          return { data: null, error: 'Invalid user data format' };
        }
      }

      if (!response.ok) {
        console.error(
          'Failed to get user data:',
          response.status,
          response.statusText,
        );
        try {
          const errorData = await response.json();
          console.error('Error response:', errorData);
        } catch (e) {
          // Ignore parse errors on error responses
        }
        return { data: null };
      }

      try {
        const data = await response.json();
        console.log('Member info response:', data);

        if (!data.id) {
          console.error('No member data in response');
          return { data: null };
        }

        console.log('User data retrieved successfully');
        return { data: mapServerMemberToUser(data) };
      } catch (parseError) {
        console.error('Failed to parse user data:', parseError);
        return { data: null, error: 'Invalid user data format' };
      }
    } catch (error) {
      console.error('Get user error:', error);
      return {
        data: null,
        error: 'Network error. Please check your connection and try again.',
      };
    }
  }

  async signOut(): Promise<{ error?: string }> {
    const refreshToken = localStorage.getItem('refresh-token');

    if (refreshToken) {
      try {
        console.log('Revoking refresh token');
        // Use /auth/logout endpoint
        await this.fetchWithAuth('/auth/logout', {
          method: 'POST',
          body: JSON.stringify({ refresh_token: refreshToken }),
        });
      } catch (error) {
        console.error('Sign out error:', error);
      }
    }

    // Clear tokens from local storage
    localStorage.removeItem('access-token');
    localStorage.removeItem('refresh-token');

    console.log('User signed out, tokens removed');

    return {};
  }
}

export const authClient = new AuthClient();
