import { Module } from '@nestjs/common';
import { MongooseModule } from '@nestjs/mongoose';
import { CommonModule } from '@/common';
import { AuthModule } from '@/auth';
import { OrganizationController } from './organization.controller';
import { OrganizationService } from './organization.service';
import {
  OrganizationDocument,
  OrganizationSchema,
  OrganizationMember,
  OrganizationMemberSchema,
} from './organization.schema';

@Module({
  imports: [
    CommonModule, // Provides shared services
    AuthModule, // Provides ApiKeyService
    MongooseModule.forFeature([
      { name: OrganizationDocument.name, schema: OrganizationSchema },
      { name: OrganizationMember.name, schema: OrganizationMemberSchema },
    ]),
  ],
  controllers: [OrganizationController],
  providers: [OrganizationService],
  exports: [OrganizationService],
})
export class OrganizationModule {}
