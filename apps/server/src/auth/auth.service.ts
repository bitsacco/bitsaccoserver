import { AxiosError } from 'axios';
import { firstValueFrom } from 'rxjs';
import { JwtService } from '@nestjs/jwt';
import { HttpService } from '@nestjs/axios';
import { ConfigService } from '@nestjs/config';
import { Injectable, HttpException, HttpStatus, Logger } from '@nestjs/common';
import { LoginDto, RegisterDto } from './auth.dto';

export interface KeycloakTokenResponse {
  access_token: string;
  refresh_token: string;
  expires_in: number;
  token_type: string;
  scope?: string;
}

export interface KeycloakUserInfo {
  sub: string;
  email: string;
  email_verified: boolean;
  given_name: string;
  family_name: string;
  preferred_username: string;
  name: string;
}

@Injectable()
export class AuthService {
  private readonly logger = new Logger(AuthService.name);
  private readonly keycloakBaseUrl: string;
  private readonly realm: string;
  private readonly clientId: string;
  private readonly clientSecret: string;

  constructor(
    private jwtService: JwtService,
    private readonly httpService: HttpService,
    private readonly configService: ConfigService,
  ) {
    this.keycloakBaseUrl = this.configService.get<string>(
      'KEYCLOAK_AUTH_SERVER_URL',
      '',
    );
    this.realm = this.configService.get<string>('KEYCLOAK_REALM', '');
    this.clientId = this.configService.get<string>('KEYCLOAK_CLIENT_ID', '');
    this.clientSecret = this.configService.get<string>(
      'KEYCLOAK_CLIENT_SECRET',
      '',
    );

    // Log warning if Keycloak is not configured, but don't throw error
    if (
      !this.keycloakBaseUrl ||
      !this.realm ||
      !this.clientId ||
      !this.clientSecret
    ) {
      this.logger.warn(
        'Keycloak configuration is incomplete - authentication endpoints will return appropriate error responses',
      );
    }
  }

  // Helper method to check if Keycloak is properly configured
  private isKeycloakConfigured(): boolean {
    return !!(
      this.keycloakBaseUrl &&
      this.realm &&
      this.clientId &&
      this.clientSecret
    );
  }

  // Helper method to ensure Keycloak is configured before making API calls
  private ensureKeycloakConfigured(methodName: string): void {
    if (!this.isKeycloakConfigured()) {
      this.logger.error(`${methodName}: Keycloak is not configured`);
      throw new HttpException(
        'Authentication service is not available. Please contact support.',
        HttpStatus.SERVICE_UNAVAILABLE,
      );
    }
  }

  // Get authentication service status
  getServiceStatus(): { configured: boolean; provider: string | null } {
    return {
      configured: this.isKeycloakConfigured(),
      provider: this.isKeycloakConfigured() ? 'Keycloak' : null,
    };
  }

  // Legacy JWT methods for backward compatibility
  async validateUser(email: string, memberId: string): Promise<any> {
    // This would typically validate against Keycloak
    // For now, we'll return a basic member object
    return {
      memberId,
      email,
      roles: ['member'],
    };
  }

  async legacyLogin(member: {
    memberId: string;
    email: string;
    roles: string[];
  }) {
    const payload = {
      email: member.email,
      sub: member.memberId,
      roles: member.roles,
    };

    return {
      access_token: this.jwtService.sign(payload),
      member: {
        id: member.memberId,
        email: member.email,
        roles: member.roles,
      },
    };
  }

