import {
  Controller,
  Get,
  Post,
  Put,
  Body,
  Param,
  Query,
  UseGuards,
} from '@nestjs/common';
import { ApiTags, ApiOperation, ApiParam } from '@nestjs/swagger';
import {
  CurrentUser,
  GlobalScope,
  RequiresApproval,
  AuthenticatedMember,
  Permission,
  PermissionScope,
  ServiceRole,
  GroupRole,
  AuthGuard,
  RequireRole,
  WorkflowType,
  ApprovalStatus,
  RiskLevel,
  Context,
  ApprovalRequest,
  WorkflowRequest,
  AuditQueryFilters,
  AuditService,
} from '../common';
import {
  MakerCheckerService,
  SegregationService,
  OperationContext,
  ComplianceService,
  ComplianceMetrics,
  RiskManagementService,
  TransactionRisk,
  RiskAssessment,
} from '../base';

/**
 * Compliance Controller - Maker-Checker, Risk Management, and Regulatory Features
 */
@ApiTags('compliance')
@Controller('compliance')
@UseGuards(AuthGuard)
export class ComplianceController {
  constructor(
    private makerCheckerService: MakerCheckerService,
    private segregationService: SegregationService,
    private complianceService: ComplianceService,
    private riskManagementService: RiskManagementService,
    private auditService: AuditService,
  ) {}

  // Maker-Checker Workflows

  @Post('workflows')
  @GlobalScope([Permission.FINANCE_DEPOSIT, Permission.FINANCE_WITHDRAW])
  @ApiOperation({ summary: 'Initiate approval workflow' })
  async initiateWorkflow(
    @CurrentUser() member: AuthenticatedMember,
    @Body()
    workflowData: {
      workflowType: WorkflowType;
      scope: PermissionScope;
      organizationId: string;
      chamaId?: string;
      operationData: {
        action: string;
        resourceType: string;
        resourceId?: string;
        parameters: Record<string, any>;
        estimatedValue?: number;
        currency?: string;
        description: string;
      };
      metadata?: {
        sourceSystem?: string;
        correlationId?: string;
        businessJustification?: string;
        urgency?: 'low' | 'medium' | 'high' | 'critical';
        customerImpact?: 'none' | 'low' | 'medium' | 'high';
      };
    },
  ) {
    const request: WorkflowRequest = {
      workflowType: workflowData.workflowType,
      scope: workflowData.scope,
      organizationId: workflowData.organizationId,
      chamaId: workflowData.chamaId,
      operationData: workflowData.operationData,
      metadata: workflowData.metadata,
    };

    return await this.makerCheckerService.initiateWorkflow(member, request);
  }

  @Get('workflows/pending')
  @GlobalScope([Permission.FINANCE_APPROVE])
  @ApiOperation({ summary: 'Get pending workflows for approval' })
  async getPendingWorkflows(
    @CurrentUser() member: AuthenticatedMember,
    @Query('scope') scope?: PermissionScope,
    @Query('workflowType') workflowType?: WorkflowType,
    @Query('limit') limit?: number,
    @Query('offset') offset?: number,
  ) {
    return await this.makerCheckerService.getPendingWorkflows(
      member,
      scope,
      workflowType,
      limit,
      offset,
    );
  }

  @Get('workflows/:workflowId')
  @GlobalScope([Permission.FINANCE_READ])
  @ApiOperation({ summary: 'Get workflow details' })
  @ApiParam({ name: 'workflowId', description: 'Workflow ID' })
  async getWorkflow(
    @CurrentUser() member: AuthenticatedMember,
    @Param('workflowId') workflowId: string,
  ) {
    return await this.makerCheckerService.getWorkflow(member, workflowId);
  }

  @Post('workflows/:workflowId/approve')
  @RequiresApproval()
  @GlobalScope([Permission.FINANCE_APPROVE])
  @ApiOperation({ summary: 'Approve or reject workflow' })
  @ApiParam({ name: 'workflowId', description: 'Workflow ID' })
  async submitApproval(
    @CurrentUser() member: AuthenticatedMember,
    @Param('workflowId') workflowId: string,
    @Body()
    approvalData: {
      status: 'approved' | 'rejected';
      comment?: string;
      ipAddress?: string;
      memberAgent?: string;
    },
  ) {
    const request: ApprovalRequest = {
      workflowId,
      status:
        approvalData.status === 'approved'
          ? ApprovalStatus.APPROVED
          : ApprovalStatus.REJECTED,
      comment: approvalData.comment,
      ipAddress: approvalData.ipAddress,
      memberAgent: approvalData.memberAgent,
    };

    return await this.makerCheckerService.submitApproval(member, request);
  }

