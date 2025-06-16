import { Module, MiddlewareConsumer, NestModule } from '@nestjs/common';
import { MongooseModule } from '@nestjs/mongoose';

import {
  CommonModule,
  UsageTrackingMiddleware,
  SaccoService,
  PermissionService,
} from '@/common';
import {
  SharesDocument,
  SharesSchema,
  SharesOfferDocument,
  SharesOfferSchema,
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
} from '@/common/schemas';
import { SharesController } from './shares.controller';
import { SharesService } from './shares.service';

@Module({
  imports: [
    CommonModule,
    MongooseModule.forFeature([
      { name: SharesDocument.name, schema: SharesSchema },
      { name: SharesOfferDocument.name, schema: SharesOfferSchema },
      { name: Sacco.name, schema: SaccoSchema },
      { name: Chama.name, schema: ChamaSchema },
      { name: SaccoMember.name, schema: SaccoMemberSchema },
      { name: SaccoMembership.name, schema: SaccoMembershipSchema },
      { name: ChamaMembership.name, schema: ChamaMembershipSchema },
      { name: GroupRelationship.name, schema: GroupRelationshipSchema },
    ]),
  ],
  providers: [SharesService, SaccoService, PermissionService],
  controllers: [SharesController],
  exports: [SharesService],
})
export class SharesModule implements NestModule {
  configure(consumer: MiddlewareConsumer) {
    consumer.apply(UsageTrackingMiddleware).forRoutes('*');
  }
}