  // Main Keycloak-based authentication methods
  async register(registerDto: RegisterDto) {
    // Check if Keycloak is configured
    if (!this.isKeycloakConfigured()) {
      this.logger.error(
        'Registration failed: Keycloak authentication service is not configured',
      );
      throw new HttpException(
        'Authentication service is not available. Please contact support.',
        HttpStatus.SERVICE_UNAVAILABLE,
      );
    }

    try {
      // Get admin access token for member creation
      const adminToken = await this.getAdminAccessToken();

      // Create member in Keycloak
      const keycloakUser = {
        username: registerDto.email,
        email: registerDto.email,
        firstName: registerDto.firstName,
        lastName: registerDto.lastName,
        enabled: true,
        emailVerified: false,
        credentials: [
          {
            type: 'password',
            value: registerDto.password,
            temporary: false,
          },
        ],
        attributes: {
          phoneNumber: registerDto.phoneNumber ? [registerDto.phoneNumber] : [],
        },
      };

      const createUserResponse = await firstValueFrom(
        this.httpService.post(
          `${this.keycloakBaseUrl}/admin/realms/${this.realm}/users`,
          keycloakUser,
          {
            headers: {
              Authorization: `Bearer ${adminToken}`,
              'Content-Type': 'application/json',
            },
          },
        ),
      );

      // Extract member ID from Location header
      const locationHeader = createUserResponse.headers.location;
      const memberId = locationHeader?.split('/').pop();

      if (!memberId) {
        throw new HttpException(
          'Failed to retrieve member ID after creation',
          HttpStatus.INTERNAL_SERVER_ERROR,
        );
      }

      // Send email verification
      await this.sendEmailVerification(memberId, adminToken);

      return {
        message:
          'Member registered successfully. Please check your email for verification.',
        memberId,
      };
    } catch (error) {
      this.logger.error(`Registration failed: ${error.message}`, error.stack);

      if (error instanceof AxiosError) {
        if (error.response?.status === 409) {
          throw new HttpException(
            'Member with this email already exists',
            HttpStatus.CONFLICT,
          );
        }

        if (error.response?.status === 400) {
          const errorMessage =
            error.response.data?.errorMessage || 'Invalid member data';
          throw new HttpException(errorMessage, HttpStatus.BAD_REQUEST);
        }
      }

      throw new HttpException(
        'Registration failed',
        HttpStatus.INTERNAL_SERVER_ERROR,
      );
    }
  }

  async login(loginDto: LoginDto) {
    // Check if Keycloak is configured
    if (!this.isKeycloakConfigured()) {
      this.logger.error(
        'Login failed: Keycloak authentication service is not configured',
      );
      throw new HttpException(
        'Authentication service is not available. Please contact support.',
        HttpStatus.SERVICE_UNAVAILABLE,
      );
    }

    try {
      // Authenticate with Keycloak
      const tokenResponse = await this.authenticateWithKeycloak(
        loginDto.email,
        loginDto.password,
      );

      // Get member information from JWT token directly (bypass memberinfo endpoint issue)
      this.logger.debug(
        `Extracting member info from JWT token: ${tokenResponse.access_token.substring(0, 20)}...`,
      );
      const memberInfo = this.extractUserInfoFromToken(
        tokenResponse.access_token,
      );
      this.logger.debug(
        `Member info extracted: ${memberInfo.email}, serviceRole: ${memberInfo.serviceRole}`,
      );

      // Create a server-side JWT token that abstracts away Keycloak
      const serverToken = this.jwtService.sign({
        sub: memberInfo.sub,
        email: memberInfo.email,
        firstName: memberInfo.given_name,
        lastName: memberInfo.family_name,
        emailVerified: memberInfo.email_verified,
        serviceRole: memberInfo.serviceRole,
        authMethod: 'keycloak',
        iat: Math.floor(Date.now() / 1000),
      });

      return {
        access_token: serverToken, // Return server-generated JWT instead of Keycloak token
        refresh_token: tokenResponse.refresh_token, // Keep Keycloak refresh token for renewals
        expires_in: tokenResponse.expires_in,
        token_type: tokenResponse.token_type,
        member: {
          id: memberInfo.sub,
          email: memberInfo.email,
          firstName: memberInfo.given_name,
          lastName: memberInfo.family_name,
          emailVerified: memberInfo.email_verified,
          serviceRole: memberInfo.serviceRole,
        },
      };
    } catch (error) {
      this.logger.error(`Login failed: ${error.message}`, error.stack);

      if (error instanceof AxiosError) {
        if (error.response?.status === 401) {
          throw new HttpException(
            'Invalid email or password',
            HttpStatus.UNAUTHORIZED,
          );
        }
      }

      throw new HttpException('Login failed', HttpStatus.INTERNAL_SERVER_ERROR);
    }
  }