  @Post('workflows/:workflowId/cancel')
  @GlobalScope([Permission.FINANCE_WITHDRAW])
  @ApiOperation({ summary: 'Cancel pending workflow' })
  @ApiParam({ name: 'workflowId', description: 'Workflow ID' })
  async cancelWorkflow(
    @CurrentUser() member: AuthenticatedMember,
    @Param('workflowId') workflowId: string,
    @Body() cancellationData: { reason: string },
  ) {
    return await this.makerCheckerService.cancelWorkflow(
      member,
      workflowId,
      cancellationData.reason,
    );
  }

  // Segregation of Duties

  @Post('sod/check')
  @GlobalScope([Permission.SYSTEM_MONITOR])
  @ApiOperation({ summary: 'Check segregation of duties violations' })
  async checkSegregationViolation(
    @CurrentUser() member: AuthenticatedMember,
    @Body()
    operationData: {
      action: string;
      permissions: Permission[];
      roles: (ServiceRole | GroupRole)[];
      scope: PermissionScope;
      organizationId?: string;
      chamaId?: string;
      sessionId?: string;
      metadata?: Record<string, any>;
    },
  ) {
    const operationContext: Omit<OperationContext, 'timestamp'> = {
      memberId: member.memberId,
      action: operationData.action,
      permissions: operationData.permissions,
      roles: operationData.roles,
      scope: operationData.scope,
      organizationId: operationData.organizationId,
      chamaId: operationData.chamaId,
      sessionId: operationData.sessionId,
      metadata: operationData.metadata,
    };

    return await this.segregationService.checkSegregationViolation(
      member,
      operationContext,
    );
  }

  @Get('sod/rules')
  @RequireRole(ServiceRole.ADMIN)
  @GlobalScope([Permission.SYSTEM_CONFIG])
  @ApiOperation({ summary: 'Get segregation of duties rules (ADMIN+)' })
  async getSegregationRules(
    @CurrentUser() member: AuthenticatedMember,
    @Query('scope') scope?: PermissionScope,
    @Query('isActive') isActive?: boolean,
  ) {
    return await this.segregationService.getSegregationRules(
      member,
      scope,
      isActive,
    );
  }

  @Post('sod/rules')
  @RequireRole(ServiceRole.SYSTEM_ADMIN)
  @GlobalScope([Permission.SYSTEM_CONFIG])
  @ApiOperation({ summary: 'Create segregation rule (SYSTEM-ADMIN only)' })
  async createSegregationRule(
    @CurrentUser() member: AuthenticatedMember,
    @Body()
    ruleData: {
      ruleName: string;
      description: string;
      scope: PermissionScope;
      conflictingOperations: {
        operation1: {
          action: string;
          permissions: Permission[];
          roles: (ServiceRole | GroupRole)[];
        };
        operation2: {
          action: string;
          permissions: Permission[];
          roles: (ServiceRole | GroupRole)[];
        };
        conflictType:
          | 'same_member'
          | 'same_role'
          | 'same_session'
          | 'time_window';
        timeWindowHours?: number;
      };
      enforcement: {
        blockConflicting: boolean;
        requireApproval: boolean;
        alertLevel: 'info' | 'warning' | 'critical';
        notificationChannels: ('email' | 'sms' | 'dashboard' | 'audit')[];
      };
    },
  ) {
    return await this.segregationService.createSegregationRule(
      member,
      ruleData,
    );
  }

