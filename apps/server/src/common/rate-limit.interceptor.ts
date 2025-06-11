import { Model } from 'mongoose';
import { Observable } from 'rxjs';
import { InjectModel } from '@nestjs/mongoose';
import {
  Injectable,
  NestInterceptor,
  ExecutionContext,
  CallHandler,
  HttpException,
  HttpStatus,
} from '@nestjs/common';
import { ApiKeyDocument } from './schemas/api-key.schema';
import { OrganizationServiceDocument } from './schemas/service.schema';
import { TransactionLogDocument } from './schemas/transaction-log.schema';

@Injectable()
export class RateLimitInterceptor implements NestInterceptor {
  constructor(
    @InjectModel(ApiKeyDocument.name)
    private apiKeyModel: Model<ApiKeyDocument>,

    @InjectModel(OrganizationServiceDocument.name)
    private organizationServiceModel: Model<OrganizationServiceDocument>,

    @InjectModel(TransactionLogDocument.name)
    private transactionLogModel: Model<TransactionLogDocument>,
  ) {}

  async intercept(
    context: ExecutionContext,
    next: CallHandler,
  ): Promise<Observable<any>> {
    const request = context.switchToHttp().getRequest();
    const organizationId = request.organizationId;
    const apiKeyId = request.apiKeyId;

    if (organizationId && apiKeyId) {
      await this.checkRateLimits(organizationId, apiKeyId);
    }

    return next.handle();
  }

  private async checkRateLimits(
    organizationId: string,
    apiKeyId: string,
  ): Promise<void> {
    const now = new Date();
    const oneMinuteAgo = new Date(now.getTime() - 60000);
    const oneDayAgo = new Date(now.getTime() - 86400000);

    // Get API key limits
    const apiKey = await this.apiKeyModel.findById(apiKeyId);
    if (!apiKey) return;

    // Get organization service limits
    const orgService = await this.organizationServiceModel.findOne({
      organizationId,
      serviceId: '', // TODO: Make dynamic
    });

    // Check per-minute limits
    if (
      apiKey.limits?.requestsPerMinute ||
      orgService?.customLimits?.requestsPerMinute
    ) {
      const minuteLimit =
        apiKey.limits?.requestsPerMinute ||
        orgService?.customLimits?.requestsPerMinute;
      const minuteCount = await this.transactionLogModel.countDocuments({
        apiKeyId,
        timestamp: { $gte: oneMinuteAgo },
        status: 'success',
      });

      if (minuteCount >= minuteLimit) {
        throw new HttpException(
          'Rate limit exceeded: too many requests per minute',
          HttpStatus.TOO_MANY_REQUESTS,
        );
      }
    }

    // Check per-day limits
    if (
      apiKey.limits?.requestsPerDay ||
      orgService?.customLimits?.requestsPerDay
    ) {
      const dayLimit =
        apiKey.limits?.requestsPerDay ||
        orgService?.customLimits?.requestsPerDay;
      const dayCount = await this.transactionLogModel.countDocuments({
        apiKeyId,
        timestamp: { $gte: oneDayAgo },
        status: 'success',
      });

      if (dayCount >= dayLimit) {
        throw new HttpException(
          'Rate limit exceeded: too many requests per day',
          HttpStatus.TOO_MANY_REQUESTS,
        );
      }
    }

    // Check monthly volume limits
    if (
      apiKey.limits?.monthlyVolume ||
      orgService?.customLimits?.monthlyVolume
    ) {
      const monthStart = new Date(now.getFullYear(), now.getMonth(), 1);
      const volumeLimit =
        apiKey.limits?.monthlyVolume || orgService?.customLimits?.monthlyVolume;

      const monthlyVolume = await this.transactionLogModel.aggregate([
        {
          $match: {
            apiKeyId,
            timestamp: { $gte: monthStart },
            volume: { $exists: true },
          },
        },
        {
          $group: {
            _id: null,
            totalVolume: { $sum: '$volume' },
          },
        },
      ]);

      const currentVolume = monthlyVolume[0]?.totalVolume || 0;
      if (currentVolume >= volumeLimit) {
        throw new HttpException(
          'Rate limit exceeded: monthly volume limit reached',
          HttpStatus.TOO_MANY_REQUESTS,
        );
      }
    }
  }
}
