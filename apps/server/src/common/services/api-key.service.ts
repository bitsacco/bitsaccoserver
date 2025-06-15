import { Model } from 'mongoose';
import * as bcrypt from 'bcrypt';
import * as crypto from 'crypto';
import {
  Injectable,
  NotFoundException,
  ConflictException,
} from '@nestjs/common';
import { InjectModel } from '@nestjs/mongoose';
import { ApiKeyDocument, ApiKeyStatus } from '../schemas/api-key.schema';
import { CreateApiKeyDto, UpdateApiKeyDto } from '../dto/api-key.dto';

@Injectable()
export class ApiKeyService {
  constructor(
    @InjectModel(ApiKeyDocument.name)
    private apiKeyModel: Model<ApiKeyDocument>,
  ) {}

  async create(
    organizationId: string,
    createdBy: string,
    createApiKeyDto: CreateApiKeyDto,
  ) {
    // Generate API key
    const plainKey = `mk_${crypto.randomBytes(16).toString('hex')}`;
    const keyId = plainKey.substring(0, 8);
    const hashedKey = await bcrypt.hash(plainKey, 10);

    // Check for name conflicts within organization
    const existingKey = await this.apiKeyModel.findOne({
      organizationId,
      name: createApiKeyDto.name,
      status: { $ne: ApiKeyStatus.DISABLED },
    });

    if (existingKey) {
      throw new ConflictException(
        `API key with name '${createApiKeyDto.name}' already exists`,
      );
    }

    const apiKey = new this.apiKeyModel({
      keyId,
      hashedKey,
      name: createApiKeyDto.name,
      description: createApiKeyDto.description,
      organizationId,
      createdBy,
      serviceIds: createApiKeyDto.serviceIds || [],
      permissions: createApiKeyDto.permissions || [],
      status: ApiKeyStatus.ACTIVE,
      limits: createApiKeyDto.limits || {},
      usage: {
        totalRequests: 0,
        currentMonth: { requests: 0, volume: 0, costs: 0 },
        lastMonth: { requests: 0, volume: 0, costs: 0 },
      },
      allowedIps: createApiKeyDto.allowedIps || [],
      allowedDomains: createApiKeyDto.allowedDomains || [],
      expiresAt: createApiKeyDto.expiresAt,
    });

    const savedKey = await apiKey.save();

    return {
      apiKey: {
        id: savedKey._id,
        keyId: savedKey.keyId,
        name: savedKey.name,
        description: savedKey.description,
        serviceIds: savedKey.serviceIds,
        permissions: savedKey.permissions,
        status: savedKey.status,
        limits: savedKey.limits,
        createdAt: savedKey.createdAt,
        expiresAt: savedKey.expiresAt,
      },
      plainKey,
      warning: 'Store this API key securely. It will not be shown again.',
    };
  }

  async findAll(organizationId: string) {
    const apiKeys = await this.apiKeyModel
      .find({
        organizationId,
        status: { $ne: ApiKeyStatus.DISABLED },
      })
      .select('-hashedKey')
      .sort({ createdAt: -1 });

    return apiKeys.map((key) => ({
      id: key._id,
      keyId: key.keyId,
      name: key.name,
      description: key.description,
      serviceIds: key.serviceIds,
      permissions: key.permissions,
      status: key.status,
      limits: key.limits,
      usage: key.usage,
      allowedIps: key.allowedIps,
      allowedDomains: key.allowedDomains,
      lastUsedAt: key.lastUsedAt,
      createdAt: key.createdAt,
      expiresAt: key.expiresAt,
    }));
  }

  async findOne(organizationId: string, keyId: string) {
    const apiKey = await this.apiKeyModel
      .findOne({
        organizationId,
        keyId: keyId,
        status: { $ne: ApiKeyStatus.DISABLED },
      })
      .select('-hashedKey');

    if (!apiKey) {
      throw new NotFoundException('API key not found');
    }

    return {
      id: apiKey._id,
      keyId: apiKey.keyId,
      name: apiKey.name,
      description: apiKey.description,
      serviceIds: apiKey.serviceIds,
      permissions: apiKey.permissions,
      status: apiKey.status,
      limits: apiKey.limits,
      usage: apiKey.usage,
      allowedIps: apiKey.allowedIps,
      allowedDomains: apiKey.allowedDomains,
      lastUsedAt: apiKey.lastUsedAt,
      createdAt: apiKey.createdAt,
      expiresAt: apiKey.expiresAt,
    };
  }

