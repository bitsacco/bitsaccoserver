import { Module, MiddlewareConsumer, NestModule } from '@nestjs/common';
import { MongooseModule } from '@nestjs/mongoose';

import { CommonModule, UsageTrackingMiddleware } from '@/common';
import {
  SharesDocument,
  SharesSchema,
  SharesOfferDocument,
  SharesOfferSchema,
} from '@/common/schemas';
import { SharesController } from './shares.controller';
import { SharesService } from './shares.service';

@Module({
  imports: [
    CommonModule,
    MongooseModule.forFeature([
      { name: SharesDocument.name, schema: SharesSchema },
      { name: SharesOfferDocument.name, schema: SharesOfferSchema },
    ]),
  ],
  providers: [SharesService],
  controllers: [SharesController],
  exports: [SharesService],
})
export class SharesModule implements NestModule {
  configure(consumer: MiddlewareConsumer) {
    consumer.apply(UsageTrackingMiddleware).forRoutes('*');
  }
}
