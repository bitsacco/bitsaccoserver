import { Module } from '@nestjs/common';
import { JwtModule } from '@nestjs/jwt';
import { MongooseModule } from '@nestjs/mongoose';
import { ConfigModule, ConfigService } from '@nestjs/config';

import { UsageTrackingMiddleware } from './usage-tracking.middleware';
import {
  ApiKeyService,
  MetricsService,
  AuditService,
  PermissionService,
} from './services';
import {
  TransactionLogDocument,
  TransactionLogSchema,
  ApiKeyDocument,
  ApiKeySchema,
  OrganizationServiceDocument,
  OrganizationServiceSchema,
  AuditTrail,
  AuditTrailSchema,
} from './schemas';

@Module({
  imports: [
    MongooseModule.forFeature([
      { name: TransactionLogDocument.name, schema: TransactionLogSchema },
      { name: ApiKeyDocument.name, schema: ApiKeySchema },
      {
        name: OrganizationServiceDocument.name,
        schema: OrganizationServiceSchema,
      },
      {
        name: AuditTrail.name,
        schema: AuditTrailSchema,
      },
    ]),
    JwtModule.registerAsync({
      imports: [ConfigModule],
      inject: [ConfigService],
      useFactory: (configService: ConfigService) => ({
        secret:
          configService.get<string>('JWT_SECRET') || 'fallback-secret-key',
        signOptions: {
          expiresIn: configService.get<string>('JWT_EXPIRES_IN') || '24h',
        },
      }),
    }),
  ],
  controllers: [],
  providers: [
    ApiKeyService,
    UsageTrackingMiddleware,
    MetricsService,
    AuditService,
    PermissionService,
  ],
  exports: [
    ApiKeyService,
    UsageTrackingMiddleware,
    MetricsService,
    AuditService,
    PermissionService,
    MongooseModule,
    JwtModule,
  ],
})
export class CommonModule {}
