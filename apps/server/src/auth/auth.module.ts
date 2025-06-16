import { Module } from '@nestjs/common';
import { JwtModule } from '@nestjs/jwt';
import { PassportModule } from '@nestjs/passport';
import { MongooseModule } from '@nestjs/mongoose';
import { HttpModule } from '@nestjs/axios';
import { ConfigModule, ConfigService } from '@nestjs/config';
import { KeycloakConnectModule } from 'nest-keycloak-connect';

import { AuthController } from './auth.controller';
import { AuthService } from './auth.service';
import { JwtStrategy } from './jwt.strategy';
import { Member, MemberSchema } from './member.schema';
import {
  ServiceDocument,
  ServiceSchema,
  RateLimitInterceptor,
  CommonModule,
} from '@/common';

@Module({
  imports: [
    CommonModule,
    PassportModule,
    HttpModule.register({
      timeout: 10000,
      maxRedirects: 5,
    }),
    MongooseModule.forFeature([
      { name: Member.name, schema: MemberSchema },
      { name: ServiceDocument.name, schema: ServiceSchema },
    ]),

    // JWT Module
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

    // Keycloak Module
    KeycloakConnectModule.registerAsync({
      imports: [ConfigModule],
      inject: [ConfigService],
      useFactory: (configService: ConfigService) => ({
        authServerUrl:
          configService.get<string>('KEYCLOAK_AUTH_SERVER_URL') ||
          'http://localhost:8080',
        realm: configService.get<string>('KEYCLOAK_REALM') || 'bitsaccoserver',
        clientId: configService.get<string>('KEYCLOAK_CLIENT_ID') || '',
        secret: configService.get<string>('KEYCLOAK_CLIENT_SECRET'),
        cookieKey: 'KEYCLOAK_JWT',
        logLevels: ['verbose'],
      }),
    }),
  ],
  controllers: [AuthController],
  providers: [AuthService, JwtStrategy, RateLimitInterceptor],
  exports: [AuthService, RateLimitInterceptor, MongooseModule, JwtModule],
})
export class AuthModule {}
