import {
  Controller,
  Get,
  Post,
  Body,
  Patch,
  Param,
  Delete,
  Query,
  Put,
} from '@nestjs/common';
import {
  ApiTags,
  ApiOperation,
  ApiResponse,
  ApiBearerAuth,
  ApiBody,
  ApiParam,
} from '@nestjs/swagger';
import { GroupRole } from '@bitsaccoserver/types';
import {
  ApiKeyService,
  CreateApiKeyDto,
  CurrentUser,
  OrganizationId,
  OrganizationScope,
  GlobalScope,
  MultiScope,
  RequiresApproval,
  FinancialOperation,
  AuthenticatedMember,
  Permission,
  PermissionScope,
  Context,
  ServiceContext,
  FinancialService,
  AuditService,
  PermissionService,
  RiskLevel,
} from '../common';
import { OrganizationService } from './organization.service';
import {
  AddMemberDto,
  CreateOrganizationDto,
  UpdateOrganizationDto,
} from './organization.dto';
import { SharesService } from '../shares';

@ApiTags('organizations')
@ApiBearerAuth()
@Controller('organizations')
export class OrganizationController {
  constructor(
    private readonly organizationService: OrganizationService,
    private readonly apiKeyService: ApiKeyService,
    private readonly financialService: FinancialService,
    private readonly auditService: AuditService,
    private readonly permissionService: PermissionService,
    private readonly sharesService: SharesService,
  ) {}

  @Post()
  @GlobalScope([Permission.ORG_CREATE])
  @ApiOperation({ summary: 'Create a new organization' })
  @ApiResponse({
    status: 201,
    description: 'Organization created successfully',
  })
  async create(
    @Body() createOrganizationDto: CreateOrganizationDto,
    @CurrentUser() member: AuthenticatedMember,
    @Context() context: ServiceContext,
  ) {
    const organization = await this.organizationService.create(
      createOrganizationDto,
      member.memberId,
      member.email,
    );

    await this.auditService.logAuditEvent({
      action: 'ORGANIZATION_CREATED',
      memberId: member.memberId,
      organizationId: (organization as any)._id.toString(),
      resourceType: 'organization',
      resourceId: (organization as any)._id.toString(),
      scope: context.scope,
      timestamp: new Date(),
      businessContext: {
        businessJustification: `Created organization: ${organization.name}`,
      },
    });

    return organization;
  }

  @Get()
  @GlobalScope([Permission.ORG_READ])
  @ApiOperation({ summary: 'Get all organizations for current member' })
  @ApiResponse({
    status: 200,
    description: 'Organizations retrieved successfully',
  })
  async findAll(@CurrentUser() member: AuthenticatedMember) {
    return this.organizationService.findAll(member.memberId);
  }

  @Get(':organizationId')
  @OrganizationScope([Permission.ORG_READ])
  @ApiOperation({ summary: 'Get organization by ID' })
  @ApiParam({ name: 'organizationId', description: 'Organization ID' })
  @ApiResponse({
    status: 200,
    description: 'Organization retrieved successfully',
  })
  async findOne(
    @OrganizationId() organizationId: string,
    @Context() context: ServiceContext,
  ) {
    const organization = await this.organizationService.findOne(organizationId);

    // Get organization structure with financial summary
    const [members, financialSummary] = await Promise.all([
      this.organizationService.getMembers(organizationId),
      this.financialService.executeOperation('getBalance', context, {}),
    ]);

    return {
      ...organization,
      memberCount: members.length,
      financialSummary,
    };
  }

