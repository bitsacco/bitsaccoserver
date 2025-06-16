import { Controller, Post, UseGuards, Request } from '@nestjs/common';
import {
  ApiTags,
  ApiOperation,
  ApiResponse,
  ApiSecurity,
} from '@nestjs/swagger';
import { AuthenticatedRequest, AuthGuard } from '../common';

@ApiTags('auth')
@Controller('auth')
export class ApiKeyController {
  constructor() {}

  @Post('validate')
  @UseGuards(AuthGuard)
  @ApiSecurity('api-key')
  @ApiOperation({ summary: 'Validate API key' })
  @ApiResponse({ status: 200, description: 'API key is valid' })
  @ApiResponse({ status: 401, description: 'Invalid API key' })
  async validateKey(@Request() req: AuthenticatedRequest) {
    // If we reach this point, the API key is valid (guard passed)
    return {
      valid: true,
      keyId: req.apiKeyId,
      organizationId: req.organizationId,
      permissions: req.member?.permissions || [],
      message: 'API key is valid',
    };
  }
}
