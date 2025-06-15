import { BadRequestException, Injectable } from '@nestjs/common';
import {
  ContextAwareService,
  Permission,
  PermissionScope,
  ServiceContext,
  ServiceOperation,
} from '../common';

/**
 * Loan Service - Context-aware loan management
 */
@Injectable()
export class LoanService extends ContextAwareService {
  getServiceOperations(): Record<string, ServiceOperation> {
    return {
      applyLoan: {
        name: 'applyLoan',
        requiredPermissions: [Permission.LOAN_APPLY],
        allowedScopes: [
          PermissionScope.ORGANIZATION,
          PermissionScope.CHAMA,
          PermissionScope.PERSONAL,
        ],
        requiresApproval: true,
        rateLimits: {
          maxOperationsPerDay: 1,
        },
      },
      approveLoan: {
        name: 'approveLoan',
        requiredPermissions: [Permission.LOAN_APPROVE],
        allowedScopes: [PermissionScope.ORGANIZATION, PermissionScope.CHAMA],
      },
      disburseLoan: {
        name: 'disburseLoan',
        requiredPermissions: [Permission.LOAN_DISBURSE],
        allowedScopes: [PermissionScope.ORGANIZATION],
      },
      viewLoans: {
        name: 'viewLoans',
        requiredPermissions: [Permission.LOAN_READ],
        allowedScopes: [
          PermissionScope.GLOBAL,
          PermissionScope.ORGANIZATION,
          PermissionScope.CHAMA,
          PermissionScope.PERSONAL,
        ],
      },
      repayLoan: {
        name: 'repayLoan',
        requiredPermissions: [Permission.FINANCE_DEPOSIT],
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
      case 'applyLoan':
        return this.applyForLoan(context, operationData) as Promise<T>;
      case 'approveLoan':
        return this.approveLoan(context, operationData) as Promise<T>;
      case 'disburseLoan':
        return this.disburseLoan(context, operationData) as Promise<T>;
      case 'viewLoans':
        return this.getLoans(context) as Promise<T>;
      case 'repayLoan':
        return this.repayLoan(context, operationData) as Promise<T>;
      default:
        throw new BadRequestException(
          `Unsupported loan operation: ${operationName}`,
        );
    }
  }

  private async applyForLoan(context: ServiceContext, data: any): Promise<any> {
    return {
      applicationId: 'LOAN001',
      amount: data.amount,
      status: 'pending_approval',
      scope: context.scope,
    };
  }

  private async approveLoan(context: ServiceContext, data: any): Promise<any> {
    return {
      loanId: data.loanId,
      status: 'approved',
      approvedBy: context.userId,
    };
  }

  private async disburseLoan(context: ServiceContext, data: any): Promise<any> {
    return { loanId: data.loanId, status: 'disbursed', amount: data.amount };
  }

  private async getLoans(context: ServiceContext): Promise<any> {
    return { loans: [], totalOutstanding: 0, scope: context.scope };
  }

  private async repayLoan(context: ServiceContext, data: any): Promise<any> {
    return {
      loanId: data.loanId,
      paymentAmount: data.amount,
      remainingBalance: 5000,
    };
  }
}
