import { Controller, Get, Post, Put, Body, Param, Query } from '@nestjs/common';
import { ApiTags, ApiOperation, ApiParam } from '@nestjs/swagger';
import {
  CurrentUser,
  OrganizationId,
  ChamaId,
  OrganizationScope,
  ChamaScope,
  PersonalScope,
  MultiScope,
  CrossScope,
  RequiresApproval,
  FinancialOperation,
  AuthenticatedMember,
  Permission,
  PermissionScope,
  Context,
  SaccoService,
  ServiceContext,
  FinancialService,
} from '../common';
import { LoanService } from '../loans';
import { SharesService } from '../shares';

/**
 * SACCO Controller - Context-aware API endpoints
 * Demonstrates multi-scope service operations with proper authorization
 */
@ApiTags('SACCO Operations')
@Controller('sacco')
export class SaccoController {
  constructor(
    private saccoService: SaccoService,
    private financialService: FinancialService,
    private sharesService: SharesService,
    private loanService: LoanService,
  ) {}

  // Financial Operations

  @Get('balance')
  @MultiScope(
    [
      PermissionScope.GLOBAL,
      PermissionScope.ORGANIZATION,
      PermissionScope.CHAMA,
      PermissionScope.PERSONAL,
    ],
    [Permission.FINANCE_READ],
  )
  @ApiOperation({ summary: 'Get balance for current context' })
  async getBalance(
    @Context() context: ServiceContext,
    @CurrentUser() _member: AuthenticatedMember,
  ) {
    return await this.financialService.executeOperation(
      'viewBalance',
      context,
      {},
    );
  }

  @Post('organization/:organizationId/deposit')
  @OrganizationScope([Permission.FINANCE_DEPOSIT])
  @FinancialOperation(100000, 50000)
  @ApiOperation({ summary: 'Deposit to organization account' })
  @ApiParam({ name: 'organizationId', description: 'Organization ID' })
  async depositToOrganization(
    @Context() context: ServiceContext,
    @OrganizationId() organizationId: string,
    @Body()
    depositData: { amount: number; currency: string; description?: string },
  ) {
    return await this.financialService.executeOperation(
      'deposit',
      context,
      depositData,
    );
  }

  @Post('chama/:chamaId/deposit')
  @ChamaScope([Permission.FINANCE_DEPOSIT])
  @FinancialOperation(50000, 25000)
  @ApiOperation({ summary: 'Deposit to chama account' })
  @ApiParam({ name: 'chamaId', description: 'Chama ID' })
  async depositToChama(
    @Context() context: ServiceContext,
    @ChamaId() chamaId: string,
    @Body()
    depositData: { amount: number; currency: string; description?: string },
  ) {
    return await this.financialService.executeOperation(
      'deposit',
      context,
      depositData,
    );
  }

  @Post('organization/:organizationId/withdraw')
  @OrganizationScope([Permission.FINANCE_WITHDRAW])
  @RequiresApproval(['org_admin'], [Permission.FINANCE_APPROVE])
  @FinancialOperation(50000, 10000)
  @ApiOperation({ summary: 'Withdraw from organization account' })
  async withdrawFromOrganization(
    @Context() context: ServiceContext,
    @OrganizationId() organizationId: string,
    @Body() withdrawData: { amount: number; currency: string; reason: string },
  ) {
    return await this.financialService.executeOperation(
      'withdraw',
      context,
      withdrawData,
    );
  }

  @Post('chama/:chamaId/withdraw')
  @ChamaScope([Permission.FINANCE_WITHDRAW])
  @RequiresApproval(['chama_admin'], [Permission.FINANCE_APPROVE])
  @FinancialOperation(25000, 5000)
  @ApiOperation({ summary: 'Withdraw from chama account' })
  async withdrawFromChama(
    @Context() context: ServiceContext,
    @ChamaId() chamaId: string,
    @Body() withdrawData: { amount: number; currency: string; reason: string },
  ) {
    return await this.financialService.executeOperation(
      'withdraw',
      context,
      withdrawData,
    );
  }

  @Post('transfer')
  @CrossScope(PermissionScope.ORGANIZATION, PermissionScope.CHAMA, [
    Permission.FINANCE_TRANSFER,
  ])
  @RequiresApproval(['org_admin', 'chama_admin'], [Permission.FINANCE_APPROVE])
  @ApiOperation({ summary: 'Transfer funds between accounts' })
  async transferFunds(
    @Context() context: ServiceContext,
    @Body()
    transferData: {
      fromAccount: string;
      toAccount: string;
      amount: number;
      currency: string;
      description: string;
    },
  ) {
    return await this.financialService.executeOperation(
      'transfer',
      context,
      transferData,
    );
  }

  // Shares Operations

  @Get('organization/:organizationId/shares')
  @OrganizationScope([Permission.SHARES_READ])
  @ApiOperation({ summary: 'Get organization shares' })
  async getOrganizationShares(
    @Context() context: ServiceContext,
    @OrganizationId() _organizationId: string,
  ) {
    return await this.sharesService.executeOperation('viewShares', context, {});
  }

