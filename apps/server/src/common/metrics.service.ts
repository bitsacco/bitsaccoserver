import { Injectable, Logger } from '@nestjs/common';
import { InjectModel } from '@nestjs/mongoose';
import { Model } from 'mongoose';
import {
  TransactionLogDocument,
  TransactionType,
  TransactionStatus,
} from './schemas';

export interface SmsMetricData {
  receiver: string;
  messageLength: number;
  success: boolean;
  duration: number;
  errorType?: string;
}

export interface SmsBulkMetricData {
  receiverCount: number;
  messageLength: number;
  success: boolean;
  duration: number;
  errorType?: string;
}

export interface ApiMetricData {
  endpoint: string;
  method: string;
  statusCode: number;
  duration: number;
  success: boolean;
  requestSize?: number;
  responseSize?: number;
  userAgent?: string;
  clientIp?: string;
}

export interface AuthMetricData {
  action: 'login' | 'register' | 'token_refresh' | 'logout';
  success: boolean;
  duration: number;
  errorType?: string;
  userId?: string;
}

export interface SharesMetricData {
  userId: string;
  offerId?: string;
  quantity: number;
  success: boolean;
  duration: number;
  errorType?: string;
}

export interface SharesOwnershipMetricData {
  userId: string;
  quantity: number;
  percentageOfTotal: number;
  limitReached: boolean;
}

@Injectable()
export class MetricsService {
  private readonly logger = new Logger(MetricsService.name);

  constructor(
    @InjectModel(TransactionLogDocument.name)
    private transactionLogModel: Model<TransactionLogDocument>,
  ) {}

  /**
   * Record SMS metrics
   */
  async recordSmsMetric(
    data: SmsMetricData,
    organizationId?: string,
    apiKeyId?: string,
  ): Promise<void> {
    try {
      const logData = {
        organizationId,
        apiKeyId,
        serviceId: 'sms',
        type: TransactionType.SMS,
        status: data.success
          ? TransactionStatus.COMPLETE
          : TransactionStatus.FAILED,
        endpoint: '/sms/send',
        method: 'POST',
        statusCode: data.success ? 200 : 500,
        responseTime: data.duration,
        requestSize: data.messageLength,
        responseSize: 0,
        timestamp: new Date(),
        metadata: {
          receiver: data.receiver,
          messageLength: data.messageLength,
          errorType: data.errorType,
        },
      };

      await this.transactionLogModel.create(logData);
      this.logger.debug(`SMS metric recorded for ${data.receiver}`);
    } catch (error) {
      this.logger.error('Failed to record SMS metric:', error);
    }
  }

  /**
   * Record bulk SMS metrics
   */
  async recordSmsBulkMetric(
    data: SmsBulkMetricData,
    organizationId?: string,
    apiKeyId?: string,
  ): Promise<void> {
    try {
      const logData = {
        organizationId,
        apiKeyId,
        serviceId: 'sms',
        type: TransactionType.SMS,
        status: data.success
          ? TransactionStatus.COMPLETE
          : TransactionStatus.FAILED,
        endpoint: '/sms/send-bulk',
        method: 'POST',
        statusCode: data.success ? 200 : 500,
        responseTime: data.duration,
        requestSize: data.messageLength,
        responseSize: 0,
        timestamp: new Date(),
        metadata: {
          receiverCount: data.receiverCount,
          messageLength: data.messageLength,
          errorType: data.errorType,
        },
      };

      await this.transactionLogModel.create(logData);
      this.logger.debug(
        `Bulk SMS metric recorded for ${data.receiverCount} receivers`,
      );
    } catch (error) {
      this.logger.error('Failed to record bulk SMS metric:', error);
    }
  }

  /**
   * Record API metrics
   */
  async recordApiMetric(
    data: ApiMetricData,
    organizationId?: string,
    apiKeyId?: string,
  ): Promise<void> {
    try {
      const logData = {
        organizationId,
        apiKeyId,
        serviceId: 'api',
        type: TransactionType.API_REQUEST,
        status: data.success
          ? TransactionStatus.COMPLETE
          : TransactionStatus.FAILED,
        endpoint: data.endpoint,
        method: data.method,
        statusCode: data.statusCode,
        responseTime: data.duration,
        requestSize: data.requestSize || 0,
        responseSize: data.responseSize || 0,
        clientIp: data.clientIp,
        userAgent: data.userAgent,
        timestamp: new Date(),
        metadata: {},
      };

      await this.transactionLogModel.create(logData);
      this.logger.debug(`API metric recorded for ${data.endpoint}`);
    } catch (error) {
      this.logger.error('Failed to record API metric:', error);
    }
  }

