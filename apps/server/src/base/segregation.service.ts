import {
  Injectable,
  BadRequestException,
  ForbiddenException,
} from '@nestjs/common';
import { Model } from 'mongoose';
import { EventEmitter2 } from '@nestjs/event-emitter';
import { InjectModel } from '@nestjs/mongoose';
import {
  AuditService,
  GroupRole,
  SegregationRule,
  SegregationRuleDocument,
  AuthenticatedMember,
  ServiceRole,
  Permission,
  PermissionScope,
  RiskLevel,
} from '../common';

export interface SoDViolation {
  ruleId: string;
  ruleName: string;
  description: string;
  conflictType: 'same_member' | 'same_role' | 'same_session' | 'time_window';
  operation1: {
    memberId: string;
    action: string;
    timestamp: Date;
    sessionId?: string;
  };
  operation2: {
    memberId: string;
    action: string;
    timestamp: Date;
    sessionId?: string;
  };
  severity: 'low' | 'medium' | 'high' | 'critical';
  autoBlocked: boolean;
  requiresApproval: boolean;
}

export interface OperationContext {
  memberId: string;
  action: string;
  permissions: Permission[];
  roles: (ServiceRole | GroupRole)[];
  scope: PermissionScope;
  organizationId?: string;
  chamaId?: string;
  sessionId?: string;
  timestamp: Date;
  metadata?: Record<string, any>;
}

/**
 * Segregation of Duties Service
 * Implements dual control mechanisms and prevents conflicts of interest
 */
@Injectable()
export class SegregationService {
  private operationHistory: Map<string, OperationContext[]> = new Map();

  constructor(
    @InjectModel(SegregationRule.name)
    private segregationRuleModel: Model<SegregationRuleDocument>,
    private auditService: AuditService,
    private eventEmitter: EventEmitter2,
  ) {
    // Initialize default SoD rules
    this.initializeDefaultRules();
  }

  /**
   * Check if an operation violates segregation of duties
   */
  async checkSegregationViolation(
    member: AuthenticatedMember,
    operationContext: Omit<OperationContext, 'timestamp'>,
  ): Promise<SoDViolation[]> {
    const context: OperationContext = {
      ...operationContext,
      timestamp: new Date(),
    };

    // Get applicable SoD rules
    const applicableRules = await this.getApplicableRules(context);
    const violations: SoDViolation[] = [];

    for (const rule of applicableRules) {
      const violation = await this.checkRuleViolation(context, rule);
      if (violation) {
        violations.push(violation);

        // Log the violation
        await this.auditService.logAuditEvent({
          memberId: context.memberId,
          action: 'SOD_VIOLATION_DETECTED',
          resourceType: 'segregation_rule',
          resourceId: rule._id.toString(),
          scope: context.scope,
          organizationId: context.organizationId,
          chamaId: context.chamaId,
          complianceContext: {
            riskLevel: RiskLevel.HIGH,
            sensitiveData: true,
            approvalRequired: true,
          },
          businessContext: {
            businessJustification: `SoD violation: ${rule.description}`,
          },
          timestamp: new Date(),
        });

        // Emit violation event
        this.eventEmitter.emit('sod.violation_detected', violation);
      }
    }

    // Store operation in history for future checks
    this.storeOperationHistory(context);

    return violations;
  }

  /**
   * Create a new segregation rule
   */
  async createSegregationRule(
    member: AuthenticatedMember,
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
  ): Promise<SegregationRuleDocument> {
    // Validate member permissions (only SYSTEM_ADMIN can create SoD rules)
    if (member.serviceRole !== ServiceRole.SYSTEM_ADMIN) {
      throw new ForbiddenException(
        'Only SYSTEM_ADMIN can create segregation rules',
      );
    }

    const rule = new this.segregationRuleModel({
      ...ruleData,
      isActive: true,
      createdBy: member.memberId,
      lastModifiedBy: member.memberId,
    });

    const savedRule = await rule.save();

    await this.auditService.logAuditEvent({
      memberId: member.memberId,
      action: 'SOD_RULE_CREATED',
      resourceType: 'segregation_rule',
      resourceId: savedRule._id.toString(),
      scope: PermissionScope.GLOBAL,
      timestamp: new Date(),
    });

    return savedRule;
  }

