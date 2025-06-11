import {
  Controller,
  Post,
  Body,
  HttpException,
  HttpStatus,
  Logger,
  Get,
  Query,
  Res,
  Req,
  Param,
  UseGuards,
} from '@nestjs/common';
import {
  ApiTags,
  ApiOperation,
  ApiResponse,
  ApiBody,
  ApiQuery,
  ApiSecurity,
} from '@nestjs/swagger';
import { Response } from 'express';
import { AuthService } from './auth.service';
import {
  LoginDto,
  RegisterDto,
  RefreshTokenDto,
  LogoutDto,
  ResetPasswordDto,
  ChangePasswordDto,
} from './auth.dto';
import { AuthenticatedRequest, UnifiedAuthGuard } from '@/common';

@ApiTags('auth')
@Controller('auth')
export class AuthController {
  private readonly logger = new Logger(AuthController.name);

  constructor(private readonly authService: AuthService) {}

  @Post('register')
  @ApiOperation({
    summary: 'Register a new user',
    description:
      'Creates a new user account in Keycloak and optionally creates an organization',
  })
  @ApiResponse({
    status: 201,
    description: 'User registered successfully',
    schema: {
      type: 'object',
      properties: {
        message: { type: 'string' },
        userId: { type: 'string' },
        organizationId: { type: 'string', nullable: true },
      },
    },
  })
  @ApiResponse({ status: 400, description: 'Invalid registration data' })
  @ApiResponse({ status: 409, description: 'User already exists' })
  @ApiBody({ type: RegisterDto })
  async register(@Body() registerDto: RegisterDto) {
    try {
      this.logger.log(`Registration attempt for email: ${registerDto.email}`);

      const result = await this.authService.register(registerDto);

      this.logger.log(`User registered successfully: ${result.userId}`);
      return result;
    } catch (error) {
      this.logger.error(`Registration failed: ${error.message}`, error.stack);

      if (error.response?.status === 409) {
        throw new HttpException(
          'User with this email already exists',
          HttpStatus.CONFLICT,
        );
      }

      throw new HttpException(
        error.message || 'Registration failed',
        error.status || HttpStatus.BAD_REQUEST,
      );
    }
  }

  @Post('login')
  @ApiOperation({
    summary: 'Login user',
    description:
      'Authenticates user credentials with Keycloak and returns access tokens.\n\n' +
      '**To use the token in Swagger:**\n' +
      '1. Copy the `access_token` from the response\n' +
      "2. Click the 'Authorize' button at the top of this page\n" +
      "3. Paste the token in the 'Bearer Token' field\n" +
      "4. Click 'Authorize' - all endpoints will now use this token automatically\n\n" +
      '**Note:** The token will persist in your browser session and be used for all authenticated endpoints.',
  })
  @ApiResponse({
    status: 200,
    description:
      'Login successful - Copy the access_token to authorize other endpoints',
    schema: {
      type: 'object',
      properties: {
        access_token: {
          type: 'string',
          description:
            'JWT token to use for authentication (copy this to the Authorize button)',
          example: 'eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...',
        },
        refresh_token: {
          type: 'string',
          description: 'Token for refreshing the access token',
        },
        expires_in: {
          type: 'number',
          description: 'Token expiration time in seconds',
          example: 300,
        },
        token_type: {
          type: 'string',
          description: 'Type of token',
          example: 'bearer',
        },
        user: {
          type: 'object',
          properties: {
            id: { type: 'string' },
            email: { type: 'string' },
            firstName: { type: 'string' },
            lastName: { type: 'string' },
            emailVerified: { type: 'boolean' },
          },
        },
        organizations: {
          type: 'array',
          description: 'Organizations the user belongs to',
        },
      },
    },
  })
  @ApiResponse({ status: 401, description: 'Invalid credentials' })
  @ApiBody({ type: LoginDto })
  async login(@Body() loginDto: LoginDto) {
    try {
      this.logger.log(`Login attempt for email: ${loginDto.email}`);

      const result = await this.authService.login(loginDto);

      this.logger.log(`Login successful for user: ${result.user.id}`);
      return result;
    } catch (error) {
      this.logger.error(`Login failed: ${error.message}`, error.stack);

      throw new HttpException(
        'Invalid email or password',
        HttpStatus.UNAUTHORIZED,
      );
    }
  }