  /**
   * Record authentication metrics
   */
  async recordAuthMetric(
    data: AuthMetricData,
    organizationId?: string,
    apiKeyId?: string,
  ): Promise<void> {
    try {
      const logData = {
        organizationId,
        apiKeyId,
        serviceId: 'auth',
        type: TransactionType.AUTH,
        status: data.success
          ? TransactionStatus.COMPLETE
          : TransactionStatus.FAILED,
        endpoint: `/auth/${data.action}`,
        method: 'POST',
        statusCode: data.success ? 200 : 401,
        responseTime: data.duration,
        requestSize: 0,
        responseSize: 0,
        timestamp: new Date(),
        metadata: {
          action: data.action,
          userId: data.userId,
          errorType: data.errorType,
        },
      };

      await this.transactionLogModel.create(logData);
      this.logger.debug(`Auth metric recorded for ${data.action}`);
    } catch (error) {
      this.logger.error('Failed to record auth metric:', error);
    }
  }

  /**
   * Record custom metric
   */
  async recordCustomMetric(
    serviceId: string,
    type: TransactionType,
    endpoint: string,
    data: {
      success: boolean;
      duration: number;
      statusCode?: number;
      method?: string;
      metadata?: Record<string, any>;
    },
    organizationId?: string,
    apiKeyId?: string,
  ): Promise<void> {
    try {
      const logData = {
        organizationId,
        apiKeyId,
        serviceId,
        type,
        status: data.success
          ? TransactionStatus.COMPLETE
          : TransactionStatus.FAILED,
        endpoint,
        method: data.method || 'POST',
        statusCode: data.statusCode || (data.success ? 200 : 500),
        responseTime: data.duration,
        requestSize: 0,
        responseSize: 0,
        timestamp: new Date(),
        metadata: data.metadata || {},
      };

      await this.transactionLogModel.create(logData);
      this.logger.debug(`Custom metric recorded for ${serviceId}:${endpoint}`);
    } catch (error) {
      this.logger.error('Failed to record custom metric:', error);
    }
  }

  /**
   * Get metrics for a specific organization and time range
   */
  async getMetrics(
    organizationId: string,
    startDate?: Date,
    endDate?: Date,
    serviceId?: string,
  ) {
    const filter: any = { organizationId };

    if (startDate && endDate) {
      filter.timestamp = { $gte: startDate, $lte: endDate };
    }

    if (serviceId) {
      filter.serviceId = serviceId;
    }

    try {
      const metrics = await this.transactionLogModel
        .find(filter)
        .sort({ timestamp: -1 })
        .lean();

      return metrics;
    } catch (error) {
      this.logger.error('Failed to fetch metrics:', error);
      throw error;
    }
  }

  /**
   * Record shares subscription metrics
   * Note: Shares are managed globally, not by organizations
   */
  async recordSharesSubscriptionMetric(
    data: SharesMetricData,
    organizationId?: string,
    apiKeyId?: string,
  ): Promise<void> {
    try {
      const logData = {
        organizationId,
        apiKeyId,
        serviceId: 'shares',
        type: TransactionType.API_REQUEST,
        status: data.success
          ? TransactionStatus.COMPLETE
          : TransactionStatus.FAILED,
        endpoint: '/shares/subscribe',
        method: 'POST',
        statusCode: data.success ? 200 : 400,
        responseTime: data.duration,
        requestSize: 0,
        responseSize: 0,
        timestamp: new Date(),
        metadata: {
          userId: data.userId,
          offerId: data.offerId,
          quantity: data.quantity,
          errorType: data.errorType,
        },
      };

      await this.transactionLogModel.create(logData);
      this.logger.debug(
        `Shares subscription metric recorded for user ${data.userId}`,
      );
    } catch (error) {
      this.logger.error('Failed to record shares subscription metric:', error);
    }
  }

