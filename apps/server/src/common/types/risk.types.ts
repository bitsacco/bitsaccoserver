// Risk management and assessment types

export interface RiskAssessment {
  transactionId?: string;
  userId: string;
  organizationId: string;
  chamaId?: string;
  operationType: string;
  amount?: number;
  currency?: string;
  riskScore: number; // 0-100
  riskLevel: RiskLevel;
  factors: RiskFactor[];
  mitigationRecommendations: string[];
  assessmentTimestamp: Date;
  validUntil?: Date;
  assessorId?: string;
  automaticFlags: string[];
  manualOverride?: {
    reason: string;
    overrideBy: string;
    timestamp: Date;
  };
}

export interface RiskFactor {
  category: 'transaction' | 'user' | 'location' | 'temporal' | 'behavioral';
  name: string;
  description: string;
  weight: number; // 0-1
  value: number; // 0-100
  evidence?: any;
}

export interface TransactionRisk {
  amount: number;
  frequency: number;
  unusualPattern: boolean;
  velocityCheck: boolean;
  locationRisk: number;
  timeRisk: number;
  counterpartyRisk?: number;
  aggregateRisk: number;
}

export interface LimitViolation {
  limitType: 'daily' | 'weekly' | 'monthly' | 'transaction' | 'count';
  limitValue: number;
  currentValue: number;
  violationAmount: number;
  userId: string;
  organizationId: string;
  chamaId?: string;
  transactionId?: string;
  timestamp: Date;
  severity: RiskLevel;
  requiresApproval: boolean;
  approvalWorkflowId?: string;
}

export interface SoDViolation {
  ruleId: string;
  ruleName: string;
  description: string;
  severity: RiskLevel;
  userId: string;
  organizationId: string;
  chamaId?: string;
  operationType: string;
  conflictingRoles: string[];
  previousActions: string[];
  timestamp: Date;
  requiresApproval: boolean;
  exemptionReason?: string;
  approvedBy?: string;
}

export interface OperationContext {
  userId: string;
  organizationId: string;
  chamaId?: string;
  operationType: string;
  operationData: any;
  timestamp: Date;
  ipAddress?: string;
  userAgent?: string;
  sessionId?: string;
  requestId: string;
}

// Import dependencies
import { RiskLevel } from './compliance.types';