  /**
   * Update segregation rule
   */
  async updateSegregationRule(
    member: AuthenticatedMember,
    ruleId: string,
    updateData: Partial<SegregationRule>,
  ): Promise<SegregationRuleDocument> {
    if (member.serviceRole !== ServiceRole.SYSTEM_ADMIN) {
      throw new ForbiddenException(
        'Only SYSTEM_ADMIN can update segregation rules',
      );
    }

    const rule = await this.segregationRuleModel.findByIdAndUpdate(
      ruleId,
      { ...updateData, lastModifiedBy: member.memberId },
      { new: true, runValidators: true },
    );

    if (!rule) {
      throw new BadRequestException('Segregation rule not found');
    }

    await this.auditService.logAuditEvent({
      memberId: member.memberId,
      action: 'SOD_RULE_UPDATED',
      resourceType: 'segregation_rule',
      resourceId: ruleId,
      scope: PermissionScope.GLOBAL,
      timestamp: new Date(),
    });

    return rule;
  }

  /**
   * Get all segregation rules
   */
  async getSegregationRules(
    member: AuthenticatedMember,
    scope?: PermissionScope,
    isActive?: boolean,
  ): Promise<SegregationRuleDocument[]> {
    // ADMIN and above can view SoD rules
    if (
      member.serviceRole !== ServiceRole.SYSTEM_ADMIN &&
      member.serviceRole !== ServiceRole.ADMIN
    ) {
      throw new ForbiddenException(
        'Insufficient permissions to view segregation rules',
      );
    }

    const query: any = {};
    if (scope) query.scope = scope;
    if (isActive !== undefined) query.isActive = isActive;

    return this.segregationRuleModel.find(query).sort({ createdAt: -1 });
  }

  /**
   * Activate/deactivate segregation rule
   */
  async toggleSegregationRule(
    member: AuthenticatedMember,
    ruleId: string,
    isActive: boolean,
  ): Promise<SegregationRuleDocument> {
    if (member.serviceRole !== ServiceRole.SYSTEM_ADMIN) {
      throw new ForbiddenException(
        'Only SYSTEM_ADMIN can toggle segregation rules',
      );
    }

    const rule = await this.segregationRuleModel.findByIdAndUpdate(
      ruleId,
      { isActive, lastModifiedBy: member.memberId },
      { new: true },
    );

    if (!rule) {
      throw new BadRequestException('Segregation rule not found');
    }

    await this.auditService.logAuditEvent({
      memberId: member.memberId,
      action: isActive ? 'SOD_RULE_ACTIVATED' : 'SOD_RULE_DEACTIVATED',
      resourceType: 'segregation_rule',
      resourceId: ruleId,
      scope: PermissionScope.GLOBAL,
      timestamp: new Date(),
    });

    return rule;
  }

  /**
   * Get segregation violations for a specific time period
   */
  async getViolationReport(
    member: AuthenticatedMember,
    startDate: Date,
    endDate: Date,
    scope?: PermissionScope,
    organizationId?: string,
    chamaId?: string,
  ): Promise<{
    violations: any[];
    summary: {
      totalViolations: number;
      blockedOperations: number;
      approvalRequired: number;
      violationsByRule: Record<string, number>;
      violationsBySeverity: Record<string, number>;
    };
  }> {
    // Implementation would query audit logs for SoD violations
    const { logs: violations } = await this.auditService.getAuditLogs({
      action: 'SOD_VIOLATION_DETECTED',
      startDate,
      endDate,
      scope,
      organizationId,
      chamaId,
    });

    const summary = this.generateViolationSummary(violations);

    return { violations, summary };
  }

  /**
   * Analyze dual control effectiveness
   */
  async analyzeDualControlEffectiveness(
    scope: PermissionScope,
    organizationId?: string,
    chamaId?: string,
    days: number = 30,
  ): Promise<{
    totalOperations: number;
    dualControlledOperations: number;
    effectivenessRate: number;
    riskMitigated: number;
    recommendations: string[];
  }> {
    const endDate = new Date();
    const _startDate = new Date(endDate.getTime() - days * 24 * 60 * 60 * 1000);

    // Analyze operations that went through dual control
    const analysis = {
      totalOperations: 1000, // Mock data
      dualControlledOperations: 850,
      effectivenessRate: 85,
      riskMitigated: 15,
      recommendations: [
        'Implement mandatory dual control for transactions above $10,000',
        'Add time-based segregation for sensitive configuration changes',
        'Require approval for cross-chama transfers',
      ],
    };

    return analysis;
  }