  /**
   * Record shares transfer metrics
   * Note: Shares are managed globally, not by organizations
   */
  async recordSharesTransferMetric(
    data: SharesMetricData & { fromUserId: string; toUserId: string },
    organizationId?: string,
    apiKeyId?: string,
  ): Promise<void> {
    try {
      const logData = {
        organizationId,
        apiKeyId,
        serviceId: 'shares',
        type: TransactionType.API_REQUEST,
        status: data.success
          ? TransactionStatus.COMPLETE
          : TransactionStatus.FAILED,
        endpoint: '/shares/transfer',
        method: 'POST',
        statusCode: data.success ? 200 : 400,
        responseTime: data.duration,
        requestSize: 0,
        responseSize: 0,
        timestamp: new Date(),
        metadata: {
          fromUserId: data.fromUserId,
          toUserId: data.toUserId,
          quantity: data.quantity,
          errorType: data.errorType,
        },
      };

      await this.transactionLogModel.create(logData);
      this.logger.debug(
        `Shares transfer metric recorded from ${data.fromUserId} to ${data.toUserId}`,
      );
    } catch (error) {
      this.logger.error('Failed to record shares transfer metric:', error);
    }
  }

  /**
   * Record shares ownership metrics
   * Note: Shares are managed globally, not by organizations
   */
  async recordSharesOwnershipMetric(
    data: SharesOwnershipMetricData,
    organizationId?: string,
    apiKeyId?: string,
  ): Promise<void> {
    try {
      const logData = {
        organizationId,
        apiKeyId,
        serviceId: 'shares',
        type: TransactionType.API_REQUEST,
        status: TransactionStatus.COMPLETE,
        endpoint: '/shares/ownership',
        method: 'POST',
        statusCode: 200,
        responseTime: 0,
        requestSize: 0,
        responseSize: 0,
        timestamp: new Date(),
        metadata: {
          userId: data.userId,
          quantity: data.quantity,
          percentageOfTotal: data.percentageOfTotal,
          limitReached: data.limitReached,
        },
      };

      await this.transactionLogModel.create(logData);
      this.logger.debug(
        `Shares ownership metric recorded for user ${data.userId}: ${data.percentageOfTotal}%`,
      );
    } catch (error) {
      this.logger.error('Failed to record shares ownership metric:', error);
    }
  }

  /**
   * Record generic metric (backwards compatibility)
   */
  async recordMetric(
    metricType: string,
    data: Record<string, any>,
    organizationId?: string,
    apiKeyId?: string,
  ): Promise<void> {
    try {
      const logData = {
        organizationId,
        apiKeyId,
        serviceId: 'generic',
        type: TransactionType.API_REQUEST,
        status: data.success
          ? TransactionStatus.COMPLETE
          : TransactionStatus.FAILED,
        endpoint: `/${metricType}`,
        method: 'POST',
        statusCode: data.success ? 200 : 500,
        responseTime: data.duration || 0,
        requestSize: 0,
        responseSize: 0,
        timestamp: new Date(),
        metadata: data,
      };

      await this.transactionLogModel.create(logData);
      this.logger.debug(`Generic metric recorded for ${metricType}`);
    } catch (error) {
      this.logger.error('Failed to record generic metric:', error);
    }
  }

  /**
   * Get aggregated metrics summary
   */
  async getMetricsSummary(
    organizationId: string,
    startDate?: Date,
    endDate?: Date,
  ) {
    const filter: any = { organizationId };

    if (startDate && endDate) {
      filter.timestamp = { $gte: startDate, $lte: endDate };
    }

    try {
      const summary = await this.transactionLogModel.aggregate([
        { $match: filter },
        {
          $group: {
            _id: {
              serviceId: '$serviceId',
              status: '$status',
            },
            count: { $sum: 1 },
            avgResponseTime: { $avg: '$responseTime' },
            totalRequestSize: { $sum: '$requestSize' },
            totalResponseSize: { $sum: '$responseSize' },
          },
        },
        {
          $group: {
            _id: '$_id.serviceId',
            stats: {
              $push: {
                status: '$_id.status',
                count: '$count',
                avgResponseTime: '$avgResponseTime',
                totalRequestSize: '$totalRequestSize',
                totalResponseSize: '$totalResponseSize',
              },
            },
          },
        },
      ]);

      return summary;
    } catch (error) {
      this.logger.error('Failed to fetch metrics summary:', error);
      throw error;
    }
  }
}