  async refreshToken(refreshToken: string): Promise<KeycloakTokenResponse> {
    // Check if Keycloak is configured
    if (!this.isKeycloakConfigured()) {
      this.logger.error(
        'Token refresh failed: Keycloak authentication service is not configured',
      );
      throw new HttpException(
        'Authentication service is not available. Please contact support.',
        HttpStatus.SERVICE_UNAVAILABLE,
      );
    }

    try {
      const tokenData = {
        grant_type: 'refresh_token',
        refresh_token: refreshToken,
        client_id: this.clientId,
        client_secret: this.clientSecret,
      };

      const response = await firstValueFrom(
        this.httpService.post(
          `${this.keycloakBaseUrl}/realms/${this.realm}/protocol/openid-connect/token`,
          new URLSearchParams(tokenData),
          {
            headers: {
              'Content-Type': 'application/x-www-form-urlencoded',
            },
          },
        ),
      );

      const tokenResponse = response.data;

      // Extract member info from the new access token
      const memberInfo = this.extractUserInfoFromToken(
        tokenResponse.access_token,
      );

      // Create a server-side JWT token that abstracts away Keycloak
      const serverToken = this.jwtService.sign({
        sub: memberInfo.sub,
        email: memberInfo.email,
        firstName: memberInfo.given_name,
        lastName: memberInfo.family_name,
        emailVerified: memberInfo.email_verified,
        serviceRole: memberInfo.serviceRole,
        authMethod: 'keycloak',
        iat: Math.floor(Date.now() / 1000),
      });

      return {
        ...tokenResponse,
        access_token: serverToken, // Return server-generated JWT instead of Keycloak token
      };
    } catch (error) {
      this.logger.error(`Token refresh failed: ${error.message}`, error.stack);
      throw new HttpException('Invalid refresh token', HttpStatus.UNAUTHORIZED);
    }
  }

  async logout(refreshToken: string): Promise<void> {
    // Check if Keycloak is configured - for logout, we just return success
    if (!this.isKeycloakConfigured()) {
      this.logger.warn(
        'Logout requested but Keycloak authentication service is not configured - returning success',
      );
      return; // Gracefully handle logout when service is unavailable
    }

    try {
      const logoutData = {
        refresh_token: refreshToken,
        client_id: this.clientId,
        client_secret: this.clientSecret,
      };

      await firstValueFrom(
        this.httpService.post(
          `${this.keycloakBaseUrl}/realms/${this.realm}/protocol/openid-connect/logout`,
          new URLSearchParams(logoutData),
          {
            headers: {
              'Content-Type': 'application/x-www-form-urlencoded',
            },
          },
        ),
      );
    } catch (error) {
      this.logger.error(`Logout failed: ${error.message}`, error.stack);
      // Don't throw error for logout failures
    }
  }

  async requestPasswordReset(email: string): Promise<void> {
    // Check if Keycloak is configured
    if (!this.isKeycloakConfigured()) {
      this.logger.warn(
        'Password reset requested but Keycloak authentication service is not configured - returning graceful response',
      );
      return; // Gracefully handle password reset when service is unavailable (don't reveal service status)
    }

    try {
      const adminToken = await this.getAdminAccessToken();

      // Find member by email
      const members = await firstValueFrom(
        this.httpService.get(
          `${this.keycloakBaseUrl}/admin/realms/${this.realm}/users?email=${encodeURIComponent(email)}`,
          {
            headers: {
              Authorization: `Bearer ${adminToken}`,
            },
          },
        ),
      );

      if (members.data.length === 0) {
        // Member not found - but don't reveal this for security
        return;
      }

      const memberId = members.data[0].id;

      // Send password reset email
      await firstValueFrom(
        this.httpService.put(
          `${this.keycloakBaseUrl}/admin/realms/${this.realm}/users/${memberId}/execute-actions-email`,
          ['UPDATE_PASSWORD'],
          {
            headers: {
              Authorization: `Bearer ${adminToken}`,
              'Content-Type': 'application/json',
            },
          },
        ),
      );
    } catch (error) {
      this.logger.error(
        `Password reset request failed: ${error.message}`,
        error.stack,
      );
      // Don't throw error for security
    }
  }

  async resetPassword(_token: string, _newPassword: string): Promise<void> {
    // This would typically involve validating the reset token
    // and updating the password in Keycloak
    // Implementation depends on your reset token strategy
    throw new HttpException(
      'Password reset via token not implemented. Use email reset flow.',
      HttpStatus.NOT_IMPLEMENTED,
    );
  }

