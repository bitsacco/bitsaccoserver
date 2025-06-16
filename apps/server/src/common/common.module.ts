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
  SaccoService,
  FinancialService,
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
  Sacco,
  SaccoSchema,
  Chama,
  ChamaSchema,
  SaccoMember,
  SaccoMemberSchema,
  SaccoMembership,
  SaccoMembershipSchema,
  ChamaMembership,
  ChamaMembershipSchema,
  GroupRelationship,
  GroupRelationshipSchema,
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
      { name: Sacco.name, schema: SaccoSchema },
      { name: Chama.name, schema: ChamaSchema },
      { name: SaccoMember.name, schema: SaccoMemberSchema },
      { name: SaccoMembership.name, schema: SaccoMembershipSchema },
      { name: ChamaMembership.name, schema: ChamaMembershipSchema },
      { name: GroupRelationship.name, schema: GroupRelationshipSchema },
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
    SaccoService,
    FinancialService,
  ],
  exports: [
    ApiKeyService,
    UsageTrackingMiddleware,
    MetricsService,
    AuditService,
    PermissionService,
    SaccoService,
    FinancialService,
    MongooseModule,
    JwtModule,
  ],
})
export class CommonModule {}
