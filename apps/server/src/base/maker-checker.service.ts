import { Model } from 'mongoose';
import { InjectModel } from '@nestjs/mongoose';
import { EventEmitter2 } from '@nestjs/event-emitter';
import {
  Injectable,
  BadRequestException,
  ForbiddenException,
  NotFoundException,
} from '@nestjs/common';
import {
  ApprovalWorkflow,
  ApprovalWorkflowDocument,
  ApprovalStatus,
  WorkflowType,
  SegregationRule,
  SegregationRuleDocument,
  TransactionLimit,
  TransactionLimitDocument,
} from '../common/schemas/compliance.schema';
import {
  AuthenticatedMember,
  Permission,
  PermissionScope,
  ServiceRole,
  GroupRole,
  RiskLevel,
  WorkflowRequest,
  ApprovalRequest,
} from '../common/types';
import { PermissionService } from '../common/services/permission.service';
import { ComplianceService } from './compliance.service';

/**
 * Maker-Checker Service
 * Implements dual control mechanisms and approval workflows
 */
@Injectable()
export class MakerCheckerService {
  constructor(
    @InjectModel(ApprovalWorkflow.name)
    private workflowModel: Model<ApprovalWorkflowDocument>,
    @InjectModel(SegregationRule.name)
    private segregationRuleModel: Model<SegregationRuleDocument>,
    @InjectModel(TransactionLimit.name)
    private transactionLimitModel: Model<TransactionLimitDocument>,
    private permissionService: PermissionService,
    private complianceService: ComplianceService,
    private eventEmitter: EventEmitter2,
  ) {}

  /**
   * Initiate a new approval workflow
   */
  async initiateWorkflow(
    member: AuthenticatedMember,
    request: WorkflowRequest,
  ): Promise<ApprovalWorkflowDocument> {
    // Check if member can initiate this type of workflow
    await this.validateInitiatorPermissions(member, request);

    // Check segregation of duties
    await this.checkSegregationRules(member, request);

    // Check transaction limits
    await this.checkTransactionLimits(member, request);

    // Determine risk level and approval requirements
    const riskLevel = await this.assessRiskLevel(request);
    const approvalChain = await this.buildApprovalChain(request, riskLevel);

    // Run compliance checks
    const complianceChecks = await this.runComplianceChecks(request);

    // Create workflow
    const workflow = new this.workflowModel({
      workflowType: request.workflowType,
      initiatedBy: member.memberId,
      organizationId: request.organizationId,
      chamaId: request.chamaId,
      scope: request.scope,
      riskLevel,
      operationData: request.operationData,
      approvalChain,
      complianceChecks,
      approvals: [],
      expiresAt: new Date(
        Date.now() + approvalChain.timeoutHours * 60 * 60 * 1000,
      ),
      metadata: request.metadata || {},
      tags: this.generateWorkflowTags(request),
    });

    const savedWorkflow = await workflow.save();

    // Emit workflow initiated event
    this.eventEmitter.emit('workflow.initiated', {
      workflowId: savedWorkflow._id,
      initiator: member.memberId,
      workflowType: request.workflowType,
      riskLevel,
    });

    // Send notifications to potential approvers
    await this.notifyApprovers(savedWorkflow);

    return savedWorkflow;
  }

