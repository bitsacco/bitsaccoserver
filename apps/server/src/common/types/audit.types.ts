// Audit and monitoring types
import { AuthenticatedUser } from './auth.types';
import { PermissionScope, Permission } from './permissions.types';
import { RiskLevel } from './compliance.types';

export interface AuditEventData {
  userId: string;
  impersonatedBy?: string;
  action: string;
  resourceType: string;
  resourceId?: string;
  scope: PermissionScope;
  organizationId?: string;
  chamaId?: string;
  requestData?: {
    method: string;
    endpoint: string;
    parameters?: Record<string, any>;
    body?: any;
    userAgent: string;
    ipAddress: string;
    sessionId?: string;
  };
  responseData?: {
    statusCode: number;
    success: boolean;
    error?: string;
    responseTime: number;
  };
  businessContext?: {
    workflowId?: string;
    transactionId?: string;
    amount?: number;
    currency?: string;
    approvalRequired?: boolean;
    transactionType?: string;
    businessJustification?: string;
  };
  securityContext?: {
    authMethod: 'jwt' | 'api-key';
    permissions: string[];
    riskScore?: number;
    anomalyFlags?: string[];
  };
  complianceContext?: {
    riskLevel?: RiskLevel;
    sensitiveData?: boolean;
    approvalRequired?: boolean;
    dataClassification?: string;
  };
  timestamp: Date;
  correlationId?: string;
  tags?: string[];
}

export interface AuditQueryFilters {
  userId?: string;
  organizationId?: string;
  chamaId?: string;
  action?: string;
  resourceType?: string;
  fromDate?: Date;
  toDate?: Date;
  startDate?: Date;
  endDate?: Date;
  scope?: PermissionScope;
  success?: boolean;
  riskLevel?: RiskLevel;
  sensitiveData?: boolean;
  page?: number;
  limit?: number;
  offset?: number;
  sortBy?: string;
  sortOrder?: 'asc' | 'desc';
}

export interface ServiceContext {
  userId: string;
  organizationId?: string;
  chamaId?: string;
  scope: PermissionScope;
  permissions: string[];
  user: AuthenticatedUser;
  correlationId?: string;
  requestId?: string;
  timestamp?: Date;
}

export interface ServiceOperation {
  name: string;
  description: string;
  requiredPermissions: Permission[];
  allowedScopes: PermissionScope[];
  riskLevel: RiskLevel;
  auditLevel: 'none' | 'basic' | 'detailed' | 'comprehensive';
  requiresApproval?: boolean;
  timeoutMs?: number;
  retryPolicy?: {
    maxRetries: number;
    backoffMs: number;
  };
}

export interface ServiceResult<T = any> {
  success: boolean;
  data?: T;
  error?: {
    code: string;
    message: string;
    details?: any;
  };
  context?: ServiceContext;
  metadata?: {
    executionTimeMs: number;
    resourcesUsed?: string[];
    auditEventId?: string;
    correlationId: string;
  };
}

export interface ApiMetricData {
  endpoint: string;
  method: string;
  statusCode: number;
  responseTime: number;
  duration: number;
  requestSize?: number;
  responseSize?: number;
  clientIp?: string;
  success: boolean;
  userId?: string;
  organizationId?: string;
  userAgent?: string;
  ipAddress?: string;
  timestamp: Date;
  apiKeyId?: string;
}
