import * as bcrypt from 'bcrypt';
import { JwtService } from '@nestjs/jwt';
import { getModelToken } from '@nestjs/mongoose';
import { Test, TestingModule } from '@nestjs/testing';
import { ExecutionContext, UnauthorizedException } from '@nestjs/common';
import { ApiKeyDocument } from '../schemas/api-key.schema';
import { OrganizationMember } from '../schemas/organization.schema';
import { OrganizationServiceDocument } from '../schemas/service.schema';
import { UnifiedAuthGuard } from './unified-auth.guard';

jest.mock('bcrypt');

describe('UnifiedAuthGuard', () => {
  let guard: UnifiedAuthGuard;
  let mockApiKeyModel: any;
  let mockOrganizationMemberModel: any;
  let mockOrganizationServiceModel: any;
  let mockJwtService: any;

  beforeEach(async () => {
    mockApiKeyModel = {
      findOne: jest.fn(),
      save: jest.fn(),
    };

    mockOrganizationMemberModel = {
      findOne: jest.fn(),
    };

    mockOrganizationServiceModel = {
      findOne: jest.fn(),
    };

    mockJwtService = {
      verifyAsync: jest.fn(),
      decode: jest.fn(),
    };

    const module: TestingModule = await Test.createTestingModule({
      providers: [
        UnifiedAuthGuard,
        {
          provide: getModelToken(ApiKeyDocument.name),
          useValue: mockApiKeyModel,
        },
        {
          provide: getModelToken(OrganizationMember.name),
          useValue: mockOrganizationMemberModel,
        },
        {
          provide: getModelToken(OrganizationServiceDocument.name),
          useValue: mockOrganizationServiceModel,
        },
        {
          provide: JwtService,
          useValue: mockJwtService,
        },
      ],
    }).compile();

    guard = module.get<UnifiedAuthGuard>(UnifiedAuthGuard);
  });

  it('should be defined', () => {
    expect(guard).toBeDefined();
  });

  describe('canActivate', () => {
    it.skip('should allow access with valid API key', async () => {
      const mockRequest = {
        headers: {
          'x-api-key': 'mk_validapikey123456789012345678901',
        },
        path: '/api/v1/fooservice/quote',
      };

      const mockContext = {
        switchToHttp: () => ({
          getRequest: () => mockRequest,
        }),
      } as ExecutionContext;

      const mockApiKey = {
        _id: 'key-id',
        organizationId: 'org-id',
        keyId: 'mk_valid',
        hashedKey: 'hashed-key',
        serviceIds: ['fooservice'],
        permissions: ['fooservice:read', 'fooservice:write'],
        createdBy: 'user-id',
        lastUsedAt: new Date(),
        save: jest.fn(),
      };

      const mockOrgService = {
        organizationId: 'org-id',
        serviceId: 'fooservice',
        isEnabled: true,
      };

      mockApiKeyModel.findOne.mockResolvedValue(mockApiKey);
      mockOrganizationServiceModel.findOne.mockResolvedValue(mockOrgService);
      (bcrypt.compare as jest.Mock).mockResolvedValue(true);

      const result = await guard.canActivate(mockContext);

      expect(result).toBe(true);
      expect(mockRequest).toHaveProperty('user');
      expect(mockRequest).toHaveProperty('organizationId', 'org-id');
    });

    it('should allow access with valid JWT token', async () => {
      const mockRequest = {
        headers: {
          authorization: 'Bearer valid.jwt.token',
        },
        params: { organizationId: 'org-id' },
      };

      const mockContext = {
        switchToHttp: () => ({
          getRequest: () => mockRequest,
        }),
      } as ExecutionContext;

      const mockUserData = {
        sub: 'user-id',
        email: 'user@example.com',
      };

      const mockMembership = {
        userId: 'user-id',
        organizationId: 'org-id',
        role: 'admin',
        isActive: true,
      };

      mockJwtService.decode.mockReturnValue(mockUserData);
      mockOrganizationMemberModel.findOne.mockResolvedValue(mockMembership);

      const result = await guard.canActivate(mockContext);

      expect(result).toBe(true);
      expect(mockRequest).toHaveProperty('user');
      expect(mockRequest).toHaveProperty('organizationId', 'org-id');
    });

    it('should deny access without authentication', async () => {
      const mockRequest = {
        headers: {},
      };

      const mockContext = {
        switchToHttp: () => ({
          getRequest: () => mockRequest,
        }),
      } as ExecutionContext;

      await expect(guard.canActivate(mockContext)).rejects.toThrow(
        UnauthorizedException,
      );
    });

    it.skip('should deny access with invalid API key', async () => {
      const mockRequest = {
        headers: {
          'x-api-key': 'mk_invalidapikey123456789012345678901',
        },
        path: '/api/v1/fooservice/quote',
      };

      const mockContext = {
        switchToHttp: () => ({
          getRequest: () => mockRequest,
        }),
      } as ExecutionContext;

      mockApiKeyModel.findOne.mockResolvedValue(null);

      await expect(guard.canActivate(mockContext)).rejects.toThrow(
        UnauthorizedException,
      );
    });
  });
});
