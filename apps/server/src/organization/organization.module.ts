import { Module } from '@nestjs/common';
import { MongooseModule } from '@nestjs/mongoose';
import { CommonModule } from '@/common';
import { AuthModule } from '@/auth';
import { SharesModule } from '@/shares';
import { OrganizationController } from './organization.controller';
import { OrganizationService } from './organization.service';
import {
  OrganizationDocument,
  OrganizationSchema,
} from './organization.schema';

@Module({
  imports: [
    AuthModule, // Provides ApiKeyService
    CommonModule, // Provides shared services (FinancialService, AuditService, PermissionService)
    SharesModule, // Provides SharesService
    MongooseModule.forFeature([
      { name: OrganizationDocument.name, schema: OrganizationSchema },
    ]),
  ],
  controllers: [OrganizationController],
  providers: [OrganizationService],
  exports: [OrganizationService],
})
export class OrganizationModule {}