  async changePassword(
    memberId: string,
    currentPassword: string,
    newPassword: string,
  ): Promise<void> {
    try {
      const adminToken = await this.getAdminAccessToken();

      // Set new password
      await firstValueFrom(
        this.httpService.put(
          `${this.keycloakBaseUrl}/admin/realms/${this.realm}/users/${memberId}/reset-password`,
          {
            type: 'password',
            value: newPassword,
            temporary: false,
          },
          {
            headers: {
              Authorization: `Bearer ${adminToken}`,
              'Content-Type': 'application/json',
            },
          },
        ),
      );
    } catch (error) {
      this.logger.error(
        `Password change failed: ${error.message}`,
        error.stack,
      );
      throw new HttpException(
        'Failed to change password',
        HttpStatus.INTERNAL_SERVER_ERROR,
      );
    }
  }

  async verifyEmail(_token: string): Promise<void> {
    // Implementation depends on your email verification strategy
    throw new HttpException(
      'Email verification via token not implemented',
      HttpStatus.NOT_IMPLEMENTED,
    );
  }

  async resendEmailVerification(email: string): Promise<void> {
    // Check if Keycloak is configured
    if (!this.isKeycloakConfigured()) {
      this.logger.warn(
        'Email verification resend requested but Keycloak authentication service is not configured - returning graceful response',
      );
      return; // Gracefully handle email verification when service is unavailable
    }

    try {
      const adminToken = await this.getAdminAccessToken();

      // Find member by email
      const members = await firstValueFrom(
        this.httpService.get(
          `${this.keycloakBaseUrl}/admin/realms/${this.realm}/users?email=${encodeURIComponent(email)}`,
          {
            headers: {
              Authorization: `Bearer ${adminToken}`,
            },
          },
        ),
      );

      if (members.data.length === 0) {
        return; // Don't reveal if member exists
      }

      const memberId = members.data[0].id;
      await this.sendEmailVerification(memberId, adminToken);
    } catch (error) {
      this.logger.error(
        `Resend verification failed: ${error.message}`,
        error.stack,
      );
      // Don't throw error for security
    }
  }

  async getUserInfo(memberId: string) {
    // Check if Keycloak is configured
    if (!this.isKeycloakConfigured()) {
      this.logger.error(
        'Get member info failed: Keycloak authentication service is not configured',
      );
      throw new HttpException(
        'Authentication service is not available. Please contact support.',
        HttpStatus.SERVICE_UNAVAILABLE,
      );
    }

    try {
      const adminToken = await this.getAdminAccessToken();

      const memberResponse = await firstValueFrom(
        this.httpService.get(
          `${this.keycloakBaseUrl}/admin/realms/${this.realm}/users/${memberId}`,
          {
            headers: {
              Authorization: `Bearer ${adminToken}`,
            },
          },
        ),
      );

      const member = memberResponse.data;

      return {
        id: member.id,
        email: member.email,
        firstName: member.firstName,
        lastName: member.lastName,
        emailVerified: member.emailVerified,
      };
    } catch (error) {
      this.logger.error(
        `Get member info failed: ${error.message}`,
        error.stack,
      );
      throw new HttpException(
        'Failed to retrieve member information',
        HttpStatus.INTERNAL_SERVER_ERROR,
      );
    }
  }

  // Private helper methods
  private async getAdminAccessToken(): Promise<string> {
    this.ensureKeycloakConfigured('getAdminAccessToken');

    try {
      const tokenData = {
        grant_type: 'client_credentials',
        client_id: this.clientId,
        client_secret: this.clientSecret,
      };

      const response = await firstValueFrom(
        this.httpService.post(
          `${this.keycloakBaseUrl}/realms/${this.realm}/protocol/openid-connect/token`,
          new URLSearchParams(tokenData),
          {
            headers: {
              'Content-Type': 'application/x-www-form-urlencoded',
            },
          },
        ),
      );

      return response.data.access_token;
    } catch (error) {
      this.logger.error(
        `Failed to get admin token: ${error.message}`,
        error.stack,
      );
      throw new HttpException(
        'Authentication service unavailable',
        HttpStatus.SERVICE_UNAVAILABLE,
      );
    }
  }

