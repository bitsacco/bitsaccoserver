// Workflow, approval, and compliance business logic types

export interface WorkflowRequest {
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
}

export interface ApprovalRequest {
  workflowId: string;
  status: ApprovalStatus.APPROVED | ApprovalStatus.REJECTED;
  comment?: string;
  ipAddress?: string;
  userAgent?: string;
}

// These enums are now exported from schemas/compliance.schema.ts
// Remove duplicates to avoid conflicts

export interface ComplianceEventData {
  eventType: string;
  severity: RiskLevel;
  description: string;
  userId?: string;
  organizationId?: string;
  chamaId?: string;
  scope: PermissionScope;
  sourceSystem?: string;
  correlationId?: string;
  metadata?: Record<string, any>;
  detectionMethod?: 'automatic' | 'manual' | 'external';
  regulatoryRequirement?: string[];
}

export interface ComplianceMetrics {
  organizationId: string;
  period: 'daily' | 'weekly' | 'monthly' | 'quarterly' | 'yearly';
  totalEvents: number;
  eventsByType: Record<string, number>;
  eventsBySeverity: Record<RiskLevel, number>;
  resolutionRate: number;
  averageResolutionTime: number; // in hours
  pendingEvents: number;
  criticalEventsCount: number;
  complianceScore: number; // 0-100
  trendsFromPreviousPeriod: {
    totalEventsChange: number;
    complianceScoreChange: number;
    criticalEventsChange: number;
  };
}

// Import dependencies
import { RiskLevel } from './compliance.types';
import { PermissionScope } from './permissions.types';
import { WorkflowType, ApprovalStatus } from '../schemas/compliance.schema';
