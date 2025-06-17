import { Injectable, NestMiddleware } from '@nestjs/common';
import { Request, Response, NextFunction } from 'express';
import { AuthService } from '../auth/auth.service';

@Injectable()
export class JwtAuthMiddleware implements NestMiddleware {
  constructor(private authService: AuthService) {}

  async use(req: Request, res: Response, next: NextFunction) {
    const authHeader = req.headers.authorization;

    if (authHeader && authHeader.startsWith('Bearer ')) {
      try {
        const token = authHeader.slice(7);

        // Try to decode as server-generated JWT first
        let memberInfo: any;
        try {
          // Decode JWT token parts to check structure
          const tokenParts = token.split('.');
          if (tokenParts.length === 3) {
            const payload = JSON.parse(
              Buffer.from(tokenParts[1], 'base64url').toString('utf8'),
            );

            // Check if it's a server-generated token (has serviceRole directly)
            if (payload.serviceRole && payload.authMethod) {
              memberInfo = payload;
            } else {
              // It's a Keycloak token, extract member info using auth service
              memberInfo =
                await this.authService.extractMemberInfoFromToken(token);
            }
          }
        } catch (serverTokenError) {
          // If server token decode fails, try Keycloak token extraction
          memberInfo = await this.authService.extractMemberInfoFromToken(token);
        }

        // Attach member info to request
        (req as any).member = {
          ...memberInfo,
          memberId: memberInfo.sub,
          authMethod: memberInfo.authMethod || 'keycloak',
        };
      } catch (error) {
        // Don't throw error, just continue without member info
        // Let the AuthGuard handle authentication requirements
      }
    }

    next();
  }
}