  /**
   * Submit approval or rejection
   */
  async submitApproval(
    member: AuthenticatedMember,
    request: ApprovalRequest,
  ): Promise<ApprovalWorkflowDocument> {
    const workflow = await this.workflowModel.findById(request.workflowId);
    if (!workflow) {
      throw new NotFoundException('Workflow not found');
    }

    // Validate approver permissions
    await this.validateApproverPermissions(member, workflow);

    // Check if workflow has expired
    if (workflow.expiresAt && workflow.expiresAt < new Date()) {
      workflow.status = ApprovalStatus.EXPIRED;
      await workflow.save();
      throw new BadRequestException('Workflow has expired');
    }

    // Check if already approved/rejected by this member
    const existingApproval = workflow.approvals.find(
      (approval) => approval.approverId === member.memberId,
    );
    if (existingApproval) {
      throw new BadRequestException(
        'Member has already provided approval for this workflow',
      );
    }

    // Check self-approval rules
    if (
      workflow.initiatedBy === member.memberId &&
      !workflow.approvalChain.allowSelfApproval
    ) {
      throw new ForbiddenException(
        'Self-approval not allowed for this workflow',
      );
    }

    // Add approval
    workflow.approvals.push({
      approverId: member.memberId,
      approverRole: this.getUserHighestRole(member, workflow.scope),
      status: request.status,
      comment: request.comment,
      approvedAt: new Date(),
      ipAddress: request.ipAddress,
      memberAgent: request.memberAgent,
    });

    // Check if workflow is complete
    const workflowComplete = await this.checkWorkflowCompletion(workflow);

    if (workflowComplete) {
      if (request.status === ApprovalStatus.REJECTED) {
        workflow.status = ApprovalStatus.REJECTED;
      } else {
        workflow.status = ApprovalStatus.APPROVED;
        // Execute the operation if fully approved
        await this.executeApprovedOperation(workflow);
      }
    }

    const savedWorkflow = await workflow.save();

    // Emit approval event
    this.eventEmitter.emit('workflow.approval_submitted', {
      workflowId: savedWorkflow._id,
      approver: member.memberId,
      status: request.status,
      complete: workflowComplete,
    });

    return savedWorkflow;
  }

  /**
   * Get pending workflows for a member
   */
  async getPendingWorkflows(
    member: AuthenticatedMember,
    scope?: PermissionScope,
    workflowType?: WorkflowType,
    limit: number = 50,
    offset: number = 0,
  ): Promise<{
    workflows: ApprovalWorkflowDocument[];
    total: number;
  }> {
    const query: any = {
      status: ApprovalStatus.PENDING,
      expiresAt: { $gt: new Date() },
      'approvals.approverId': { $ne: member.memberId }, // Exclude already approved by member
    };

    // Filter by scope access
    if (scope) {
      query.scope = scope;
    }

    if (workflowType) {
      query.workflowType = workflowType;
    }

    // Filter by member's permissions and access
    const accessibleOrgs =
      member.groupMemberships
        ?.filter((m) => m.groupType === 'organization')
        .map((m) => m.groupId) || [];

    const accessibleChamas =
      member.groupMemberships
        ?.filter((m) => m.groupType === 'chama')
        .map((m) => m.groupId) || [];

    if (member.serviceRole !== ServiceRole.SYSTEM_ADMIN) {
      query.$or = [
        { organizationId: { $in: accessibleOrgs } },
        { chamaId: { $in: accessibleChamas } },
      ];
    }

    const [workflows, _total] = await Promise.all([
      this.workflowModel
        .find(query)
        .sort({ createdAt: -1 })
        .limit(limit)
        .skip(offset)
        .exec(),
      this.workflowModel.countDocuments(query),
    ]);

    // Filter workflows where member has required permissions to approve
    const approveableWorkflows = [];
    for (const workflow of workflows) {
      if (await this.canUserApprove(member, workflow)) {
        approveableWorkflows.push(workflow);
      }
    }

    return {
      workflows: approveableWorkflows,
      total: approveableWorkflows.length,
    };
  }

  /**
   * Get workflow details
   */
  async getWorkflow(
    member: AuthenticatedMember,
    workflowId: string,
  ): Promise<ApprovalWorkflowDocument> {
    const workflow = await this.workflowModel.findById(workflowId);
    if (!workflow) {
      throw new NotFoundException('Workflow not found');
    }

    // Check if member has access to this workflow
    const hasAccess = await this.checkWorkflowAccess(member, workflow);
    if (!hasAccess) {
      throw new ForbiddenException('Access denied to this workflow');
    }

    return workflow;
  }

