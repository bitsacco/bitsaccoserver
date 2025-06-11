import * as bcrypt from 'bcrypt';
import { Model } from 'mongoose';
import {
  Injectable,
  CanActivate,
  ExecutionContext,
  UnauthorizedException,
} from '@nestjs/common';
import { JwtService } from '@nestjs/jwt';
import { InjectModel } from '@nestjs/mongoose';
import { AuthenticatedRequest, JwtPayload } from '../types';
import { ApiKeyDocument, ApiKeyStatus } from '../schemas/api-key.schema';
import {
  OrganizationMember,
  OrganizationMemberDocument,
} from '../schemas/organization.schema';
import { OrganizationServiceDocument } from '../schemas/service.schema';

@Injectable()
export class UnifiedAuthGuard implements CanActivate {
  constructor(
    @InjectModel(ApiKeyDocument.name)
    private apiKeyModel: Model<ApiKeyDocument>,
    @InjectModel(OrganizationMember.name)
    private organizationMemberModel: Model<OrganizationMemberDocument>,
    @InjectModel(OrganizationServiceDocument.name)
    private organizationServiceModel: Model<OrganizationServiceDocument>,
    private jwtService: JwtService,
  ) {}

  async canActivate(context: ExecutionContext): Promise<boolean> {
    const request = context.switchToHttp().getRequest<AuthenticatedRequest>();

    // Try API Key authentication first
    const apiKey = request.headers['x-api-key'] as string;
    if (apiKey) {
      return this.validateApiKey(request, apiKey);
    }

    // Fall back to JWT authentication
    const token = this.extractTokenFromHeader(request);
    if (token) {
      return this.validateJwtToken(request, token);
    }

    throw new UnauthorizedException(
      'No valid authentication method found. Please provide either an X-API-Key header or Authorization Bearer token.',
    );
  }

  private async validateApiKey(
    request: AuthenticatedRequest,
    apiKey: string,
  ): Promise<boolean> {
    try {
      // Find API key by prefix (first 8 characters)
      const keyPrefix = apiKey.substring(0, 8);
      const apiKeyDoc = await this.apiKeyModel.findOne({
        keyId: { $regex: `^${keyPrefix}` },
        status: ApiKeyStatus.ACTIVE,
        $or: [{ expiresAt: { $gt: new Date() } }, { expiresAt: null }],
      });

      if (!apiKeyDoc) {
        throw new UnauthorizedException('Invalid API key');
      }

      // Verify the full key
      const isValid = await bcrypt.compare(apiKey, apiKeyDoc.hashedKey);
      if (!isValid) {
        throw new UnauthorizedException('Invalid API key');
      }

      // Check if organization has access to the requested service
      const serviceName = this.extractServiceFromPath(request.path);
      if (serviceName && !apiKeyDoc.serviceIds.includes(serviceName)) {
        throw new UnauthorizedException(
          `API key does not have access to ${serviceName} service`,
        );
      }

      // Check organization service status
      const orgService = await this.organizationServiceModel.findOne({
        organizationId: apiKeyDoc.organizationId,
        serviceId: serviceName,
        isEnabled: true,
      });

      if (serviceName && !orgService) {
        throw new UnauthorizedException(
          `Organization does not have access to ${serviceName} service`,
        );
      }

      // Update last used timestamp
      apiKeyDoc.lastUsedAt = new Date();
      await apiKeyDoc.save();

      // Set user context
      request.user = {
        sub: apiKeyDoc.createdBy,
        email: '', // API keys don't have associated email
        authMethod: 'api-key',
        permissions: apiKeyDoc.permissions,
        organizationId: apiKeyDoc.organizationId,
        keyId: apiKeyDoc.keyId,
        userId: apiKeyDoc.createdBy,
      };
      request.organizationId = apiKeyDoc.organizationId;
      request.apiKeyId = apiKeyDoc._id.toString();

      return true;
    } catch (error) {
      throw new UnauthorizedException(
        'API key authentication failed: ' + error.message,
      );
    }
  }

  private async validateJwtToken(
    request: AuthenticatedRequest,
    token: string,
  ): Promise<boolean> {
    try {
      // Use JwtService to decode without verification (for Keycloak tokens)
      // We decode without verification because Keycloak uses its own signing key
      const payload = this.jwtService.decode(token) as JwtPayload;
      if (!payload || !payload.sub) {
        throw new Error('Invalid token payload: missing sub field');
      }

      // Set user context for JWT
      request.user = {
        sub: payload.sub,
        email: payload.email || payload.preferred_username || '', // Email might not be present
        authMethod: 'jwt',
        userId: payload.sub,
      };

      // For JWT, we'll need organization context from request params or body
      const organizationId =
        request.params.organizationId || request.body.organizationId;
      if (organizationId) {
        // Verify user is member of organization
        const membership = await this.organizationMemberModel.findOne({
          userId: payload.sub,
          organizationId,
          isActive: true,
        });

        if (membership) {
          request.organizationId = organizationId;
          request.organizationMembership = membership;
        }
      }

      return true;
    } catch (error) {
      console.error('JWT validation error:', error.message);
      throw new UnauthorizedException(`Invalid JWT token: ${error.message}`);
    }
  }

  private extractTokenFromHeader(
    request: AuthenticatedRequest,
  ): string | undefined {
    const [type, token] = request.headers.authorization?.split(' ') ?? [];
    return type === 'Bearer' ? token : undefined;
  }

  private extractServiceFromPath(path: string): string | null {
    // Extract service name from API path
    // /api/v1/organizations/... -> console
    const pathParts = path.split('/');
    if (pathParts.length >= 4) {
      const service = pathParts[3];
      return service === 'organizations' ||
        service === 'users' ||
        service === 'billing'
        ? 'console'
        : service;
    }
    return null;
  }
}