  @Patch(':organizationId')
  @OrganizationScope([Permission.ORG_UPDATE])
  @RequiresApproval(['org_admin'], [Permission.ORG_APPROVE])
  @ApiOperation({ summary: 'Update organization' })
  @ApiParam({ name: 'organizationId', description: 'Organization ID' })
  @ApiResponse({
    status: 200,
    description: 'Organization updated successfully',
  })
  async update(
    @OrganizationId() organizationId: string,
    @Body() updateOrganizationDto: UpdateOrganizationDto,
    @CurrentUser() member: AuthenticatedMember,
    @Context() context: ServiceContext,
  ) {
    const organization = await this.organizationService.update(
      organizationId,
      updateOrganizationDto,
    );

    await this.auditService.logAuditEvent({
      action: 'ORGANIZATION_UPDATED',
      memberId: member.memberId,
      organizationId,
      resourceType: 'organization',
      resourceId: organizationId,
      scope: context.scope,
      timestamp: new Date(),
      businessContext: {
        businessJustification: 'Updated organization details',
      },
      requestData: {
        method: 'PATCH',
        endpoint: `/organizations/${organizationId}`,
        body: updateOrganizationDto,
        memberAgent: 'web-app',
        ipAddress: 'unknown',
      },
    });

    return organization;
  }

  @Delete(':organizationId')
  @OrganizationScope([Permission.ORG_DELETE])
  @RequiresApproval(['org_admin'], [Permission.ORG_APPROVE])
  @ApiOperation({ summary: 'Delete organization' })
  @ApiParam({ name: 'organizationId', description: 'Organization ID' })
  @ApiResponse({
    status: 200,
    description: 'Organization deleted successfully',
  })
  async remove(
    @OrganizationId() organizationId: string,
    @CurrentUser() member: AuthenticatedMember,
    @Context() context: ServiceContext,
  ) {
    await this.organizationService.delete(organizationId);

    await this.auditService.logAuditEvent({
      action: 'ORGANIZATION_DELETED',
      memberId: member.memberId,
      organizationId,
      resourceType: 'organization',
      resourceId: organizationId,
      scope: context.scope,
      timestamp: new Date(),
      complianceContext: {
        riskLevel: RiskLevel.HIGH,
        sensitiveData: false,
        approvalRequired: true,
      },
    });

    return { success: true, message: 'Organization deleted successfully' };
  }

  @Get(':organizationId/members')
  @OrganizationScope([Permission.MEMBER_READ])
  @ApiOperation({ summary: 'Get organization members' })
  @ApiParam({ name: 'organizationId', description: 'Organization ID' })
  @ApiResponse({ status: 200, description: 'Members retrieved successfully' })
  async getMembers(
    @OrganizationId() organizationId: string,
    @Query('includeInactive') includeInactive?: boolean,
    @Query('role') role?: string,
  ) {
    const members = await this.organizationService.getMembers(
      organizationId,
      includeInactive,
      role as GroupRole,
    );

    // Enrich with member details and permissions
    const enrichedMembers = await Promise.all(
      members.map(async (member) => {
        const permissions = []; // TODO: Implement member permissions retrieval
        return {
          ...member.toObject(),
          permissions,
        };
      }),
    );

    return enrichedMembers;
  }

  @Post(':organizationId/members')
  @OrganizationScope([Permission.MEMBER_INVITE])
  @RequiresApproval(['org_admin'], [Permission.MEMBER_APPROVE])
  @ApiOperation({ summary: 'Add member to organization' })
  @ApiParam({ name: 'organizationId', description: 'Organization ID' })
  @ApiBody({
    type: AddMemberDto,
    description: 'Member details to add to the organization',
    examples: {
      developer: {
        summary: 'Add developer member',
        description: 'Example of adding a developer to the organization',
        value: {
          memberId: 'member-123-abc',
          role: 'developer',
          customPermissions: ['FINANCE_READ'],
        },
      },
      admin: {
        summary: 'Add admin member',
        description: 'Example of adding an admin to the organization',
        value: {
          memberId: 'member-456-def',
          role: 'admin',
        },
      },
    },
  })
  @ApiResponse({
    status: 201,
    description: 'Member added successfully',
  })
  async addMember(
    @OrganizationId() organizationId: string,
    @Body() addMemberDto: AddMemberDto,
    @CurrentUser() member: AuthenticatedMember,
    @Context() context: ServiceContext,
  ) {
    const newMember = await this.organizationService.addMember(
      organizationId,
      addMemberDto.memberId,
      addMemberDto.role,
      member.memberId,
      addMemberDto.customPermissions,
    );

    await this.auditService.logAuditEvent({
      action: 'MEMBER_ADDED',
      memberId: member.memberId,
      organizationId,
      resourceType: 'member',
      resourceId: addMemberDto.memberId,
      scope: context.scope,
      timestamp: new Date(),
      businessContext: {
        businessJustification: `Added member ${addMemberDto.memberId} with role ${addMemberDto.role}`,
      },
    });

    return newMember;
  }