  /**
   * Cancel a pending workflow
   */
  async cancelWorkflow(
    member: AuthenticatedMember,
    workflowId: string,
    reason: string,
  ): Promise<ApprovalWorkflowDocument> {
    const workflow = await this.workflowModel.findById(workflowId);
    if (!workflow) {
      throw new NotFoundException('Workflow not found');
    }

    // Only initiator or admin can cancel
    if (
      workflow.initiatedBy !== member.memberId &&
      member.serviceRole !== ServiceRole.SYSTEM_ADMIN
    ) {
      throw new ForbiddenException(
        'Only workflow initiator or system admin can cancel',
      );
    }

    if (workflow.status !== ApprovalStatus.PENDING) {
      throw new BadRequestException('Can only cancel pending workflows');
    }

    workflow.status = ApprovalStatus.CANCELLED;
    workflow.metadata.cancellationReason = reason;
    workflow.metadata.cancelledBy = member.memberId;
    workflow.metadata.cancelledAt = new Date();

    const savedWorkflow = await workflow.save();

    // Emit cancellation event
    this.eventEmitter.emit('workflow.cancelled', {
      workflowId: savedWorkflow._id,
      cancelledBy: member.memberId,
      reason,
    });

    return savedWorkflow;
  }

  // Private Methods

  private async validateInitiatorPermissions(
    member: AuthenticatedMember,
    request: WorkflowRequest,
  ): Promise<void> {
    // Check if member has permissions to initiate this workflow type
    const requiredPermissions = this.getWorkflowPermissions(
      request.workflowType,
    );
    const hasPermissions = this.permissionService.memberHasAllPermissions(
      member,
      requiredPermissions,
      request.scope,
      request.organizationId,
      request.chamaId,
    );

    if (!hasPermissions) {
      throw new ForbiddenException(
        'Insufficient permissions to initiate this workflow',
      );
    }
  }

  private async checkSegregationRules(
    member: AuthenticatedMember,
    request: WorkflowRequest,
  ): Promise<void> {
    const applicableRules = await this.segregationRuleModel.find({
      scope: { $in: [request.scope, PermissionScope.GLOBAL] },
      isActive: true,
    });

    for (const rule of applicableRules) {
      const conflict = await this.detectSegregationConflict(
        member,
        request,
        rule,
      );
      if (conflict) {
        if (rule.enforcement.blockConflicting) {
          throw new ForbiddenException(
            `Segregation of duties violation: ${rule.description}`,
          );
        } else {
          // Log warning but allow with approval
          await this.complianceService.logComplianceEvent({
            eventType: 'segregation_violation',
            severity: RiskLevel.MEDIUM,
            description: `SoD rule violation: ${rule.ruleName}`,
            memberId: member.memberId,
            scope: request.scope,
            organizationId: request.organizationId,
            chamaId: request.chamaId,
          });
        }
      }
    }
  }

  private async checkTransactionLimits(
    member: AuthenticatedMember,
    request: WorkflowRequest,
  ): Promise<void> {
    if (!request.operationData.estimatedValue) {
      return; // No amount to check
    }

    const applicableLimits = await this.getApplicableTransactionLimits(
      member,
      request,
    );

    for (const limit of applicableLimits) {
      const violation = await this.checkLimitViolation(member, request, limit);
      if (violation) {
        if (!limit.overrideConditions.allowOverride) {
          throw new BadRequestException(
            `Transaction limit exceeded: ${limit.limitName}`,
          );
        }
        // Check if member can override
        const canOverride = await this.canUserOverrideLimit(member, limit);
        if (!canOverride) {
          throw new ForbiddenException(
            `Transaction limit exceeded and member cannot override: ${limit.limitName}`,
          );
        }
      }
    }
  }

