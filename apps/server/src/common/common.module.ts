import { Module } from '@nestjs/common';
import { MongooseModule } from '@nestjs/mongoose';
import { JwtModule } from '@nestjs/jwt';
import { ConfigModule, ConfigService } from '@nestjs/config';
import { ApiKeyController } from './api-key.controller';
import { OrganizationService } from './organization.service';
import { ApiKeyService } from './api-key.service';
import { UnifiedAuthGuard } from './guards/unified-auth.guard';
import { RBACGuard } from './guards/rbac.guard';
import { UsageTrackingMiddleware } from './usage-tracking.middleware';
import { MetricsService } from './metrics.service';
import {
  OrganizationDocument,
  OrganizationSchema,
  OrganizationMember,
  OrganizationMemberSchema,
  TransactionLogDocument,
  TransactionLogSchema,
  ApiKeyDocument,
  ApiKeySchema,
  OrganizationServiceDocument,
  OrganizationServiceSchema,
} from './schemas';

@Module({
  imports: [
    MongooseModule.forFeature([
      { name: OrganizationDocument.name, schema: OrganizationSchema },
      { name: OrganizationMember.name, schema: OrganizationMemberSchema },
      { name: TransactionLogDocument.name, schema: TransactionLogSchema },
      { name: ApiKeyDocument.name, schema: ApiKeySchema },
      {
        name: OrganizationServiceDocument.name,
        schema: OrganizationServiceSchema,
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
  controllers: [ApiKeyController],
  providers: [
    ApiKeyService,
    OrganizationService,
    UnifiedAuthGuard,
    RBACGuard,
    UsageTrackingMiddleware,
    MetricsService,
  ],
  exports: [
    ApiKeyService,
    OrganizationService,
    UnifiedAuthGuard,
    RBACGuard,
    UsageTrackingMiddleware,
    MetricsService,
    MongooseModule,
    JwtModule,
  ],
})
export class CommonModule {}