  // Member Management Endpoints
  @Put(':organizationId/members/:memberId/role')
  @OrganizationScope([Permission.MEMBER_UPDATE])
  @RequiresApproval(['org_admin'], [Permission.MEMBER_APPROVE])
  @ApiOperation({ summary: 'Update member role' })
  @ApiParam({ name: 'organizationId', description: 'Organization ID' })
  @ApiParam({ name: 'memberId', description: 'Member ID' })
  async updateMemberRole(
    @OrganizationId() organizationId: string,
    @Param('memberId') targetMemberId: string,
    @Body() roleData: { role: GroupRole; customPermissions?: Permission[] },
    @CurrentUser() member: AuthenticatedMember,
    @Context() context: ServiceContext,
  ) {
    const updatedMember = await this.organizationService.updateMemberRole(
      organizationId,
      targetMemberId,
      roleData.role,
      roleData.customPermissions,
    );

    await this.auditService.logAuditEvent({
      action: 'MEMBER_ROLE_UPDATED',
      memberId: member.memberId,
      organizationId,
      resourceType: 'member',
      resourceId: targetMemberId,
      scope: context.scope,
      timestamp: new Date(),
      businessContext: {
        businessJustification: `Updated member ${targetMemberId} role to ${roleData.role}`,
      },
    });

    return updatedMember;
  }

  @Delete(':organizationId/members/:memberId')
  @OrganizationScope([Permission.MEMBER_REMOVE])
  @RequiresApproval(['org_admin'], [Permission.MEMBER_APPROVE])
  @ApiOperation({ summary: 'Remove member from organization' })
  @ApiParam({ name: 'organizationId', description: 'Organization ID' })
  @ApiParam({ name: 'memberId', description: 'Member ID' })
  async removeMember(
    @OrganizationId() organizationId: string,
    @Param('memberId') targetMemberId: string,
    @CurrentUser() member: AuthenticatedMember,
    @Context() context: ServiceContext,
  ) {
    await this.organizationService.removeMember(organizationId, targetMemberId);

    await this.auditService.logAuditEvent({
      action: 'MEMBER_REMOVED',
      memberId: member.memberId,
      organizationId,
      resourceType: 'member',
      resourceId: targetMemberId,
      scope: context.scope,
      timestamp: new Date(),
      complianceContext: {
        riskLevel: RiskLevel.MEDIUM,
        sensitiveData: false,
        approvalRequired: true,
      },
    });

    return { success: true, message: 'Member removed successfully' };
  }

  // Financial Operations
  @Get(':organizationId/balance')
  @OrganizationScope([Permission.FINANCE_READ])
  @ApiOperation({ summary: 'Get organization balance' })
  @ApiParam({ name: 'organizationId', description: 'Organization ID' })
  async getBalance(
    @Context() context: ServiceContext,
    @CurrentUser() member: AuthenticatedMember,
  ) {
    return await this.financialService.executeOperation(
      'viewBalance',
      context,
      {},
    );
  }