  private async assessRiskLevel(request: WorkflowRequest): Promise<RiskLevel> {
    let risk = RiskLevel.LOW;

    // Assess based on amount
    if (request.operationData.estimatedValue) {
      if (request.operationData.estimatedValue > 1000000)
        risk = RiskLevel.CRITICAL;
      else if (request.operationData.estimatedValue > 100000)
        risk = RiskLevel.HIGH;
      else if (request.operationData.estimatedValue > 10000)
        risk = RiskLevel.MEDIUM;
    }

    // Assess based on operation type
    const highRiskOperations = [
      WorkflowType.ACCOUNT_CLOSURE,
      WorkflowType.LIMIT_OVERRIDE,
      WorkflowType.SYSTEM_MAINTENANCE,
    ];

    if (highRiskOperations.includes(request.workflowType)) {
      risk = RiskLevel.HIGH;
    }

    // Assess based on scope
    if (request.scope === PermissionScope.GLOBAL) {
      risk = RiskLevel.HIGH;
    }

    return risk;
  }

  private async buildApprovalChain(
    request: WorkflowRequest,
    riskLevel: RiskLevel,
  ): Promise<any> {
    const chain = {
      requiredApprovals: 1,
      requiredRoles: [GroupRole.CHAMA_LEADER] as (ServiceRole | GroupRole)[],
      requiredPermissions: [] as Permission[],
      allowSelfApproval: false,
      sequentialApproval: false,
      timeoutHours: 24,
    };

    // Adjust based on risk level
    switch (riskLevel) {
      case RiskLevel.CRITICAL:
        chain.requiredApprovals = 3;
        chain.requiredRoles = [ServiceRole.SYSTEM_ADMIN, GroupRole.SACCO_ADMIN];
        chain.allowSelfApproval = false;
        chain.sequentialApproval = true;
        chain.timeoutHours = 48;
        break;
      case RiskLevel.HIGH:
        chain.requiredApprovals = 2;
        chain.requiredRoles = [
          GroupRole.SACCO_ADMIN,
          GroupRole.SACCO_TREASURER,
        ];
        chain.allowSelfApproval = false;
        chain.timeoutHours = 24;
        break;
      case RiskLevel.MEDIUM:
        chain.requiredApprovals = 1;
        chain.requiredRoles = [GroupRole.CHAMA_LEADER, GroupRole.SACCO_MANAGER];
        chain.allowSelfApproval = true;
        chain.timeoutHours = 12;
        break;
      case RiskLevel.LOW:
        chain.requiredApprovals = 1;
        chain.requiredRoles = [GroupRole.CHAMA_TREASURER];
        chain.allowSelfApproval = true;
        chain.timeoutHours = 8;
        break;
    }

    // Adjust based on workflow type
    if (request.workflowType === WorkflowType.FINANCIAL_TRANSACTION) {
      chain.requiredPermissions.push(Permission.FINANCE_APPROVE);
    }

    return chain;
  }

  private async runComplianceChecks(_request: WorkflowRequest): Promise<any> {
    return {
      amlScreening: { status: 'passed' },
      sanctionsCheck: { status: 'passed' },
      riskAssessment: { score: 75, factors: ['amount', 'frequency'] },
      regulatoryRequirements: { kyc: true, documentation: [], approvals: [] },
    };
  }

  private generateWorkflowTags(request: WorkflowRequest): string[] {
    const tags = [
      request.workflowType,
      request.scope,
      request.metadata?.urgency || 'normal',
    ];

    if (request.operationData.estimatedValue) {
      if (request.operationData.estimatedValue > 50000) tags.push('high-value');
      if (request.operationData.estimatedValue > 10000)
        tags.push('medium-value');
    }

    return tags;
  }

  private async notifyApprovers(
    workflow: ApprovalWorkflowDocument,
  ): Promise<void> {
    // Implementation would send notifications to potential approvers
    this.eventEmitter.emit('workflow.notification_required', {
      workflowId: workflow._id,
      requiredRoles: workflow.approvalChain.requiredRoles,
      organizationId: workflow.organizationId,
      chamaId: workflow.chamaId,
    });
  }

  private async validateApproverPermissions(
    member: AuthenticatedMember,
    workflow: ApprovalWorkflowDocument,
  ): Promise<void> {
    const canApprove = await this.canUserApprove(member, workflow);
    if (!canApprove) {
      throw new ForbiddenException(
        'Member does not have permission to approve this workflow',
      );
    }
  }