  @Post('organization/:organizationId/shares/purchase')
  @OrganizationScope([Permission.SHARES_TRADE])
  @ApiOperation({ summary: 'Purchase organization shares' })
  async purchaseShares(
    @Context() context: ServiceContext,
    @OrganizationId() organizationId: string,
    @Body() purchaseData: { quantity: number; price: number },
  ) {
    return await this.sharesService.executeOperation(
      'purchaseShares',
      context,
      purchaseData,
    );
  }

  @Post('organization/:organizationId/shares/sell')
  @OrganizationScope([Permission.SHARES_TRADE])
  @RequiresApproval(['org_admin'], [Permission.SHARES_APPROVE])
  @ApiOperation({ summary: 'Sell organization shares' })
  async sellShares(
    @Context() context: ServiceContext,
    @OrganizationId() organizationId: string,
    @Body() sellData: { quantity: number; price: number },
  ) {
    return await this.sharesService.executeOperation(
      'sellShares',
      context,
      sellData,
    );
  }

  @Post('organization/:organizationId/shares/offer')
  @OrganizationScope([Permission.SHARES_CREATE])
  @RequiresApproval(['org_admin'], [Permission.SHARES_APPROVE])
  @ApiOperation({ summary: 'Create shares offering' })
  async createSharesOffer(
    @Context() context: ServiceContext,
    @OrganizationId() organizationId: string,
    @Body()
    offerData: {
      quantity: number;
      price: number;
      description: string;
      validUntil: Date;
    },
  ) {
    return await this.sharesService.executeOperation(
      'createOffer',
      context,
      offerData,
    );
  }

  // Loan Operations

  @Get('loans')
  @MultiScope(
    [
      PermissionScope.ORGANIZATION,
      PermissionScope.CHAMA,
      PermissionScope.PERSONAL,
    ],
    [Permission.LOAN_READ],
  )
  @ApiOperation({ summary: 'Get loans for current context' })
  async getLoans(
    @Context() context: ServiceContext,
    @Query('status') status?: string,
    @Query('limit') limit?: number,
  ) {
    return await this.loanService.executeOperation('viewLoans', context, {
      status,
      limit,
    });
  }

  @Post('organization/:organizationId/loans/apply')
  @OrganizationScope([Permission.LOAN_APPLY])
  @RequiresApproval(['org_admin'], [Permission.LOAN_APPROVE])
  @ApiOperation({ summary: 'Apply for organization loan' })
  async applyForOrganizationLoan(
    @Context() context: ServiceContext,
    @OrganizationId() organizationId: string,
    @Body()
    loanData: {
      amount: number;
      purpose: string;
      term: number; // in months
      collateral?: string;
    },
  ) {
    return await this.loanService.executeOperation(
      'applyLoan',
      context,
      loanData,
    );
  }

  @Post('chama/:chamaId/loans/apply')
  @ChamaScope([Permission.LOAN_APPLY])
  @RequiresApproval(['chama_admin'], [Permission.LOAN_APPROVE])
  @ApiOperation({ summary: 'Apply for chama loan' })
  async applyForChamaLoan(
    @Context() context: ServiceContext,
    @ChamaId() chamaId: string,
    @Body()
    loanData: {
      amount: number;
      purpose: string;
      term: number; // in months
      guarantors?: string[];
    },
  ) {
    return await this.loanService.executeOperation(
      'applyLoan',
      context,
      loanData,
    );
  }

  @Post('personal/loans/apply')
  @PersonalScope([Permission.LOAN_APPLY])
  @RequiresApproval(['chama_admin', 'org_admin'], [Permission.LOAN_APPROVE])
  @ApiOperation({ summary: 'Apply for personal loan' })
  async applyForPersonalLoan(
    @Context() context: ServiceContext,
    @Body()
    loanData: {
      amount: number;
      purpose: string;
      term: number; // in months
      guarantors: string[];
      collateral?: string;
    },
  ) {
    return await this.loanService.executeOperation(
      'applyLoan',
      context,
      loanData,
    );
  }

  @Put('loans/:loanId/approve')
  @MultiScope(
    [PermissionScope.ORGANIZATION, PermissionScope.CHAMA],
    [Permission.LOAN_APPROVE],
  )
  @ApiOperation({ summary: 'Approve loan application' })
  @ApiParam({ name: 'loanId', description: 'Loan ID' })
  async approveLoan(
    @Context() context: ServiceContext,
    @Param('loanId') loanId: string,
    @Body()
    approvalData: {
      approvedAmount?: number;
      approvedTerm?: number;
      conditions?: string[];
      interestRate?: number;
    },
  ) {
    return await this.loanService.executeOperation('approveLoan', context, {
      loanId,
      ...approvalData,
    });
  }