  private async authenticateWithKeycloak(
    email: string,
    password: string,
  ): Promise<KeycloakTokenResponse> {
    this.ensureKeycloakConfigured('authenticateWithKeycloak');

    const tokenData = {
      grant_type: 'password',
      username: email,
      password: password,
      client_id: this.clientId,
      client_secret: this.clientSecret,
    };

    this.logger.debug(`Authenticating member: ${email} with Keycloak`);
    this.logger.debug(
      `Keycloak URL: ${this.keycloakBaseUrl}/realms/${this.realm}/protocol/openid-connect/token`,
    );
    this.logger.debug(`Client ID: ${this.clientId}`);

    try {
      const response = await firstValueFrom(
        this.httpService.post(
          `${this.keycloakBaseUrl}/realms/${this.realm}/protocol/openid-connect/token`,
          new URLSearchParams(tokenData),
          {
            headers: {
              'Content-Type': 'application/x-www-form-urlencoded',
            },
          },
        ),
      );

      this.logger.debug(`Keycloak authentication successful for: ${email}`);
      return response.data;
    } catch (error) {
      this.logger.error(
        `Keycloak authentication failed for ${email}:`,
        error.response?.data || error.message,
      );
      throw error;
    }
  }

  private async getKeycloakUserInfo(
    accessToken: string,
  ): Promise<KeycloakUserInfo> {
    try {
      const response = await firstValueFrom(
        this.httpService.get(
          `${this.keycloakBaseUrl}/realms/${this.realm}/protocol/openid-connect/userinfo`,
          {
            headers: {
              Authorization: `Bearer ${accessToken}`,
            },
          },
        ),
      );

      return response.data;
    } catch (error) {
      this.logger.error(
        `Failed to get member info from Keycloak:`,
        error.response?.data || error.message,
      );
      throw error;
    }
  }

  // Public method that can be called from controller
  async extractMemberInfoFromToken(
    accessToken: string,
  ): Promise<KeycloakUserInfo & { serviceRole?: string }> {
    return this.extractUserInfoFromToken(accessToken);
  }

  private extractUserInfoFromToken(
    accessToken: string,
  ): KeycloakUserInfo & { serviceRole?: string } {
    try {
      // Decode JWT token (we only need the payload, so we can use simple base64 decode)
      const tokenParts = accessToken.split('.');
      if (tokenParts.length !== 3) {
        throw new Error('Invalid JWT token format');
      }

      // Decode the payload (middle part)
      const payload = JSON.parse(
        Buffer.from(tokenParts[1], 'base64url').toString('utf8'),
      );

      this.logger.debug(`JWT payload: ${JSON.stringify(payload, null, 2)}`);

      // Determine service role based on token claims
      let serviceRole = 'member'; // default role

      // If the token already has a serviceRole field (server-generated token), use it
      if (payload.serviceRole) {
        serviceRole = payload.serviceRole;
      } else {
        // For Keycloak tokens, check for realm roles
        if (payload.realm_access?.roles) {
          if (payload.realm_access.roles.includes('system_admin')) {
            serviceRole = 'system_admin';
          } else if (payload.realm_access.roles.includes('admin')) {
            serviceRole = 'admin';
          }
        }

        // Check for resource roles (client-specific)
        if (payload.resource_access?.[this.clientId]?.roles) {
          const clientRoles = payload.resource_access[this.clientId].roles;
          if (clientRoles.includes('system_admin')) {
            serviceRole = 'system_admin';
          } else if (clientRoles.includes('admin')) {
            serviceRole = 'admin';
          }
        }
      }

      // Log the determined service role
      this.logger.debug(
        `Determined service role for ${payload.email}: ${serviceRole}`,
      );

      // Map token claims to KeycloakUserInfo format
      // Handle both server-generated tokens and Keycloak tokens
      return {
        sub: payload.sub,
        email: payload.email,
        email_verified:
          payload.email_verified || payload.emailVerified || false,
        given_name: payload.given_name || payload.firstName,
        family_name: payload.family_name || payload.lastName,
        preferred_username: payload.preferred_username,
        name: payload.name,
        serviceRole,
      };
    } catch (error) {
      this.logger.error(
        `Failed to extract member info from token: ${error.message}`,
      );
      throw new HttpException('Invalid access token', HttpStatus.UNAUTHORIZED);
    }
  }

  private async sendEmailVerification(
    memberId: string,
    adminToken: string,
  ): Promise<void> {
    try {
      await firstValueFrom(
        this.httpService.put(
          `${this.keycloakBaseUrl}/admin/realms/${this.realm}/users/${memberId}/execute-actions-email`,
          ['VERIFY_EMAIL'],
          {
            headers: {
              Authorization: `Bearer ${adminToken}`,
              'Content-Type': 'application/json',
            },
          },
        ),
      );
    } catch (error) {
      this.logger.warn(`Failed to send verification email: ${error.message}`);
      // Don't fail registration if email sending fails
    }
  }