  @Post('refresh')
  @ApiOperation({
    summary: 'Refresh access token',
    description: 'Exchanges a refresh token for a new access token',
  })
  @ApiResponse({
    status: 200,
    description: 'Token refreshed successfully',
    schema: {
      type: 'object',
      properties: {
        access_token: { type: 'string' },
        refresh_token: { type: 'string' },
        expires_in: { type: 'number' },
        token_type: { type: 'string' },
      },
    },
  })
  @ApiResponse({ status: 401, description: 'Invalid refresh token' })
  @ApiBody({ type: RefreshTokenDto })
  async refreshToken(@Body() refreshTokenDto: RefreshTokenDto) {
    try {
      const result = await this.authService.refreshToken(
        refreshTokenDto.refresh_token,
      );
      return result;
    } catch (error) {
      this.logger.error(`Token refresh failed: ${error.message}`, error.stack);

      throw new HttpException('Invalid refresh token', HttpStatus.UNAUTHORIZED);
    }
  }

  @Post('logout')
  @ApiOperation({
    summary: 'Logout user',
    description: 'Invalidates the user session and refresh token in Keycloak',
  })
  @ApiResponse({ status: 200, description: 'Logout successful' })
  @ApiBody({ type: LogoutDto })
  async logout(@Body() logoutDto: LogoutDto) {
    try {
      await this.authService.logout(logoutDto.refresh_token);
      return { message: 'Logout successful' };
    } catch (error) {
      this.logger.error(`Logout failed: ${error.message}`, error.stack);

      // Even if logout fails, return success to client
      return { message: 'Logout completed' };
    }
  }

  @Post('forgot-password')
  @ApiOperation({
    summary: 'Request password reset',
    description: 'Sends a password reset email to the user via Keycloak',
  })
  @ApiResponse({ status: 200, description: 'Password reset email sent' })
  @ApiResponse({ status: 404, description: 'User not found' })
  @ApiBody({
    schema: {
      type: 'object',
      properties: {
        email: { type: 'string', format: 'email' },
      },
      required: ['email'],
    },
  })
  async forgotPassword(@Body() body: { email: string }) {
    try {
      await this.authService.requestPasswordReset(body.email);
      return { message: 'Password reset email sent if user exists' };
    } catch (error) {
      this.logger.error(
        `Password reset request failed: ${error.message}`,
        error.stack,
      );

      // Always return success for security (don't reveal if user exists)
      return { message: 'Password reset email sent if user exists' };
    }
  }

  @Post('reset-password')
  @ApiOperation({
    summary: 'Reset password',
    description: 'Resets user password using a reset token from email',
  })
  @ApiResponse({ status: 200, description: 'Password reset successful' })
  @ApiResponse({ status: 400, description: 'Invalid or expired reset token' })
  @ApiBody({ type: ResetPasswordDto })
  async resetPassword(@Body() resetPasswordDto: ResetPasswordDto) {
    try {
      await this.authService.resetPassword(
        resetPasswordDto.token,
        resetPasswordDto.newPassword,
      );
      return { message: 'Password reset successful' };
    } catch (error) {
      this.logger.error(`Password reset failed: ${error.message}`, error.stack);

      throw new HttpException(
        'Invalid or expired reset token',
        HttpStatus.BAD_REQUEST,
      );
    }
  }

  @Post('change-password')
  @ApiOperation({
    summary: 'Change password',
    description: 'Changes user password when authenticated',
  })
  @ApiResponse({ status: 200, description: 'Password changed successfully' })
  @ApiResponse({ status: 400, description: 'Invalid current password' })
  @ApiBody({ type: ChangePasswordDto })
  async changePassword(
    @Body() changePasswordDto: ChangePasswordDto,
    @Req() req: AuthenticatedRequest,
  ) {
    try {
      // Extract user ID from JWT token in request
      const userId = req.user?.sub;
      if (!userId) {
        throw new HttpException(
          'Authentication required',
          HttpStatus.UNAUTHORIZED,
        );
      }

      await this.authService.changePassword(
        userId,
        changePasswordDto.currentPassword,
        changePasswordDto.newPassword,
      );

      return { message: 'Password changed successfully' };
    } catch (error) {
      this.logger.error(
        `Password change failed: ${error.message}`,
        error.stack,
      );

      throw new HttpException(
        error.message || 'Password change failed',
        error.status || HttpStatus.BAD_REQUEST,
      );
    }
  }

  @Get('verify-email')
  @ApiOperation({
    summary: 'Verify email address',
    description: 'Verifies user email using token from verification email',
  })
  @ApiResponse({ status: 200, description: 'Email verified successfully' })
  @ApiResponse({
    status: 400,
    description: 'Invalid or expired verification token',
  })
  @ApiQuery({ name: 'token', description: 'Email verification token' })
  async verifyEmail(@Query('token') token: string, @Res() res: Response) {
    try {
      await this.authService.verifyEmail(token);

      // Redirect to success page or return success response
      res.json({ message: 'Email verified successfully' });
    } catch (error) {
      this.logger.error(
        `Email verification failed: ${error.message}`,
        error.stack,
      );

      throw new HttpException(
        'Invalid or expired verification token',
        HttpStatus.BAD_REQUEST,
      );
    }
  }