  @Put('loans/:loanId/disburse')
  @OrganizationScope([Permission.LOAN_DISBURSE])
  @ApiOperation({ summary: 'Disburse approved loan' })
  async disburseLoan(
    @Context() context: ServiceContext,
    @Param('loanId') loanId: string,
    @Body()
    disbursementData: {
      disbursementMethod: 'bank_transfer' | 'mobile_money' | 'cash';
      accountDetails?: string;
    },
  ) {
    return await this.loanService.executeOperation('disburseLoan', context, {
      loanId,
      ...disbursementData,
    });
  }

  @Post('loans/:loanId/repay')
  @MultiScope(
    [
      PermissionScope.ORGANIZATION,
      PermissionScope.CHAMA,
      PermissionScope.PERSONAL,
    ],
    [Permission.FINANCE_DEPOSIT],
  )
  @ApiOperation({ summary: 'Repay loan installment' })
  async repayLoan(
    @Context() context: ServiceContext,
    @Param('loanId') loanId: string,
    @Body()
    repaymentData: {
      amount: number;
      paymentMethod: 'bank_transfer' | 'mobile_money' | 'cash';
    },
  ) {
    return await this.loanService.executeOperation('repayLoan', context, {
      loanId,
      ...repaymentData,
    });
  }

  // Organization and Chama Management

  @Get('organization/:organizationId')
  @OrganizationScope([Permission.ORG_READ])
  @ApiOperation({ summary: 'Get organization details' })
  async getOrganization(@OrganizationId() organizationId: string) {
    return await this.saccoService.getSacco(organizationId);
  }

  @Get('organization/:organizationId/structure')
  @OrganizationScope([Permission.ORG_READ])
  @ApiOperation({
    summary: 'Get organization structure with chamas and members',
  })
  async getOrganizationStructure(@OrganizationId() organizationId: string) {
    return await this.saccoService.getOrganizationStructure(organizationId);
  }

  @Get('chama/:chamaId')
  @ChamaScope([Permission.CHAMA_READ])
  @ApiOperation({ summary: 'Get chama details' })
  async getChama(@ChamaId() chamaId: string) {
    return await this.saccoService.getChama(chamaId);
  }

  @Post('organization/:organizationId/chamas')
  @OrganizationScope([Permission.CHAMA_CREATE])
  @ApiOperation({ summary: 'Create new chama under organization' })
  async createChama(
    @Context() context: ServiceContext,
    @OrganizationId() organizationId: string,
    @Body()
    chamaData: {
      name: string;
      description?: string;
      leaderId: string;
      governance?: any;
    },
  ) {
    return await this.saccoService.createChama({
      ...chamaData,
      parentSACCOId: organizationId,
      chamaType: 'sacco_affiliated',
    });
  }

  @Post('organization/:organizationId/members')
  @OrganizationScope([Permission.USER_INVITE])
  @ApiOperation({ summary: 'Add member to organization' })
  async addOrganizationMember(
    @Context() context: ServiceContext,
    @OrganizationId() organizationId: string,
    @Body()
    memberData: {
      memberId: string;
      role: string;
      customPermissions?: string[];
    },
  ) {
    return await this.saccoService.addOrganizationMember(
      organizationId,
      memberData.memberId,
      memberData.role as any,
      context.memberId,
      memberData.customPermissions as any,
    );
  }

  @Post('chama/:chamaId/members')
  @ChamaScope([Permission.CHAMA_INVITE])
  @ApiOperation({ summary: 'Add member to chama' })
  async addChamaMember(
    @Context() context: ServiceContext,
    @ChamaId() chamaId: string,
    @Body()
    memberData: {
      memberId: string;
      role: string;
      customPermissions?: string[];
    },
  ) {
    return await this.saccoService.addChamaMember(
      chamaId,
      memberData.memberId,
      memberData.role as any,
      context.memberId,
      memberData.customPermissions as any,
    );
  }

  // Reports and Analytics

  @Get('organization/:organizationId/reports/financial')
  @OrganizationScope([Permission.REPORTS_READ])
  @ApiOperation({ summary: 'Get organization financial report' })
  async getOrganizationFinancialReport(
    @Context() context: ServiceContext,
    @OrganizationId() organizationId: string,
    @Query('period') period: string = 'monthly',
    @Query('format') format: string = 'json',
  ) {
    return await this.financialService.executeOperation(
      'generateStatement',
      context,
      { period, format, type: 'financial' },
    );
  }

  @Get('chama/:chamaId/reports/contributions')
  @ChamaScope([Permission.REPORTS_READ])
  @ApiOperation({ summary: 'Get chama contribution report' })
  async getChamaContributionReport(
    @Context() context: ServiceContext,
    @ChamaId() chamaId: string,
    @Query('period') period: string = 'monthly',
  ) {
    return await this.financialService.executeOperation(
      'generateStatement',
      context,
      { period, type: 'contributions' },
    );
  }
}
