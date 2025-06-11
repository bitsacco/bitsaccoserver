import { Controller, Get, UseGuards, Request } from '@nestjs/common';
import {
  ApiTags,
  ApiOperation,
  ApiResponse,
  ApiBearerAuth,
  ApiSecurity,
} from '@nestjs/swagger';
import { AuthenticatedRequest, UnifiedAuthGuard } from '@/common';

@ApiTags('manager')
@Controller()
@UseGuards(UnifiedAuthGuard)
@ApiBearerAuth()
@ApiSecurity('api-key')
export class ManagerController {
  @Get('profile')
  @ApiOperation({ summary: 'Get current user/API key profile (consolidated)' })
  @ApiResponse({
    status: 200,
    description: 'Profile retrieved successfully',
    schema: {
      type: 'object',
      properties: {
        userId: { type: 'string' },
        email: { type: 'string' },
        organizationId: { type: 'string' },
        authMethod: { type: 'string', enum: ['jwt', 'api-key'] },
        permissions: { type: 'array' },
        roles: { type: 'array' },
        apiKeyId: { type: 'string', nullable: true },
        membership: { type: 'object', nullable: true },
      },
    },
  })
  async getProfile(@Request() req: AuthenticatedRequest) {
    return {
      userId: req.user.sub || req.user.userId,
      email: req.user.email,
      organizationId: req.organizationId,
      authMethod: req.user.keyId ? 'api-key' : 'jwt',
      permissions: req.user.permissions,
      roles: req.user.roles,
      apiKeyId: req.apiKeyId || req.user.keyId,
      membership: req.organizationMembership,
    };
  }
}