  @Post('resend-verification')
  @ApiOperation({
    summary: 'Resend email verification',
    description: 'Resends email verification to user',
  })
  @ApiResponse({ status: 200, description: 'Verification email sent' })
  @ApiBody({
    schema: {
      type: 'object',
      properties: {
        email: { type: 'string', format: 'email' },
      },
      required: ['email'],
    },
  })
  async resendVerification(@Body() body: { email: string }) {
    try {
      await this.authService.resendEmailVerification(body.email);
      return { message: 'Verification email sent' };
    } catch (error) {
      this.logger.error(
        `Resend verification failed: ${error.message}`,
        error.stack,
      );

      // Always return success for security
      return { message: 'Verification email sent if user exists' };
    }
  }

  @Get('user-info')
  @UseGuards(UnifiedAuthGuard)
  @ApiSecurity('bearer')
  @ApiOperation({
    summary: 'Get user information',
    description: 'Gets current user information from Keycloak',
  })
  @ApiResponse({
    status: 200,
    description: 'User information retrieved',
    schema: {
      type: 'object',
      properties: {
        id: { type: 'string' },
        email: { type: 'string' },
        firstName: { type: 'string' },
        lastName: { type: 'string' },
        emailVerified: { type: 'boolean' },
        organizations: { type: 'array' },
      },
    },
  })
  async getUserInfo(@Req() req: AuthenticatedRequest) {
    try {
      const userId = req.user?.sub;
      if (!userId) {
        throw new HttpException(
          'Authentication required',
          HttpStatus.UNAUTHORIZED,
        );
      }

      const userInfo = await this.authService.getUserInfo(userId);
      return userInfo;
    } catch (error) {
      this.logger.error(`Get user info failed: ${error.message}`, error.stack);

      throw new HttpException(
        'Failed to retrieve user information',
        HttpStatus.INTERNAL_SERVER_ERROR,
      );
    }
  }

  @Post('dev/verify-user/:email')
  @ApiOperation({
    summary: 'Manually verify user email (Development Only)',
    description:
      "Directly marks a user's email as verified for development purposes. Only works in development environment.",
  })
  @ApiResponse({
    status: 200,
    description: 'User email verified successfully',
    schema: {
      type: 'object',
      properties: {
        message: { type: 'string' },
        verified: { type: 'boolean' },
      },
    },
  })
  @ApiResponse({ status: 404, description: 'User not found' })
  @ApiResponse({ status: 403, description: 'Not available in production' })
  async verifyUserManually(@Param('email') email: string) {
    try {
      await this.authService.markEmailAsVerifiedForDev(email);
      return {
        message: 'User email has been marked as verified',
        verified: true,
      };
    } catch (error) {
      this.logger.error(
        `Manual verification failed: ${error.message}`,
        error.stack,
      );

      if (error.status === HttpStatus.FORBIDDEN) {
        throw error;
      }

      throw new HttpException(
        'User not found or verification failed',
        HttpStatus.NOT_FOUND,
      );
    }
  }

  @Get('dev/user-status/:email')
  @ApiOperation({
    summary: 'Get user status for debugging (Development Only)',
    description:
      'Returns detailed user information for debugging login issues. Only works in development environment.',
  })
  @ApiResponse({
    status: 200,
    description: 'User status retrieved',
    schema: {
      type: 'object',
      properties: {
        id: { type: 'string' },
        username: { type: 'string' },
        email: { type: 'string' },
        enabled: { type: 'boolean' },
        emailVerified: { type: 'boolean' },
        firstName: { type: 'string' },
        lastName: { type: 'string' },
        createdTimestamp: { type: 'number' },
        requiredActions: { type: 'array' },
      },
    },
  })
  @ApiResponse({ status: 404, description: 'User not found' })
  @ApiResponse({ status: 403, description: 'Not available in production' })
  async getUserStatus(@Param('email') email: string) {
    try {
      const userStatus = await this.authService.getUserStatusForDev(email);
      return userStatus;
    } catch (error) {
      this.logger.error(
        `Get user status failed: ${error.message}`,
        error.stack,
      );

      if (error.status === HttpStatus.FORBIDDEN) {
        throw error;
      }

      throw new HttpException('User not found', HttpStatus.NOT_FOUND);
    }
  }
}