  async update(
    organizationId: string,
    keyId: string,
    updateApiKeyDto: UpdateApiKeyDto,
  ) {
    const apiKey = await this.apiKeyModel.findOne({
      organizationId,
      keyId: keyId,
      status: { $ne: ApiKeyStatus.DISABLED },
    });

    if (!apiKey) {
      throw new NotFoundException('API key not found');
    }

    // Check if new name conflicts with existing keys
    if (updateApiKeyDto.name && updateApiKeyDto.name !== apiKey.name) {
      const existingKey = await this.apiKeyModel.findOne({
        organizationId,
        name: updateApiKeyDto.name,
        keyId: { $ne: keyId },
        status: { $ne: ApiKeyStatus.DISABLED },
      });

      if (existingKey) {
        throw new ConflictException(
          `API key with name '${updateApiKeyDto.name}' already exists`,
        );
      }
    }

    Object.assign(apiKey, updateApiKeyDto);
    const updatedKey = await apiKey.save();

    return {
      id: updatedKey._id,
      keyId: updatedKey.keyId,
      name: updatedKey.name,
      description: updatedKey.description,
      serviceIds: updatedKey.serviceIds,
      permissions: updatedKey.permissions,
      status: updatedKey.status,
      limits: updatedKey.limits,
      usage: updatedKey.usage,
      allowedIps: updatedKey.allowedIps,
      allowedDomains: updatedKey.allowedDomains,
      lastUsedAt: updatedKey.lastUsedAt,
      createdAt: updatedKey.createdAt,
      expiresAt: updatedKey.expiresAt,
    };
  }

  async remove(organizationId: string, keyId: string) {
    const result = await this.apiKeyModel.findOneAndUpdate(
      {
        organizationId,
        keyId: keyId,
        status: { $ne: ApiKeyStatus.DISABLED },
      },
      { status: ApiKeyStatus.DISABLED },
      { new: true },
    );

    if (!result) {
      throw new NotFoundException('API key not found');
    }

    return { message: 'API key revoked successfully' };
  }

  async getUsage(organizationId: string, keyId: string) {
    const apiKey = await this.apiKeyModel.findOne({
      organizationId,
      keyId: keyId,
      status: { $ne: ApiKeyStatus.DISABLED },
    });

    if (!apiKey) {
      throw new NotFoundException('API key not found');
    }

    return {
      keyId: apiKey.keyId,
      usage: apiKey.usage,
      limits: apiKey.limits,
      status: apiKey.status,
      lastUsedAt: apiKey.lastUsedAt,
    };
  }

  async incrementUsage(keyId: string, volume = 0, cost = 0): Promise<void> {
    await this.apiKeyModel.updateOne(
      { keyId, status: ApiKeyStatus.ACTIVE },
      {
        $inc: {
          'usage.totalRequests': 1,
          'usage.currentMonth.requests': 1,
          'usage.currentMonth.volume': volume,
          'usage.currentMonth.costs': cost,
        },
        $set: {
          lastUsedAt: new Date(),
        },
      },
    );
  }

  async resetMonthlyUsage(): Promise<void> {
    await this.apiKeyModel.updateMany(
      {},
      {
        $set: {
          'usage.lastMonth': '$usage.currentMonth',
          'usage.currentMonth': { requests: 0, volume: 0, costs: 0 },
        },
      },
    );
  }

  hasPermission(apiKey: ApiKeyDocument, permission: string): boolean {
    return (
      apiKey.permissions.includes(permission) ||
      apiKey.permissions.includes('*')
    );
  }

  hasServiceAccess(apiKey: ApiKeyDocument, serviceId: string): boolean {
    return (
      apiKey.serviceIds.includes(serviceId) || apiKey.serviceIds.includes('*')
    );
  }

  isWithinLimits(apiKey: ApiKeyDocument): boolean {
    const { limits, usage } = apiKey;

    if (
      limits.requestsPerDay &&
      usage.currentMonth.requests >= limits.requestsPerDay * 30
    ) {
      return false;
    }

    if (
      limits.monthlyVolume &&
      usage.currentMonth.volume >= limits.monthlyVolume
    ) {
      return false;
    }

    return true;
  }
}
