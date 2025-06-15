import { BadRequestException, Injectable } from '@nestjs/common';
import {
  ContextAwareService,
  ServiceContext,
  ServiceOperation,
} from './context.service';
import { Permission, PermissionScope } from '../types';

/**
 * Financial Services - Context-aware financial operations
 */
@Injectable()
export class FinancialService extends ContextAwareService {
  getServiceOperations(): Record<string, ServiceOperation> {
    return {
      deposit: {
        name: 'deposit',
        requiredPermissions: [Permission.FINANCE_DEPOSIT],
        allowedScopes: [
          PermissionScope.ORGANIZATION,
          PermissionScope.CHAMA,
          PermissionScope.PERSONAL,
        ],
        rateLimits: {
          maxOperationsPerDay: 10,
          maxAmountPerOperation: 100000,
        },
      },
      withdraw: {
        name: 'withdraw',
        requiredPermissions: [Permission.FINANCE_WITHDRAW],
        allowedScopes: [
          PermissionScope.ORGANIZATION,
          PermissionScope.CHAMA,
          PermissionScope.PERSONAL,
        ],
        requiresApproval: true,
        approvalRoles: ['treasurer', 'leader'],
        rateLimits: {
          maxOperationsPerDay: 5,
          maxAmountPerOperation: 50000,
        },
      },
      transfer: {
        name: 'transfer',
        requiredPermissions: [Permission.FINANCE_TRANSFER],
        allowedScopes: [
          PermissionScope.ORGANIZATION,
          PermissionScope.CHAMA,
          PermissionScope.PERSONAL,
        ],
        requiresApproval: true,
        rateLimits: {
          maxOperationsPerDay: 5,
          maxAmountPerOperation: 25000,
        },
      },
      viewBalance: {
        name: 'viewBalance',
        requiredPermissions: [Permission.FINANCE_READ],
        allowedScopes: [
          PermissionScope.GLOBAL,
          PermissionScope.ORGANIZATION,
          PermissionScope.CHAMA,
          PermissionScope.PERSONAL,
        ],
      },
      generateStatement: {
        name: 'generateStatement',
        requiredPermissions: [Permission.REPORTS_READ],
        allowedScopes: [
          PermissionScope.ORGANIZATION,
          PermissionScope.CHAMA,
          PermissionScope.PERSONAL,
        ],
      },
    };
  }

  protected async performOperation<T>(
    operationName: string,
    context: ServiceContext,
    operationData: any,
  ): Promise<T> {
    switch (operationName) {
      case 'deposit':
        return this.processDeposit(context, operationData) as Promise<T>;
      case 'withdraw':
        return this.processWithdrawal(context, operationData) as Promise<T>;
      case 'transfer':
        return this.processTransfer(context, operationData) as Promise<T>;
      case 'viewBalance':
        return this.getBalance(context) as Promise<T>;
      case 'generateStatement':
        return this.generateStatement(context, operationData) as Promise<T>;
      default:
        throw new BadRequestException(
          `Unsupported operation: ${operationName}`,
        );
    }
  }

  private async processDeposit(
    context: ServiceContext,
    data: any,
  ): Promise<any> {
    // Implementation for deposit based on context scope
    return {
      transactionId: 'TXN001',
      amount: data.amount,
      status: 'completed',
    };
  }

  private async processWithdrawal(
    context: ServiceContext,
    data: any,
  ): Promise<any> {
    // Implementation for withdrawal based on context scope
    return {
      transactionId: 'TXN002',
      amount: data.amount,
      status: 'pending_approval',
    };
  }

  private async processTransfer(
    context: ServiceContext,
    data: any,
  ): Promise<any> {
    // Implementation for transfer based on context scope
    return {
      transactionId: 'TXN003',
      amount: data.amount,
      status: 'completed',
    };
  }

  private async getBalance(context: ServiceContext): Promise<any> {
    // Return balance based on context scope
    return { balance: 10000, currency: 'KES', scope: context.scope };
  }

  private async generateStatement(
    context: ServiceContext,
    data: any,
  ): Promise<any> {
    // Generate financial statement based on context scope
    return { statementId: 'STMT001', period: data.period, format: 'pdf' };
  }
}