  // Private Methods

  private async initializeDefaultRules(): Promise<void> {
    const defaultRules = [
      {
        ruleName: 'Financial Authorization Segregation',
        description:
          'Member who initiates a financial transaction cannot approve it',
        scope: PermissionScope.ORGANIZATION,
        conflictingOperations: {
          operation1: {
            action: 'financial_transaction_initiate',
            permissions: [
              Permission.FINANCE_DEPOSIT,
              Permission.FINANCE_WITHDRAW,
            ],
            roles: [GroupRole.CHAMA_MEMBER],
          },
          operation2: {
            action: 'financial_transaction_approve',
            permissions: [Permission.FINANCE_APPROVE],
            roles: [GroupRole.CHAMA_ADMIN, GroupRole.ORG_ADMIN],
          },
          conflictType: 'same_member' as const,
        },
        enforcement: {
          blockConflicting: true,
          requireApproval: false,
          alertLevel: 'critical' as const,
          notificationChannels: ['email', 'dashboard', 'audit'] as const,
        },
        isActive: true,
        createdBy: 'system',
        lastModifiedBy: 'system',
      },
      {
        ruleName: 'Member Management Segregation',
        description:
          'Member creation and role assignment must be done by different members',
        scope: PermissionScope.ORGANIZATION,
        conflictingOperations: {
          operation1: {
            action: 'member_create',
            permissions: [Permission.USER_CREATE],
            roles: [ServiceRole.ADMIN, GroupRole.ORG_ADMIN],
          },
          operation2: {
            action: 'member_role_assign',
            permissions: [Permission.USER_UPDATE],
            roles: [ServiceRole.ADMIN, GroupRole.ORG_ADMIN],
          },
          conflictType: 'same_member' as const,
        },
        enforcement: {
          blockConflicting: false,
          requireApproval: true,
          alertLevel: 'warning' as const,
          notificationChannels: ['email', 'dashboard'] as const,
        },
        isActive: true,
        createdBy: 'system',
        lastModifiedBy: 'system',
      },
      {
        ruleName: 'Loan Processing Segregation',
        description:
          'Loan application and approval must be done by different members within 24 hours',
        scope: PermissionScope.CHAMA,
        conflictingOperations: {
          operation1: {
            action: 'loan_apply',
            permissions: [Permission.LOAN_APPLY],
            roles: [GroupRole.CHAMA_MEMBER],
          },
          operation2: {
            action: 'loan_approve',
            permissions: [Permission.LOAN_APPROVE],
            roles: [GroupRole.CHAMA_ADMIN],
          },
          conflictType: 'time_window' as const,
          timeWindowHours: 24,
        },
        enforcement: {
          blockConflicting: true,
          requireApproval: false,
          alertLevel: 'critical' as const,
          notificationChannels: ['email', 'sms', 'dashboard', 'audit'] as const,
        },
        isActive: true,
        createdBy: 'system',
        lastModifiedBy: 'system',
      },
    ];

    // Check if rules already exist before creating
    for (const ruleData of defaultRules) {
      const existingRule = await this.segregationRuleModel.findOne({
        ruleName: ruleData.ruleName,
      });

      if (!existingRule) {
        await this.segregationRuleModel.create(ruleData);
      }
    }
  }

  private async getApplicableRules(
    context: OperationContext,
  ): Promise<SegregationRuleDocument[]> {
    return this.segregationRuleModel.find({
      isActive: true,
      scope: { $in: [context.scope, PermissionScope.GLOBAL] },
      $or: [
        { 'conflictingOperations.operation1.action': context.action },
        { 'conflictingOperations.operation2.action': context.action },
      ],
    });
  }