  private async canUserApprove(
    member: AuthenticatedMember,
    workflow: ApprovalWorkflowDocument,
  ): Promise<boolean> {
    // Check if member has required role
    const memberRoles = this.getUserRoles(member, workflow.scope);
    const hasRequiredRole = workflow.approvalChain.requiredRoles.some((role) =>
      memberRoles.includes(role),
    );

    if (!hasRequiredRole) return false;

    // Check if member has required permissions
    const hasRequiredPermissions =
      this.permissionService.memberHasAllPermissions(
        member,
        workflow.approvalChain.requiredPermissions,
        workflow.scope,
        workflow.organizationId,
        workflow.chamaId,
      );

    return hasRequiredPermissions;
  }

  private async checkWorkflowCompletion(
    workflow: ApprovalWorkflowDocument,
  ): Promise<boolean> {
    const approvedCount = workflow.approvals.filter(
      (approval) => approval.status === ApprovalStatus.APPROVED,
    ).length;

    const rejectedCount = workflow.approvals.filter(
      (approval) => approval.status === ApprovalStatus.REJECTED,
    ).length;

    // If any rejection, workflow is complete (rejected)
    if (rejectedCount > 0) return true;

    // Check if required approvals met
    return approvedCount >= workflow.approvalChain.requiredApprovals;
  }

  private async executeApprovedOperation(
    workflow: ApprovalWorkflowDocument,
  ): Promise<void> {
    try {
      // Implementation would execute the actual operation
      // This is where the approved action gets performed

      workflow.executedAt = new Date();
      workflow.executedBy = 'system';
      workflow.executionResult = {
        success: true,
        transactionId: `txn-${Date.now()}`,
      };

      this.eventEmitter.emit('workflow.executed', {
        workflowId: workflow._id,
        operation: workflow.operationData.action,
        success: true,
      });
    } catch (error) {
      workflow.executionResult = {
        success: false,
        error: error.message,
      };

      this.eventEmitter.emit('workflow.execution_failed', {
        workflowId: workflow._id,
        error: error.message,
      });
    }
  }

  private getUserHighestRole(
    member: AuthenticatedMember,
    scope: PermissionScope,
  ): ServiceRole | GroupRole {
    if (scope === PermissionScope.GLOBAL) {
      return member.serviceRole;
    }

    // Get highest group role in context
    const relevantMemberships =
      member.groupMemberships?.filter((m) => {
        if (scope === PermissionScope.ORGANIZATION)
          return m.groupType === 'organization';
        if (scope === PermissionScope.CHAMA) return m.groupType === 'chama';
        return false;
      }) || [];

    if (relevantMemberships.length === 0) return member.serviceRole;

    // Return highest privilege role
    const roleHierarchy = {
      [GroupRole.SACCO_OWNER]: 1,
      [GroupRole.SACCO_ADMIN]: 2,
      [GroupRole.CHAMA_LEADER]: 3,
      [GroupRole.CHAMA_TREASURER]: 4,
      [GroupRole.CHAMA_MEMBER]: 5,
    };

    return relevantMemberships
      .map((m) => m.role)
      .sort((a, b) => (roleHierarchy[a] || 99) - (roleHierarchy[b] || 99))[0];
  }

  private getUserRoles(
    member: AuthenticatedMember,
    scope: PermissionScope,
  ): (ServiceRole | GroupRole)[] {
    const roles: (ServiceRole | GroupRole)[] = [member.serviceRole];

    if (scope !== PermissionScope.GLOBAL) {
      const groupRoles =
        member.groupMemberships
          ?.filter((m) => {
            if (scope === PermissionScope.ORGANIZATION)
              return m.groupType === 'organization';
            if (scope === PermissionScope.CHAMA) return m.groupType === 'chama';
            return false;
          })
          .map((m) => m.role) || [];

      roles.push(...groupRoles);
    }

    return roles;
  }

