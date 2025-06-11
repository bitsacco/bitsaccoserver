import { Module, MiddlewareConsumer, NestModule } from '@nestjs/common';

import { CommonModule } from '@/common';
import { ManagerController } from './manager.controller';
import { OrganizationModule } from './organization.module';
import { UsageTrackingMiddleware } from './usage-tracking.middleware';

@Module({
  imports: [CommonModule, OrganizationModule],
  providers: [UsageTrackingMiddleware],
  controllers: [ManagerController],
})
export class ManagerModule implements NestModule {
  configure(consumer: MiddlewareConsumer) {
    consumer.apply(UsageTrackingMiddleware).forRoutes('*'); // Apply to all routes
  }
}
