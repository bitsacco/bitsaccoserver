import { Model } from 'mongoose';
import {
  Injectable,
  CanActivate,
  ExecutionContext,
  ForbiddenException,
  Logger,
} from '@nestjs/common';
import { Reflector } from '@nestjs/core';
import { InjectModel } from '@nestjs/mongoose';
import {
  OrganizationMember,
  OrganizationMemberDocument,
} from '../schemas/organization.schema';
import { ROLES_KEY } from '../roles.decorator';
import { UserRole } from '../types';

@Injectable()
export class RBACGuard implements CanActivate {
  private readonly logger = new Logger(RBACGuard.name);

  constructor(
    private reflector: Reflector,
    @InjectModel(OrganizationMember.name)
    private organizationMemberModel: Model<OrganizationMemberDocument>,
  ) {}

  async canActivate(context: ExecutionContext): Promise<boolean> {
    const requiredRoles = this.reflector.getAllAndOverride<UserRole[]>(
      ROLES_KEY,
      [context.getHandler(), context.getClass()],
    );

    if (!requiredRoles) {
      return true;
    }

    const request = context.switchToHttp().getRequest();
    const user = request.user;
    const organizationId =
      request.params.organizationId ||
      request.params.id ||
      request.body.organizationId;

    if (!user || !organizationId) {
      this.logger.debug(
        JSON.stringify({
          hasUser: !!user,
          userId: user?.sub,
          organizationId,
          params: request.params,
          body: request.body,
        }),
      );
      throw new ForbiddenException('User or organization not found');
    }

    // Convert organizationId to string to ensure consistent type matching
    const orgIdString = organizationId.toString();

    const membership = await this.organizationMemberModel.findOne({
      userId: user.sub,
      organizationId: orgIdString,
      isActive: true,
    });

    if (!membership) {
      this.logger.debug(
        JSON.stringify({
          userId: user.sub,
          organizationId: orgIdString,
          originalOrgId: organizationId,
          membershipFound: !!membership,
          query: {
            userId: user.sub,
            organizationId: orgIdString,
            isActive: true,
          },
        }),
      );
      throw new ForbiddenException('User is not a member of this organization');
    }

    const hasRole = this.checkRolePermissions(membership.role, requiredRoles);

    if (!hasRole) {
      throw new ForbiddenException('Insufficient permissions');
    }

    // Add organization context to request
    request.organizationMembership = membership;
    return true;
  }

  private checkRolePermissions(
    userRole: UserRole,
    requiredRoles: UserRole[],
  ): boolean {
    // Admin has all permissions
    if (userRole === UserRole.ADMIN) {
      return true;
    }

    // Check if user has any of the required roles
    return requiredRoles.includes(userRole);
  }
}