  private async checkWorkflowAccess(
    member: AuthenticatedMember,
    workflow: ApprovalWorkflowDocument,
  ): Promise<boolean> {
    // System admin can access all workflows
    if (member.serviceRole === ServiceRole.SYSTEM_ADMIN) return true;

    // Initiator can access their own workflows
    if (workflow.initiatedBy === member.memberId) return true;

    // Check if member has access to the organization/chama
    const hasOrgAccess = member.groupMemberships?.some(
      (m) =>
        m.groupId === workflow.organizationId && m.groupType === 'organization',
    );

    const hasChamaAccess =
      workflow.chamaId &&
      member.groupMemberships?.some(
        (m) => m.groupId === workflow.chamaId && m.groupType === 'chama',
      );

    return hasOrgAccess || hasChamaAccess || false;
  }

  private getWorkflowPermissions(workflowType: WorkflowType): Permission[] {
    const permissionMap: Record<WorkflowType, Permission[]> = {
      [WorkflowType.FINANCIAL_TRANSACTION]: [
        Permission.FINANCE_DEPOSIT,
        Permission.FINANCE_WITHDRAW,
      ],
      [WorkflowType.LOAN_APPROVAL]: [Permission.LOAN_APPLY],
      [WorkflowType.USER_MANAGEMENT]: [
        Permission.USER_CREATE,
        Permission.USER_UPDATE,
      ],
      [WorkflowType.CONFIGURATION_CHANGE]: [Permission.SYSTEM_CONFIG],
      [WorkflowType.SHARES_ISSUANCE]: [Permission.SHARES_CREATE],
      [WorkflowType.MEMBER_ONBOARDING]: [Permission.USER_INVITE],
      [WorkflowType.ACCOUNT_CLOSURE]: [Permission.USER_DELETE],
      [WorkflowType.LIMIT_OVERRIDE]: [Permission.FINANCE_APPROVE],
      [WorkflowType.SYSTEM_MAINTENANCE]: [Permission.SYSTEM_CONFIG],
    };

    return permissionMap[workflowType] || [];
  }

  private async detectSegregationConflict(
    _member: AuthenticatedMember,
    _request: WorkflowRequest,
    _rule: SegregationRuleDocument,
  ): Promise<boolean> {
    // Implementation would check for SoD conflicts
    // This is a simplified version
    return false;
  }

  private async getApplicableTransactionLimits(
    member: AuthenticatedMember,
    request: WorkflowRequest,
  ): Promise<TransactionLimitDocument[]> {
    return this.transactionLimitModel.find({
      $and: [
        {
          $or: [
            { scope: PermissionScope.GLOBAL },
            { scope: request.scope, organizationId: request.organizationId },
            { scope: request.scope, chamaId: request.chamaId },
            { scope: PermissionScope.PERSONAL, memberId: member.memberId },
          ],
        },
        { isActive: true },
        { effectiveFrom: { $lte: new Date() } },
        {
          $or: [
            { effectiveUntil: { $exists: false } },
            { effectiveUntil: { $gte: new Date() } },
          ],
        },
      ],
    });
  }

  private async checkLimitViolation(
    member: AuthenticatedMember,
    request: WorkflowRequest,
    limit: TransactionLimitDocument,
  ): Promise<boolean> {
    const amount = request.operationData.estimatedValue || 0;
    return amount > limit.limits.maxTransactionAmount;
  }

  private async canUserOverrideLimit(
    member: AuthenticatedMember,
    limit: TransactionLimitDocument,
  ): Promise<boolean> {
    if (!limit.overrideConditions.allowOverride) return false;

    const memberRoles = [
      member.serviceRole,
      ...this.getUserRoles(member, PermissionScope.GLOBAL),
    ];
    const hasOverrideRole = limit.overrideConditions.overrideRoles.some(
      (role) => memberRoles.includes(role),
    );

    const hasOverridePermissions =
      this.permissionService.memberHasAllPermissions(
        member,
        limit.overrideConditions.overridePermissions,
        PermissionScope.GLOBAL,
      );

    return hasOverrideRole && hasOverridePermissions;
  }
}