  async markEmailAsVerifiedForDev(email: string): Promise<void> {
    const nodeEnv = this.configService.get<string>('NODE_ENV');
    if (nodeEnv === 'production') {
      throw new HttpException(
        'Manual verification is not available in production',
        HttpStatus.FORBIDDEN,
      );
    }

    // Check if Keycloak is configured
    if (!this.isKeycloakConfigured()) {
      this.logger.error(
        'Dev email verification failed: Keycloak authentication service is not configured',
      );
      throw new HttpException(
        'Authentication service is not available. Please contact support.',
        HttpStatus.SERVICE_UNAVAILABLE,
      );
    }

    try {
      // Get admin token
      const adminToken = await this.getAdminAccessToken();

      // Find member by email
      const membersResponse = await firstValueFrom(
        this.httpService.get(
          `${this.keycloakBaseUrl}/admin/realms/${this.realm}/members?email=${encodeURIComponent(email)}`,
          {
            headers: {
              Authorization: `Bearer ${adminToken}`,
            },
          },
        ),
      );

      if (!membersResponse.data || membersResponse.data.length === 0) {
        throw new HttpException('Member not found', HttpStatus.NOT_FOUND);
      }

      const member = membersResponse.data[0];

      if (member.emailVerified) {
        this.logger.log(`DEV: Member ${email} is already verified`);
        return;
      }

      // Update member to mark email as verified
      await firstValueFrom(
        this.httpService.put(
          `${this.keycloakBaseUrl}/admin/realms/${this.realm}/users/${member.id}`,
          {
            ...member,
            emailVerified: true,
          },
          {
            headers: {
              Authorization: `Bearer ${adminToken}`,
              'Content-Type': 'application/json',
            },
          },
        ),
      );

      this.logger.log(`DEV: Successfully marked ${email} as verified`);
    } catch (error) {
      this.logger.error(
        `Failed to verify member email: ${error.message}`,
        error.stack,
      );

      if (error instanceof HttpException) {
        throw error;
      }

      throw new HttpException(
        'Failed to verify member email',
        HttpStatus.INTERNAL_SERVER_ERROR,
      );
    }
  }

  async getUserStatusForDev(email: string): Promise<any> {
    const nodeEnv = this.configService.get<string>('NODE_ENV');
    if (nodeEnv === 'production') {
      throw new HttpException(
        'Member status is not available in production',
        HttpStatus.FORBIDDEN,
      );
    }

    // Check if Keycloak is configured
    if (!this.isKeycloakConfigured()) {
      this.logger.error(
        'Dev get member status failed: Keycloak authentication service is not configured',
      );
      throw new HttpException(
        'Authentication service is not available. Please contact support.',
        HttpStatus.SERVICE_UNAVAILABLE,
      );
    }

    try {
      // Get admin token
      const adminToken = await this.getAdminAccessToken();

      // Find member by email
      const membersResponse = await firstValueFrom(
        this.httpService.get(
          `${this.keycloakBaseUrl}/admin/realms/${this.realm}/members?email=${encodeURIComponent(email)}`,
          {
            headers: {
              Authorization: `Bearer ${adminToken}`,
            },
          },
        ),
      );

      if (!membersResponse.data || membersResponse.data.length === 0) {
        throw new HttpException('Member not found', HttpStatus.NOT_FOUND);
      }

      const member = membersResponse.data[0];

      return {
        id: member.id,
        username: member.username,
        email: member.email,
        enabled: member.enabled,
        emailVerified: member.emailVerified,
        firstName: member.firstName,
        lastName: member.lastName,
        createdTimestamp: member.createdTimestamp,
        requiredActions: member.requiredActions || [],
        attributes: member.attributes || {},
        debug: {
          canLogin: member.enabled && member.emailVerified,
          issues: [
            ...(member.enabled ? [] : ['Member is disabled']),
            ...(member.emailVerified ? [] : ['Email not verified']),
            ...((member.requiredActions || []).length > 0
              ? [`Required actions: ${member.requiredActions.join(', ')}`]
              : []),
          ],
        },
      };
    } catch (error) {
      this.logger.error(
        `Failed to get member status: ${error.message}`,
        error.stack,
      );

      if (error instanceof HttpException) {
        throw error;
      }

      throw new HttpException(
        'Failed to get member status',
        HttpStatus.INTERNAL_SERVER_ERROR,
      );
    }
  }
}
