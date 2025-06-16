import {
  Injectable,
  BadRequestException,
  ForbiddenException,
} from '@nestjs/common';
import { PermissionScope, AuthenticatedUser } from '../types';
import {
  ServiceContext,
  ServiceOperation,
  ServiceResult,
} from '../types/audit.types';
import { PermissionService } from './permission.service';
import { SaccoService } from './sacco.service';

// Re-export types for services that import from this module
export {
  ServiceContext,
  ServiceOperation,
  ServiceResult,
} from '../types/audit.types';

/**
 * Context-aware service framework for SACCO operations
 * Handles service delivery across multiple scopes: global, organization, chama, personal
 */

// Interfaces moved to ../types/audit.types.ts

/**
 * Base class for context-aware services
 */
@Injectable()
export abstract class ContextAwareService {
  constructor(
    protected permissionService: PermissionService,
    protected saccoService: SaccoService,
  ) {}

  /**
   * Abstract method to define service operations
   */
  abstract getServiceOperations(): Record<string, ServiceOperation>;

  /**
   * Create service context from authenticated user and request parameters
   */
  async createServiceContext(
    user: AuthenticatedUser,
    organizationId?: string,
    chamaId?: string,
  ): Promise<ServiceContext> {
    // Determine scope based on provided IDs
    let scope = PermissionScope.GLOBAL;
    if (chamaId) {
      scope = PermissionScope.CHAMA;
    } else if (organizationId) {
      scope = PermissionScope.ORGANIZATION;
    }

    // Resolve permissions for the context
    const permissions = this.permissionService.resolveUserPermissions(
      user,
      scope,
      organizationId,
      chamaId,
    );

    return {
      userId: user.userId,
      scope,
      organizationId,
      chamaId,
      permissions,
      user,
    };
  }

  /**
   * Validate service operation in given context
   */
  async validateOperation(
    operationName: string,
    context: ServiceContext,
  ): Promise<void> {
    const operations = this.getServiceOperations();
    const operation = operations[operationName];

    if (!operation) {
      throw new BadRequestException(`Unknown operation: ${operationName}`);
    }

    // Check if operation is allowed in current scope
    if (!operation.allowedScopes.includes(context.scope)) {
      throw new ForbiddenException(
        `Operation ${operationName} not allowed in ${context.scope} scope`,
      );
    }

    // Check permissions
    const hasPermissions = this.permissionService.userHasAllPermissions(
      context.user,
      operation.requiredPermissions,
      context.scope,
      context.organizationId,
      context.chamaId,
    );

    if (!hasPermissions) {
      throw new ForbiddenException(
        `Insufficient permissions for operation: ${operationName}`,
      );
    }
  }

  /**
   * Execute service operation with context validation
   */
  async executeOperation<T>(
    operationName: string,
    context: ServiceContext,
    operationData: any,
  ): Promise<ServiceResult<T>> {
    try {
      // Validate operation
      await this.validateOperation(operationName, context);

      // Execute the operation
      const result = await this.performOperation<T>(
        operationName,
        context,
        operationData,
      );

      return {
        success: true,
        data: result,
        context,
      };
    } catch (error) {
      return {
        success: false,
        error: error.message,
        context,
      };
    }
  }

  /**
   * Abstract method for service-specific operation implementation
   */
  protected abstract performOperation<T>(
    operationName: string,
    context: ServiceContext,
    operationData: any,
  ): Promise<T>;

  /**
   * Check if operation requires approval workflow
   */
  protected requiresApproval(
    operationName: string,
    _context: ServiceContext,
    _operationData: any,
  ): boolean {
    const operations = this.getServiceOperations();
    const operation = operations[operationName];
    return operation.requiresApproval || false;
  }

  /**
   * Cross-scope operation validation
   * For operations that span multiple organizations/chamas
   */
  async validateCrossScope(
    sourceContext: ServiceContext,
    targetContext: ServiceContext,
    operation: string,
  ): Promise<boolean> {
    // Both contexts must have required permissions
    await this.validateOperation(operation, sourceContext);
    await this.validateOperation(operation, targetContext);

    // Additional cross-scope validation logic
    if (
      sourceContext.scope === PermissionScope.CHAMA &&
      targetContext.scope === PermissionScope.CHAMA
    ) {
      // Chama-to-chama operations need additional validation
      return this.validateChamaToChama(sourceContext, targetContext);
    }

    if (
      sourceContext.scope === PermissionScope.ORGANIZATION &&
      targetContext.scope === PermissionScope.CHAMA
    ) {
      // Organization-to-chama operations
      return this.validateOrganizationToChama(sourceContext, targetContext);
    }

    return true;
  }

  /**
   * Validate chama-to-chama operations
   */
  private async validateChamaToChama(
    _sourceContext: ServiceContext,
    _targetContext: ServiceContext,
  ): Promise<boolean> {
    // Check if chamas are related or have partnership agreements
    // This would query the GroupRelationship collection
    return true; // Simplified for now
  }

  /**
   * Validate organization-to-chama operations
   */
  private async validateOrganizationToChama(
    sourceContext: ServiceContext,
    targetContext: ServiceContext,
  ): Promise<boolean> {
    // Check if chama belongs to the organization
    const chama = await this.saccoService.getChama(targetContext.chamaId!);
    return chama.parentSACCOId === sourceContext.organizationId;
  }
}