  @Put('sod/rules/:ruleId/toggle')
  @RequireRole(ServiceRole.SYSTEM_ADMIN)
  @GlobalScope([Permission.SYSTEM_CONFIG])
  @ApiOperation({
    summary: 'Activate/deactivate segregation rule (SYSTEM-ADMIN only)',
  })
  @ApiParam({ name: 'ruleId', description: 'Rule ID' })
  async toggleSegregationRule(
    @CurrentUser() member: AuthenticatedMember,
    @Param('ruleId') ruleId: string,
    @Body() toggleData: { isActive: boolean },
  ) {
    return await this.segregationService.toggleSegregationRule(
      member,
      ruleId,
      toggleData.isActive,
    );
  }

  @Get('sod/violations/report')
  @RequireRole(ServiceRole.ADMIN)
  @GlobalScope([Permission.REPORTS_READ])
  @ApiOperation({ summary: 'Get segregation violations report (ADMIN+)' })
  async getViolationReport(
    @CurrentUser() member: AuthenticatedMember,
    @Query('startDate') startDate: string,
    @Query('endDate') endDate: string,
    @Query('scope') scope?: PermissionScope,
    @Query('organizationId') organizationId?: string,
    @Query('chamaId') chamaId?: string,
  ) {
    return await this.segregationService.getViolationReport(
      member,
      new Date(startDate),
      new Date(endDate),
      scope,
      organizationId,
      chamaId,
    );
  }

  // Risk Management

  @Post('risk/assess')
  @GlobalScope([Permission.FINANCE_READ])
  @ApiOperation({ summary: 'Assess transaction risk' })
  async assessTransactionRisk(
    @CurrentUser() member: AuthenticatedMember,
    @Context() context: any,
    @Body()
    riskData: {
      amount: number;
      currency: string;
      transactionType: string;
      frequency: number;
      memberRiskProfile: 'low' | 'medium' | 'high';
      counterpartyRisk?: 'low' | 'medium' | 'high';
      geographicRisk?: 'low' | 'medium' | 'high';
      timeOfDay: number;
      isWeekend: boolean;
      isHoliday?: boolean;
    },
  ): Promise<RiskAssessment> {
    const transactionRisk: TransactionRisk = {
      amount: riskData.amount,
      currency: riskData.currency,
      transactionType: riskData.transactionType,
      frequency: riskData.frequency,
      memberRiskProfile: riskData.memberRiskProfile,
      counterpartyRisk: riskData.counterpartyRisk,
      geographicRisk: riskData.geographicRisk,
      timeOfDay: riskData.timeOfDay,
      isWeekend: riskData.isWeekend,
      isHoliday: riskData.isHoliday,
    };

    return await this.riskManagementService.assessTransactionRisk(
      member,
      transactionRisk,
      context.scope,
      context.organizationId,
      context.chamaId,
    );
  }

  @Post('risk/limits/check')
  @GlobalScope([Permission.FINANCE_READ])
  @ApiOperation({ summary: 'Check transaction limits' })
  async checkTransactionLimits(
    @CurrentUser() member: AuthenticatedMember,
    @Context() context: any,
    @Body()
    limitCheckData: {
      amount: number;
      currency: string;
      operationType: string;
    },
  ) {
    return await this.riskManagementService.checkTransactionLimits(
      member,
      limitCheckData.amount,
      limitCheckData.currency,
      limitCheckData.operationType,
      context.scope,
      context.organizationId,
      context.chamaId,
    );
  }

  @Get('risk/limits')
  @GlobalScope([Permission.FINANCE_READ])
  @ApiOperation({ summary: 'Get transaction limits' })
  async getTransactionLimits(
    @CurrentUser() member: AuthenticatedMember,
    @Query('scope') scope?: PermissionScope,
    @Query('organizationId') organizationId?: string,
    @Query('chamaId') chamaId?: string,
    @Query('isActive') isActive?: boolean,
  ) {
    return await this.riskManagementService.getTransactionLimits(
      member,
      scope,
      organizationId,
      chamaId,
      isActive,
    );
  }