  private async checkRuleViolation(
    context: OperationContext,
    rule: SegregationRuleDocument,
  ): Promise<SoDViolation | null> {
    const { conflictingOperations } = rule;

    // Determine which operation this context represents
    const isOperation1 =
      conflictingOperations.operation1.action === context.action;
    const conflictingOperation = isOperation1
      ? conflictingOperations.operation2
      : conflictingOperations.operation1;

    // Get relevant operation history
    const historyKey = this.getHistoryKey(context);
    const history = this.operationHistory.get(historyKey) || [];

    // Look for conflicting operations
    const conflictingOps = history.filter(
      (op) =>
        op.action === conflictingOperation.action &&
        this.checkConflictCondition(context, op, rule),
    );

    if (conflictingOps.length > 0) {
      const conflictingOp = conflictingOps[0];

      return {
        ruleId: rule._id.toString(),
        ruleName: rule.ruleName,
        description: rule.description,
        conflictType: conflictingOperations.conflictType,
        operation1: isOperation1
          ? {
              memberId: context.memberId,
              action: context.action,
              timestamp: context.timestamp,
              sessionId: context.sessionId,
            }
          : {
              memberId: conflictingOp.memberId,
              action: conflictingOp.action,
              timestamp: conflictingOp.timestamp,
              sessionId: conflictingOp.sessionId,
            },
        operation2: isOperation1
          ? {
              memberId: conflictingOp.memberId,
              action: conflictingOp.action,
              timestamp: conflictingOp.timestamp,
              sessionId: conflictingOp.sessionId,
            }
          : {
              memberId: context.memberId,
              action: context.action,
              timestamp: context.timestamp,
              sessionId: context.sessionId,
            },
        severity: this.calculateViolationSeverity(rule),
        autoBlocked: rule.enforcement.blockConflicting,
        requiresApproval: rule.enforcement.requireApproval,
      };
    }

    return null;
  }

  private checkConflictCondition(
    currentOp: OperationContext,
    historicalOp: OperationContext,
    rule: SegregationRuleDocument,
  ): boolean {
    const { conflictType, timeWindowHours } = rule.conflictingOperations;

    switch (conflictType) {
      case 'same_member':
        return currentOp.memberId === historicalOp.memberId;

      case 'same_role':
        return currentOp.roles.some((role) =>
          historicalOp.roles.includes(role),
        );

      case 'same_session':
        return currentOp.sessionId === historicalOp.sessionId;

      case 'time_window':
        if (!timeWindowHours) return false;
        const timeDiff =
          currentOp.timestamp.getTime() - historicalOp.timestamp.getTime();
        const windowMs = timeWindowHours * 60 * 60 * 1000;
        return timeDiff <= windowMs;

      default:
        return false;
    }
  }

  private calculateViolationSeverity(
    rule: SegregationRuleDocument,
  ): 'low' | 'medium' | 'high' | 'critical' {
    if (rule.enforcement.alertLevel === 'critical') return 'critical';
    if (rule.enforcement.blockConflicting) return 'high';
    if (rule.enforcement.requireApproval) return 'medium';
    return 'low';
  }

  private getHistoryKey(context: OperationContext): string {
    // Create a key based on scope and organization/chama
    let key: string = context.scope;
    if (context.organizationId) key += `:org:${context.organizationId}`;
    if (context.chamaId) key += `:chama:${context.chamaId}`;
    return key;
  }

  private storeOperationHistory(context: OperationContext): void {
    const historyKey = this.getHistoryKey(context);
    const history = this.operationHistory.get(historyKey) || [];

    // Add current operation
    history.push(context);

    // Keep only last 1000 operations per scope
    if (history.length > 1000) {
      history.shift();
    }

    // Clean up operations older than 30 days
    const thirtyDaysAgo = new Date(Date.now() - 30 * 24 * 60 * 60 * 1000);
    const recentHistory = history.filter((op) => op.timestamp > thirtyDaysAgo);

    this.operationHistory.set(historyKey, recentHistory);
  }

  private generateViolationSummary(violations: any[]): any {
    const summary = {
      totalViolations: violations.length,
      blockedOperations: 0,
      approvalRequired: 0,
      violationsByRule: {} as Record<string, number>,
      violationsBySeverity: {
        low: 0,
        medium: 0,
        high: 0,
        critical: 0,
      },
    };

    violations.forEach((violation) => {
      if (violation.autoBlocked) summary.blockedOperations++;
      if (violation.requiresApproval) summary.approvalRequired++;

      const ruleName = violation.ruleName || 'unknown';
      summary.violationsByRule[ruleName] =
        (summary.violationsByRule[ruleName] || 0) + 1;

      summary.violationsBySeverity[violation.severity]++;
    });

    return summary;
  }
}