  @Post(':organizationId/deposit')
  @OrganizationScope([Permission.FINANCE_DEPOSIT])
  @FinancialOperation(1000000, 100000)
  @ApiOperation({ summary: 'Deposit to organization account' })
  @ApiParam({ name: 'organizationId', description: 'Organization ID' })
  async deposit(
    @Context() context: ServiceContext,
    @OrganizationId() organizationId: string,
    @Body()
    depositData: { amount: number; currency: string; description?: string },
    @CurrentUser() member: AuthenticatedMember,
  ) {
    const result = await this.financialService.executeOperation(
      'deposit',
      context,
      depositData,
    );

    await this.auditService.logAuditEvent({
      action: 'DEPOSIT',
      memberId: member.memberId,
      organizationId,
      resourceType: 'financial_transaction',
      scope: context.scope,
      timestamp: new Date(),
      businessContext: {
        amount: depositData.amount,
        currency: depositData.currency,
        transactionType: 'deposit',
        businessJustification:
          depositData.description || 'Organization deposit',
      },
      complianceContext: {
        riskLevel: RiskLevel.MEDIUM,
        sensitiveData: true,
      },
    });

    return result;
  }

  @Post(':organizationId/withdraw')
  @OrganizationScope([Permission.FINANCE_WITHDRAW])
  @RequiresApproval(['org_admin'], [Permission.FINANCE_APPROVE])
  @FinancialOperation(500000, 50000)
  @ApiOperation({ summary: 'Withdraw from organization account' })
  @ApiParam({ name: 'organizationId', description: 'Organization ID' })
  async withdraw(
    @Context() context: ServiceContext,
    @OrganizationId() organizationId: string,
    @Body() withdrawData: { amount: number; currency: string; reason: string },
    @CurrentUser() member: AuthenticatedMember,
  ) {
    const result = await this.financialService.executeOperation(
      'withdraw',
      context,
      withdrawData,
    );

    await this.auditService.logAuditEvent({
      action: 'WITHDRAWAL',
      memberId: member.memberId,
      organizationId,
      resourceType: 'financial_transaction',
      scope: context.scope,
      timestamp: new Date(),
      businessContext: {
        amount: withdrawData.amount,
        currency: withdrawData.currency,
        transactionType: 'withdrawal',
        businessJustification: withdrawData.reason,
        approvalRequired: true,
      },
      complianceContext: {
        riskLevel: RiskLevel.HIGH,
        sensitiveData: true,
        approvalRequired: true,
      },
    });

    return result;
  }

  // Shares Operations
  @Get(':organizationId/shares')
  @OrganizationScope([Permission.SHARES_READ])
  @ApiOperation({ summary: 'Get organization shares' })
  @ApiParam({ name: 'organizationId', description: 'Organization ID' })
  async getShares(
    @Context() context: ServiceContext,
    @OrganizationId() organizationId: string,
  ) {
    return await this.sharesService.executeOperation('viewShares', context, {});
  }