  @Post('risk/limits')
  @RequireRole(ServiceRole.ADMIN)
  @GlobalScope([Permission.SYSTEM_CONFIG])
  @ApiOperation({ summary: 'Create transaction limit (ADMIN+)' })
  async createTransactionLimit(
    @CurrentUser() member: AuthenticatedMember,
    @Body()
    limitData: {
      limitName: string;
      scope: PermissionScope;
      organizationId?: string;
      chamaId?: string;
      memberId?: string;
      applicableRoles: (ServiceRole | GroupRole)[];
      currency: string;
      limits: {
        maxTransactionAmount: number;
        minTransactionAmount?: number;
        dailyLimit?: number;
        weeklyLimit?: number;
        monthlyLimit?: number;
        yearlyLimit?: number;
        dailyTransactionCount?: number;
        monthlyTransactionCount?: number;
        totalLifetimeLimit?: number;
        outstandingLimit?: number;
      };
      applicableOperations: string[];
      overrideConditions: {
        allowOverride: boolean;
        overrideRoles: (ServiceRole | GroupRole)[];
        overridePermissions: Permission[];
        requiresApproval: boolean;
        maxOverridePercentage?: number;
      };
      effectiveFrom: Date;
      effectiveUntil?: Date;
    },
  ) {
    return await this.riskManagementService.createTransactionLimit(
      member,
      limitData,
    );
  }

  @Get('risk/report')
  @RequireRole(ServiceRole.ADMIN)
  @GlobalScope([Permission.REPORTS_READ])
  @ApiOperation({ summary: 'Generate risk report (ADMIN+)' })
  async generateRiskReport(
    @Query('scope') scope: PermissionScope,
    @Query('organizationId') organizationId?: string,
    @Query('chamaId') chamaId?: string,
    @Query('days') days?: number,
  ) {
    return await this.riskManagementService.generateRiskReport(
      scope,
      organizationId,
      chamaId,
      days,
    );
  }

  @Get('risk/monitor/realtime')
  @RequireRole(ServiceRole.ADMIN)
  @GlobalScope([Permission.SYSTEM_MONITOR])
  @ApiOperation({ summary: 'Monitor real-time risk patterns (ADMIN+)' })
  async monitorRealTimeRisk(
    @Query('scope') scope: PermissionScope,
    @Query('organizationId') organizationId?: string,
    @Query('chamaId') chamaId?: string,
  ) {
    return await this.riskManagementService.monitorRealTimeRisk(
      scope,
      organizationId,
      chamaId,
    );
  }

  // Compliance Monitoring

  @Get('events')
  @RequireRole(ServiceRole.ADMIN)
  @GlobalScope([Permission.REPORTS_READ])
  @ApiOperation({ summary: 'Get compliance events (ADMIN+)' })
  async getComplianceEvents(
    @CurrentUser() member: AuthenticatedMember,
    @Query('eventType') eventType?: string,
    @Query('severity') severity?: RiskLevel,
    @Query('scope') scope?: PermissionScope,
    @Query('organizationId') organizationId?: string,
    @Query('chamaId') chamaId?: string,
    @Query('status')
    status?: 'open' | 'investigating' | 'resolved' | 'false_positive',
    @Query('startDate') startDate?: string,
    @Query('endDate') endDate?: string,
    @Query('limit') limit?: number,
    @Query('offset') offset?: number,
  ) {
    const filters = {
      eventType,
      severity,
      scope,
      organizationId,
      chamaId,
      status,
      startDate: startDate ? new Date(startDate) : undefined,
      endDate: endDate ? new Date(endDate) : undefined,
      limit,
      offset,
    };

    return await this.complianceService.getComplianceEvents(member, filters);
  }

  @Put('events/:eventId')
  @RequireRole(ServiceRole.ADMIN)
  @GlobalScope([Permission.SYSTEM_CONFIG])
  @ApiOperation({ summary: 'Update compliance event status (ADMIN+)' })
  @ApiParam({ name: 'eventId', description: 'Event ID' })
  async updateEventStatus(
    @CurrentUser() member: AuthenticatedMember,
    @Param('eventId') eventId: string,
    @Body()
    updateData: {
      status: 'open' | 'investigating' | 'resolved' | 'false_positive';
      assignedTo?: string;
      resolutionNotes?: string;
      escalate?: boolean;
      escalatedTo?: string;
    },
  ) {
    return await this.complianceService.updateEventStatus(
      member,
      eventId,
      updateData,
    );
  }

