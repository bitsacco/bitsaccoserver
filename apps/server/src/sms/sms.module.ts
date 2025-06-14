import { Module, MiddlewareConsumer, NestModule } from '@nestjs/common';

import { CommonModule, UsageTrackingMiddleware } from '@/common';
import { SmsController } from './sms.controller';
import { SmsService } from './sms.service';

@Module({
  imports: [CommonModule],
  providers: [SmsService],
  controllers: [SmsController],
})
export class SmsModule implements NestModule {
  configure(consumer: MiddlewareConsumer) {
    consumer.apply(UsageTrackingMiddleware).forRoutes('*'); // Apply to all routes
  }
}