  @Post(':organizationId/shares/offer')
  @OrganizationScope([Permission.SHARES_CREATE])
  @RequiresApproval(['org_admin'], [Permission.SHARES_APPROVE])
  @ApiOperation({ summary: 'Create shares offering' })
  @ApiParam({ name: 'organizationId', description: 'Organization ID' })
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
    @CurrentUser() member: AuthenticatedMember,
  ) {
    const result = await this.sharesService.executeOperation(
      'createOffer',
      context,
      offerData,
    );

    await this.auditService.logAuditEvent({
      action: 'SHARES_OFFER_CREATED',
      memberId: member.memberId,
      organizationId,
      resourceType: 'shares_offer',
      scope: context.scope,
      timestamp: new Date(),
      businessContext: {
        amount: offerData.quantity * offerData.price,
        transactionType: 'shares_offer',
        businessJustification: offerData.description,
        approvalRequired: true,
      },
      complianceContext: {
        riskLevel: RiskLevel.HIGH,
        sensitiveData: false,
        approvalRequired: true,
      },
    });

    return result;
  }

  // API Key Management Endpoints
  @Post(':organizationId/api-keys')
  @OrganizationScope([Permission.API_KEY_CREATE])
  @ApiOperation({ summary: 'Create a new API key for organization' })
  @ApiParam({ name: 'organizationId', description: 'Organization ID' })
  @ApiResponse({ status: 201, description: 'API key created successfully' })
  async createApiKey(
    @OrganizationId() organizationId: string,
    @Body() createApiKeyDto: CreateApiKeyDto,
    @CurrentUser() member: AuthenticatedMember,
    @Context() context: ServiceContext,
  ) {
    const apiKey = await this.apiKeyService.create(
      organizationId,
      member.memberId,
      createApiKeyDto,
    );

    await this.auditService.logAuditEvent({
      action: 'API_KEY_CREATED',
      memberId: member.memberId,
      organizationId,
      resourceType: 'api_key',
      scope: context.scope,
      timestamp: new Date(),
      businessContext: {
        businessJustification: `Created API key: ${createApiKeyDto.name}`,
      },
      complianceContext: {
        riskLevel: RiskLevel.MEDIUM,
        sensitiveData: true,
      },
    });

    return apiKey;
  }

  @Get(':organizationId/api-keys')
  @OrganizationScope([Permission.API_KEY_READ])
  @ApiOperation({ summary: 'List organization API keys' })
  @ApiParam({ name: 'organizationId', description: 'Organization ID' })
  @ApiResponse({ status: 200, description: 'API keys retrieved successfully' })
  async getApiKeys(@OrganizationId() organizationId: string) {
    return this.apiKeyService.findAll(organizationId);
  }

  @Get(':organizationId/api-keys/:keyId')
  @OrganizationScope([Permission.API_KEY_READ])
  @ApiOperation({ summary: 'Get API key details' })
  @ApiParam({ name: 'organizationId', description: 'Organization ID' })
  @ApiParam({ name: 'keyId', description: 'API Key ID' })
  @ApiResponse({ status: 200, description: 'API key retrieved successfully' })
  async getApiKey(
    @OrganizationId() organizationId: string,
    @Param('keyId') keyId: string,
  ) {
    return this.apiKeyService.findOne(organizationId, keyId);
  }

  @Delete(':organizationId/api-keys/:keyId')
  @OrganizationScope([Permission.API_KEY_DELETE])
  @ApiOperation({ summary: 'Delete/revoke API key' })
  @ApiParam({ name: 'organizationId', description: 'Organization ID' })
  @ApiParam({ name: 'keyId', description: 'API Key ID' })
  @ApiResponse({ status: 200, description: 'API key revoked successfully' })
  async deleteApiKey(
    @OrganizationId() organizationId: string,
    @Param('keyId') keyId: string,
    @CurrentUser() member: AuthenticatedMember,
    @Context() context: ServiceContext,
  ) {
    await this.apiKeyService.remove(organizationId, keyId);

    await this.auditService.logAuditEvent({
      action: 'API_KEY_DELETED',
      memberId: member.memberId,
      organizationId,
      resourceType: 'api_key',
      resourceId: keyId,
      scope: context.scope,
      timestamp: new Date(),
      businessContext: {
        businessJustification: `Deleted API key: ${keyId}`,
      },
      complianceContext: {
        riskLevel: RiskLevel.MEDIUM,
        sensitiveData: true,
      },
    });

    return { success: true, message: 'API key revoked successfully' };
  }

  @Get(':organizationId/api-keys/:keyId/usage')
  @OrganizationScope([Permission.API_KEY_READ])
  @ApiOperation({ summary: 'Get API key usage statistics' })
  @ApiParam({ name: 'organizationId', description: 'Organization ID' })
  @ApiParam({ name: 'keyId', description: 'API Key ID' })
  @ApiResponse({
    status: 200,
    description: 'Usage statistics retrieved successfully',
  })
  async getApiKeyUsage(
    @OrganizationId() organizationId: string,
    @Param('keyId') keyId: string,
  ) {
    return this.apiKeyService.getUsage(organizationId, keyId);
  }

  // Reporting and Analytics
  @Get(':organizationId/reports/financial')
  @OrganizationScope([Permission.REPORTS_READ])
  @ApiOperation({ summary: 'Get organization financial report' })
  @ApiParam({ name: 'organizationId', description: 'Organization ID' })
  async getFinancialReport(
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

  @Get(':organizationId/reports/members')
  @OrganizationScope([Permission.REPORTS_READ])
  @ApiOperation({ summary: 'Get organization member activity report' })
  @ApiParam({ name: 'organizationId', description: 'Organization ID' })
  async getMemberReport(
    @OrganizationId() organizationId: string,
    @Query('period') period: string = 'monthly',
  ) {
    const [members, auditLogs] = await Promise.all([
      this.organizationService.getMembers(organizationId),
      this.auditService.getAuditLogs({ organizationId, limit: 100 }),
    ]);

    const recentActivity = auditLogs.logs || [];

    return {
      organizationId,
      period,
      totalMembers: members.length,
      activeMembers: members.filter((m) => m.isActive).length,
      recentActivity,
      membersByRole: members.reduce(
        (acc, member) => {
          acc[member.role] = (acc[member.role] || 0) + 1;
          return acc;
        },
        {} as Record<string, number>,
      ),
    };
  }

  // Services, Usage & Billing Endpoints
  @Get(':organizationId/services')
  @OrganizationScope([Permission.ORG_READ])
  @ApiOperation({ summary: 'Get available services for organization' })
  @ApiParam({ name: 'organizationId', description: 'Organization ID' })
  @ApiResponse({ status: 200, description: 'Services retrieved successfully' })
  async getOrganizationServices(@OrganizationId() organizationId: string) {
    // Get enabled services for the organization
    const organization = await this.organizationService.findOne(organizationId);
    return {
      organizationId,
      services: [], // TODO: Implement enabled services in organization schema
      limits: organization.limits,
    };
  }

  @Get(':organizationId/usage')
  @OrganizationScope([Permission.REPORTS_READ])
  @ApiOperation({ summary: 'Get organization usage statistics' })
  @ApiParam({ name: 'organizationId', description: 'Organization ID' })
  @ApiResponse({
    status: 200,
    description: 'Usage statistics retrieved successfully',
  })
  async getUsageStats(
    @OrganizationId() organizationId: string,
    @Query('period') period: string = 'current_month',
    @Query('service') service?: string,
    @Query('apiKeyId') apiKeyId?: string,
  ) {
    const [auditStats] = await Promise.all([
      this.auditService.getAuditStatistics(
        PermissionScope.ORGANIZATION,
        organizationId,
      ),
    ]);

    // TODO: Implement organization usage in ApiKeyService
    const apiKeyUsage = {
      totalRequests: 0,
      successfulRequests: 0,
      failedRequests: 0,
      totalCost: 0,
    };

    return {
      organizationId,
      period,
      service,
      apiKeyId,
      statistics: {
        ...apiKeyUsage,
        ...auditStats,
      },
    };
  }

  @Get(':organizationId/billing')
  @OrganizationScope([Permission.FINANCE_READ])
  @ApiOperation({ summary: 'Get organization billing information' })
  @ApiParam({ name: 'organizationId', description: 'Organization ID' })
  @ApiResponse({
    status: 200,
    description: 'Billing information retrieved successfully',
  })
  async getBillingInfo(
    @OrganizationId() organizationId: string,
    @Context() context: ServiceContext,
  ) {
    const [balance, monthlyUsage] = await Promise.all([
      this.financialService.executeOperation('viewBalance', context, {}),
      this.getUsageStats(organizationId, 'current_month'),
    ]);

    // TODO: Implement billing history in AuditService
    const billingHistory = {
      lastPayment: null,
      nextBillingDate: null,
      payments: [],
    };

    return {
      organizationId,
      billing: {
        currentBalance: (balance as any)?.data?.amount || 0,
        currency: (balance as any)?.data?.currency || 'KES',
        monthlySpend: monthlyUsage?.statistics?.totalCost || 0,
        lastPayment: billingHistory.lastPayment,
        nextBillingDate: billingHistory.nextBillingDate,
        paymentHistory: billingHistory.payments,
      },
    };
  }

  // Governance and Compliance
  @Get(':organizationId/governance')
  @OrganizationScope([Permission.GOVERNANCE_READ])
  @ApiOperation({ summary: 'Get organization governance settings' })
  @ApiParam({ name: 'organizationId', description: 'Organization ID' })
  async getGovernance(@OrganizationId() organizationId: string) {
    const organization = await this.organizationService.findOne(organizationId);
    return {
      organizationId,
      governance: organization.governance || {},
      approvalWorkflows: [], // TODO: Add approvalWorkflows field to organization schema
      complianceStatus: organization.compliance?.complianceScore
        ? 'verified'
        : 'pending',
    };
  }

  @Put(':organizationId/governance')
  @OrganizationScope([Permission.GOVERNANCE_UPDATE])
  @RequiresApproval(['org_admin'], [Permission.GOVERNANCE_APPROVE])
  @ApiOperation({ summary: 'Update organization governance settings' })
  @ApiParam({ name: 'organizationId', description: 'Organization ID' })
  async updateGovernance(
    @OrganizationId() organizationId: string,
    @Body()
    governanceData: {
      governance?: any;
      approvalWorkflows?: any[];
      complianceSettings?: any;
    },
    @CurrentUser() member: AuthenticatedMember,
    @Context() context: ServiceContext,
  ) {
    const organization = await this.organizationService.update(
      organizationId,
      governanceData as any, // TODO: Create proper DTO for governance updates
    );

    await this.auditService.logAuditEvent({
      action: 'GOVERNANCE_UPDATED',
      memberId: member.memberId,
      organizationId,
      resourceType: 'governance',
      resourceId: organizationId,
      scope: context.scope,
      timestamp: new Date(),
      businessContext: {
        businessJustification: 'Updated organization governance settings',
        approvalRequired: true,
      },
      complianceContext: {
        riskLevel: RiskLevel.HIGH,
        sensitiveData: false,
        approvalRequired: true,
      },
    });

    return organization;
  }

  // Organization Settings
  @Get(':organizationId/settings')
  @OrganizationScope([Permission.ORG_READ])
  @ApiOperation({ summary: 'Get organization settings' })
  @ApiParam({ name: 'organizationId', description: 'Organization ID' })
  async getSettings(@OrganizationId() organizationId: string) {
    const organization = await this.organizationService.findOne(organizationId);
    return {
      organizationId,
      settings: organization.settings || {},
      limits: organization.limits,
      notifications: organization.settings || {},
    };
  }

  @Put(':organizationId/settings')
  @OrganizationScope([Permission.ORG_UPDATE])
  @ApiOperation({ summary: 'Update organization settings' })
  @ApiParam({ name: 'organizationId', description: 'Organization ID' })
  async updateSettings(
    @OrganizationId() organizationId: string,
    @Body()
    settingsData: {
      settings?: any;
      limits?: any;
      notificationSettings?: any;
    },
    @CurrentUser() member: AuthenticatedMember,
    @Context() context: ServiceContext,
  ) {
    const organization = await this.organizationService.update(
      organizationId,
      settingsData as any, // TODO: Create proper DTO for settings updates
    );

    await this.auditService.logAuditEvent({
      action: 'SETTINGS_UPDATED',
      memberId: member.memberId,
      organizationId,
      resourceType: 'organization_settings',
      resourceId: organizationId,
      scope: context.scope,
      timestamp: new Date(),
      businessContext: {
        businessJustification: 'Updated organization settings',
      },
      complianceContext: {
        riskLevel: RiskLevel.LOW,
        sensitiveData: false,
      },
    });

    return organization;
  }
}