  @Get('metrics')
  @RequireRole(ServiceRole.ADMIN)
  @GlobalScope([Permission.REPORTS_READ])
  @ApiOperation({ summary: 'Get compliance metrics dashboard (ADMIN+)' })
  async getComplianceMetrics(
    @Query('scope') scope: PermissionScope,
    @Query('organizationId') organizationId?: string,
    @Query('chamaId') chamaId?: string,
    @Query('timeRange') timeRange?: '24h' | '7d' | '30d' | '90d',
  ): Promise<ComplianceMetrics> {
    return await this.complianceService.getComplianceMetrics(
      scope,
      organizationId,
      chamaId,
      timeRange,
    );
  }

  @Get('health')
  @RequireRole(ServiceRole.ADMIN)
  @GlobalScope([Permission.SYSTEM_MONITOR])
  @ApiOperation({ summary: 'Run compliance health check (ADMIN+)' })
  async runComplianceHealthCheck(
    @Query('scope') scope: PermissionScope,
    @Query('organizationId') organizationId?: string,
    @Query('chamaId') chamaId?: string,
  ) {
    return await this.complianceService.runComplianceHealthCheck(
      scope,
      organizationId,
      chamaId,
    );
  }

  // Regulatory Reporting

  @Post('reports/generate')
  @RequireRole(ServiceRole.ADMIN)
  @GlobalScope([Permission.REPORTS_EXPORT])
  @ApiOperation({ summary: 'Generate regulatory report (ADMIN+)' })
  async generateRegulatoryReport(
    @CurrentUser() member: AuthenticatedMember,
    @Body()
    reportData: {
      reportType: string;
      regulator: string;
      reportingPeriod: {
        startDate: Date;
        endDate: Date;
        frequency: 'daily' | 'weekly' | 'monthly' | 'quarterly' | 'annually';
      };
      scope: PermissionScope;
      organizationId?: string;
    },
  ) {
    return await this.complianceService.generateRegulatoryReport(
      member,
      reportData.reportType,
      reportData.regulator,
      reportData.reportingPeriod,
      reportData.scope,
      reportData.organizationId,
    );
  }

  @Get('reports')
  @RequireRole(ServiceRole.ADMIN)
  @GlobalScope([Permission.REPORTS_READ])
  @ApiOperation({ summary: 'Get regulatory reports (ADMIN+)' })
  async getRegulatoryReports(
    @CurrentUser() member: AuthenticatedMember,
    @Query('reportType') reportType?: string,
    @Query('regulator') regulator?: string,
    @Query('status') status?: string,
    @Query('scope') scope?: PermissionScope,
    @Query('organizationId') organizationId?: string,
    @Query('startDate') startDate?: string,
    @Query('endDate') endDate?: string,
    @Query('limit') limit?: number,
    @Query('offset') offset?: number,
  ) {
    const filters = {
      reportType,
      regulator,
      status,
      scope,
      organizationId,
      startDate: startDate ? new Date(startDate) : undefined,
      endDate: endDate ? new Date(endDate) : undefined,
      limit,
      offset,
    };

    return await this.complianceService.getRegulatoryReports(member, filters);
  }

  @Post('reports/:reportId/submit')
  @RequireRole(ServiceRole.ADMIN)
  @GlobalScope([Permission.REPORTS_EXPORT])
  @ApiOperation({ summary: 'Submit regulatory report (ADMIN+)' })
  @ApiParam({ name: 'reportId', description: 'Report ID' })
  async submitRegulatoryReport(
    @CurrentUser() member: AuthenticatedMember,
    @Param('reportId') reportId: string,
  ) {
    return await this.complianceService.submitRegulatoryReport(
      member,
      reportId,
    );
  }

  // Audit Logs

