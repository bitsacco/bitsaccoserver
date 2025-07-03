import * as Joi from 'joi';
import type { RedisClientOptions } from 'redis';
import { redisStore } from 'cache-manager-redis-store';
import { Module, MiddlewareConsumer, NestModule } from '@nestjs/common';
import { ConfigModule, ConfigService } from '@nestjs/config';
import { MongooseModule } from '@nestjs/mongoose';
import { CacheModule } from '@nestjs/cache-manager';
import { ThrottlerModule } from '@nestjs/throttler';
import { EventEmitterModule } from '@nestjs/event-emitter';

import { ApiController } from './api.controller';
import { ComplianceController } from './compliance.controller';
import { ApiKeyController } from './api-key.controller';
import { AdminController } from './admin.controller';
import { ApiService } from './api.service';
import { AuthModule } from '../auth/auth.module';
import { SmsModule } from '../sms/sms.module';
import { SharesModule } from '../shares/shares.module';
import { OrganizationModule } from '../organization';
import { BaseModule } from '../base/base.module';
import { CommonModule } from '../common/common.module';
import { JwtAuthMiddleware } from '../common';
import { LoanService } from '../loans';

@Module({
  imports: [
    // Configuration
    ConfigModule.forRoot({
      isGlobal: true,
      envFilePath: ['.env.local', '.env'],
      validationSchema: Joi.object({
        NODE_ENV: Joi.string()
          .valid('development', 'production', 'test')
          .default('development'),
        PORT: Joi.number().default(4000),

        // Database
        DATABASE_URL: Joi.string().required(),

        // Redis
        REDIS_HOST: Joi.string().required(),
        REDIS_PORT: Joi.number().required(),
        REDIS_PASSWORD: Joi.string().required(),
        REDIS_TLS: Joi.boolean().default(false),

        // Fedimint
        FEDIMINT_CLIENTD_BASE_URL: Joi.string().required(),
        FEDIMINT_CLIENTD_PASSWORD: Joi.string().required(),
        FEDIMINT_FEDERATION_ID: Joi.string().required(),
        FEDIMINT_GATEWAY_ID: Joi.string().required(),

        // Auth & JWT
        JWT_SECRET: Joi.string().required(),
        JWT_EXPIRES_IN: Joi.string().default('24h'),
        API_KEY_SECRET: Joi.string().required(),

        // Keycloak
        KEYCLOAK_AUTH_SERVER_URL: Joi.string().required(),
        KEYCLOAK_REALM: Joi.string().required(),
        KEYCLOAK_CLIENT_ID: Joi.string().required(),
        KEYCLOAK_CLIENT_SECRET: Joi.string().required(),

        // SMS
        SMS_AT_API_KEY: Joi.string(),
        SMS_AT_USERNAME: Joi.string(),
        SMS_AT_FROM: Joi.string(),
        SMS_AT_KEYWORD: Joi.string(),

        // CORS
        CORS_ORIGIN: Joi.string().default('*'),
      }),
    }),

    // Database
    MongooseModule.forRootAsync({
      inject: [ConfigService],
      useFactory: (configService: ConfigService) => ({
        uri: configService.get<string>('DATABASE_URL'),
      }),
    }),

    // Cache with Redis (optional)
    CacheModule.registerAsync<RedisClientOptions>({
      isGlobal: true,
      inject: [ConfigService],
      useFactory: async (configService: ConfigService) => {
        const redisConfig = {
          store: redisStore as any,
          host: configService.get<string>('REDIS_HOST'),
          port: configService.get<number>('REDIS_PORT'),
          password: configService.get<string>('REDIS_PASSWORD'),
          ttl: 3600, // 1 hour default TTL
        };

        // Return in-memory cache if Redis is not available
        const nodeEnv = configService.get<string>('NODE_ENV');
        const redisHost = configService.get<string>('REDIS_HOST');
        if (nodeEnv === 'development' || !redisHost) {
          return { ttl: 3600 }; // In-memory cache
        }

        return redisConfig;
      },
    }),

    // Rate limiting
    ThrottlerModule.forRoot([
      {
        ttl: 60000, // 1 minute
        limit: 10, // 10 requests per minute
      },
    ]),

    // Event system
    EventEmitterModule.forRoot(),

    // Feature modules
    CommonModule,
    AuthModule,
    SmsModule,
    SharesModule,
    OrganizationModule,
    BaseModule,
  ],
  controllers: [
    ApiController,
    ComplianceController,
    ApiKeyController,
    AdminController,
  ],
  providers: [ApiService, LoanService],
})
export class ApiModule implements NestModule {
  configure(consumer: MiddlewareConsumer) {
    consumer.apply(JwtAuthMiddleware).forRoutes('*'); // Apply to all routes
  }
}
