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
import { AuthenticatedRequest, AuthGuard } from '@/common';

@ApiTags('auth')
@Controller('auth')
export class AuthController {
  private readonly logger = new Logger(AuthController.name);

  constructor(private readonly authService: AuthService) {}

  @Post('register')
  @ApiOperation({
    summary: 'Register a new member',
    description:
      'Creates a new member account in Keycloak',
  })
  @ApiResponse({
    status: 201,
    description: 'Member registered successfully',
    schema: {
      type: 'object',
      properties: {
        message: { type: 'string' },
        memberId: { type: 'string' },
      },
    },
  })
  @ApiResponse({ status: 400, description: 'Invalid registration data' })
  @ApiResponse({ status: 409, description: 'Member already exists' })
  @ApiBody({ type: RegisterDto })
  async register(@Body() registerDto: RegisterDto) {
    try {
      this.logger.log(`Registration attempt for email: ${registerDto.email}`);

      const result = await this.authService.register(registerDto);

      this.logger.log(`Member registered successfully: ${result.memberId}`);
      return result;
    } catch (error) {
      this.logger.error(`Registration failed: ${error.message}`, error.stack);

      if (error.response?.status === 409) {
        throw new HttpException(
          'Member with this email already exists',
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
    summary: 'Login member',
    description:
      'Authenticates member credentials with Keycloak and returns access tokens.\n\n' +
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
        member: {
          type: 'object',
          properties: {
            id: { type: 'string' },
            email: { type: 'string' },
            firstName: { type: 'string' },
            lastName: { type: 'string' },
            emailVerified: { type: 'boolean' },
          },
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

      this.logger.log(`Login successful for member: ${result.member.id}`);
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
    summary: 'Logout member',
    description: 'Invalidates the member session and refresh token in Keycloak',
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
    description: 'Sends a password reset email to the member via Keycloak',
  })
  @ApiResponse({ status: 200, description: 'Password reset email sent' })
  @ApiResponse({ status: 404, description: 'Member not found' })
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
      return { message: 'Password reset email sent if member exists' };
    } catch (error) {
      this.logger.error(
        `Password reset request failed: ${error.message}`,
        error.stack,
      );

      // Always return success for security (don't reveal if member exists)
      return { message: 'Password reset email sent if member exists' };
    }
  }

  @Post('reset-password')
  @ApiOperation({
    summary: 'Reset password',
    description: 'Resets member password using a reset token from email',
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
    description: 'Changes member password when authenticated',
  })
  @ApiResponse({ status: 200, description: 'Password changed successfully' })
  @ApiResponse({ status: 400, description: 'Invalid current password' })
  @ApiBody({ type: ChangePasswordDto })
  async changePassword(
    @Body() changePasswordDto: ChangePasswordDto,
    @Req() req: AuthenticatedRequest,
  ) {
    try {
      // Extract member ID from JWT token in request
      const memberId = req.member?.sub;
      if (!memberId) {
        throw new HttpException(
          'Authentication required',
          HttpStatus.UNAUTHORIZED,
        );
      }

      await this.authService.changePassword(
        memberId,
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
    description: 'Verifies member email using token from verification email',
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
    description: 'Resends email verification to member',
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
      return { message: 'Verification email sent if member exists' };
    }
  }

  @Get('member-info')
  @UseGuards(AuthGuard)
  @ApiSecurity('bearer')
  @ApiOperation({
    summary: 'Get member information',
    description: 'Gets current member information from JWT token',
  })
  @ApiResponse({
    status: 200,
    description: 'Member information retrieved',
    schema: {
      type: 'object',
      properties: {
        id: { type: 'string' },
        email: { type: 'string' },
        firstName: { type: 'string' },
        lastName: { type: 'string' },
        emailVerified: { type: 'boolean' },
      },
    },
  })
  async getUserInfo(@Req() req: AuthenticatedRequest) {
    try {
      const member = req.member;
      if (!member) {
        throw new HttpException(
          'Authentication required',
          HttpStatus.UNAUTHORIZED,
        );
      }

      // Extract basic info from the authenticated member context
      // The member object should already contain the necessary information from the JWT token
      const authHeader = req.headers.authorization;
      if (!authHeader || !authHeader.startsWith('Bearer ')) {
        throw new HttpException(
          'Invalid authorization header',
          HttpStatus.UNAUTHORIZED,
        );
      }

      const token = authHeader.slice(7);

      // Decode JWT token directly to get the raw payload
      const tokenParts = token.split('.');
      if (tokenParts.length !== 3) {
        throw new HttpException(
          'Invalid JWT token format',
          HttpStatus.UNAUTHORIZED,
        );
      }

      const payload = JSON.parse(
        Buffer.from(tokenParts[1], 'base64url').toString('utf8'),
      );

      this.logger.debug(
        `Member-info JWT payload: ${JSON.stringify(payload, null, 2)}`,
      );

      // Handle both server-generated tokens and Keycloak tokens
      const firstName = payload.firstName || payload.given_name;
      const lastName = payload.lastName || payload.family_name;

      this.logger.debug(
        `Extracted firstName: ${firstName}, lastName: ${lastName}`,
      );

      return {
        id: payload.sub,
        email: payload.email,
        firstName,
        lastName,
        emailVerified: payload.email_verified || payload.emailVerified,
        serviceRole: payload.serviceRole,
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

  @Post('dev/verify-member/:email')
  @ApiOperation({
    summary: 'Manually verify member email (Development Only)',
    description:
      "Directly marks a member's email as verified for development purposes. Only works in development environment.",
  })
  @ApiResponse({
    status: 200,
    description: 'Member email verified successfully',
    schema: {
      type: 'object',
      properties: {
        message: { type: 'string' },
        verified: { type: 'boolean' },
      },
    },
  })
  @ApiResponse({ status: 404, description: 'Member not found' })
  @ApiResponse({ status: 403, description: 'Not available in production' })
  async verifyUserManually(@Param('email') email: string) {
    try {
      await this.authService.markEmailAsVerifiedForDev(email);
      return {
        message: 'Member email has been marked as verified',
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
        'Member not found or verification failed',
        HttpStatus.NOT_FOUND,
      );
    }
  }

  @Get('dev/member-status/:email')
  @ApiOperation({
    summary: 'Get member status for debugging (Development Only)',
    description:
      'Returns detailed member information for debugging login issues. Only works in development environment.',
  })
  @ApiResponse({
    status: 200,
    description: 'Member status retrieved',
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
  @ApiResponse({ status: 404, description: 'Member not found' })
  @ApiResponse({ status: 403, description: 'Not available in production' })
  async getMemberStatus(@Param('email') email: string) {
    try {
      const memberStatus = await this.authService.getMemberStatusForDev(email);
      return memberStatus;
    } catch (error) {
      this.logger.error(
        `Get member status failed: ${error.message}`,
        error.stack,
      );

      if (error.status === HttpStatus.FORBIDDEN) {
        throw error;
      }

      throw new HttpException('Member not found', HttpStatus.NOT_FOUND);
    }
  }
}