  @Get('audit')
  @RequireRole(ServiceRole.ADMIN)
  @GlobalScope([Permission.REPORTS_READ])
  @ApiOperation({ summary: 'Get audit logs (ADMIN+)' })
  async getAuditLogs(
    @Query('memberId') memberId?: string,
    @Query('action') action?: string,
    @Query('resourceType') resourceType?: string,
    @Query('scope') scope?: PermissionScope,
    @Query('organizationId') organizationId?: string,
    @Query('chamaId') chamaId?: string,
    @Query('startDate') startDate?: string,
    @Query('endDate') endDate?: string,
    @Query('sensitiveData') sensitiveData?: boolean,
    @Query('riskLevel') riskLevel?: string,
    @Query('limit') limit?: number,
    @Query('offset') offset?: number,
  ) {
    const filters: AuditQueryFilters = {
      memberId,
      action,
      resourceType,
      scope,
      organizationId,
      chamaId,
      startDate: startDate ? new Date(startDate) : undefined,
      endDate: endDate ? new Date(endDate) : undefined,
      sensitiveData,
      riskLevel: riskLevel as RiskLevel,
      limit,
      offset,
    };

    return await this.auditService.getAuditLogs(filters);
  }

  @Get('audit/search')
  @RequireRole(ServiceRole.ADMIN)
  @GlobalScope([Permission.REPORTS_READ])
  @ApiOperation({
    summary: 'Search audit logs with advanced criteria (ADMIN+)',
  })
  async searchAuditLogs(
    @Query('keyword') keyword?: string,
    @Query('memberId') memberId?: string,
    @Query('ipAddress') ipAddress?: string,
    @Query('memberAgent') memberAgent?: string,
    @Query('minAmount') minAmount?: number,
    @Query('maxAmount') maxAmount?: number,
    @Query('riskLevels') riskLevels?: string,
    @Query('dataClassifications') dataClassifications?: string,
    @Query('scope') scope?: PermissionScope,
    @Query('organizationId') organizationId?: string,
    @Query('chamaId') chamaId?: string,
    @Query('startDate') startDate?: string,
    @Query('endDate') endDate?: string,
    @Query('limit') limit?: number,
    @Query('offset') offset?: number,
  ) {
    const searchCriteria = {
      keyword,
      memberId,
      ipAddress,
      memberAgent,
      amountRange:
        minAmount && maxAmount ? { min: minAmount, max: maxAmount } : undefined,
      riskLevels: riskLevels ? riskLevels.split(',') : undefined,
      dataClassifications: dataClassifications
        ? dataClassifications.split(',')
        : undefined,
      scope,
      organizationId,
      chamaId,
      startDate: startDate ? new Date(startDate) : undefined,
      endDate: endDate ? new Date(endDate) : undefined,
      limit,
      offset,
    };

    return await this.auditService.searchAuditLogs(searchCriteria);
  }

  @Get('audit/statistics')
  @RequireRole(ServiceRole.ADMIN)
  @GlobalScope([Permission.REPORTS_READ])
  @ApiOperation({ summary: 'Get audit statistics and analytics (ADMIN+)' })
  async getAuditStatistics(
    @Query('scope') scope?: PermissionScope,
    @Query('organizationId') organizationId?: string,
    @Query('chamaId') chamaId?: string,
    @Query('days') days?: number,
  ) {
    return await this.auditService.getAuditStatistics(
      scope,
      organizationId,
      chamaId,
      days,
    );
  }

  @Post('audit/export')
  @RequireRole(ServiceRole.ADMIN)
  @GlobalScope([Permission.REPORTS_EXPORT])
  @ApiOperation({
    summary: 'Export audit logs for compliance reporting (ADMIN+)',
  })
  async exportAuditLogs(
    @Body()
    exportRequest: {
      filters: AuditQueryFilters;
      format: 'json' | 'csv' | 'pdf';
    },
  ) {
    return await this.auditService.exportAuditLogs(
      exportRequest.filters,
      exportRequest.format,
    );
  }

  @Get('audit/compliance-report')
  @RequireRole(ServiceRole.ADMIN)
  @GlobalScope([Permission.REPORTS_EXPORT])
  @ApiOperation({ summary: 'Get compliance-ready audit report (ADMIN+)' })
  async getComplianceAuditReport(
    @CurrentUser() member: AuthenticatedMember,
    @Query('scope') scope: PermissionScope,
    @Query('organizationId') organizationId?: string,
    @Query('startDate') startDate?: string,
    @Query('endDate') endDate?: string,
  ) {
    return await this.auditService.getComplianceAuditReport(
      scope,
      organizationId,
      startDate ? new Date(startDate) : undefined,
      endDate ? new Date(endDate) : undefined,
    );
  }
}
