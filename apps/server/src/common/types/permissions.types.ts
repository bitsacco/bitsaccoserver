/**
 * SACCO-specific role hierarchy with dual-scope permissions
 * Service-level roles: System-wide permissions
 * Group-level roles: Context-specific permissions within organizations/chamas
 */

// Service-level roles (system-wide)
export enum ServiceRole {
  SYSTEM_ADMIN = 'system_admin', // Full configuration access
  ADMIN = 'admin', // Member management, service configuration
  MEMBER = 'member', // Basic service access
}

// Group-level roles (context-specific within organizations/chamas)
export enum GroupRole {
  // Organization-level roles
  ORG_ADMIN = 'org_admin', // Full organization management with elevated privileges
  ORG_MEMBER = 'org_member', // Basic organization participation

  // Chama-level roles
  CHAMA_ADMIN = 'chama_admin', // Full chama management with elevated privileges
  CHAMA_MEMBER = 'chama_member', // Basic chama participation

  // Cross-group roles
  VIEWER = 'viewer', // Read-only access to groups
}

// Permission scopes for context-aware access
export enum PermissionScope {
  GLOBAL = 'global', // System-wide access
  ORGANIZATION = 'organization', // SACCO-level access
  CHAMA = 'chama', // Chama-level access
  PERSONAL = 'personal', // Individual access
}

// Granular permissions for fine-grained access control
export enum Permission {
  // System administration
  SYSTEM_CONFIG = 'system:config',
  SYSTEM_MONITOR = 'system:monitor',
  SYSTEM_BACKUP = 'system:backup',

  // Member management
  USER_CREATE = 'member:create',
  USER_READ = 'member:read',
  USER_UPDATE = 'member:update',
  USER_DELETE = 'member:delete',
  USER_INVITE = 'member:invite',

  // Organization management
  ORG_CREATE = 'org:create',
  ORG_READ = 'org:read',
  ORG_UPDATE = 'org:update',
  ORG_DELETE = 'org:delete',
  ORG_SETTINGS = 'org:settings',

  // Chama management
  CHAMA_CREATE = 'chama:create',
  CHAMA_READ = 'chama:read',
  CHAMA_UPDATE = 'chama:update',
  CHAMA_DELETE = 'chama:delete',
  CHAMA_INVITE = 'chama:invite',

  // Financial operations
  FINANCE_READ = 'finance:read',
  FINANCE_DEPOSIT = 'finance:deposit',
  FINANCE_WITHDRAW = 'finance:withdraw',
  FINANCE_TRANSFER = 'finance:transfer',
  FINANCE_APPROVE = 'finance:approve',

  // Shares management
  SHARES_CREATE = 'shares:create',
  SHARES_READ = 'shares:read',
  SHARES_TRADE = 'shares:trade',
  SHARES_APPROVE = 'shares:approve',

  // Loans management
  LOAN_APPLY = 'loan:apply',
  LOAN_READ = 'loan:read',
  LOAN_APPROVE = 'loan:approve',
  LOAN_DISBURSE = 'loan:disburse',

  // Reports and analytics
  REPORTS_READ = 'reports:read',
  REPORTS_EXPORT = 'reports:export',

  // Governance
  GOVERNANCE_VOTE = 'governance:vote',
  GOVERNANCE_PROPOSE = 'governance:propose',
  GOVERNANCE_MODERATE = 'governance:moderate',
}

export interface GroupMembership {
  groupId: string;
  groupType: 'organization' | 'chama';
  role: GroupRole;
  permissions: Permission[];
  scope: PermissionScope;
  isActive: boolean;
  joinedAt: Date;
  invitedBy?: string;
}

/**
 * Dual-scope permission assignment
 * Users have both service-level and context-specific permissions
 */
export interface UserPermissions {
  serviceRole: ServiceRole;
  servicePermissions: Permission[];

  // Context-specific permissions
  groupMemberships: GroupMembership[];
}

/**
 * Permission matrix for role-based access control
 */
export const ROLE_PERMISSIONS: Record<ServiceRole | GroupRole, Permission[]> = {
  // Service-level role permissions
  [ServiceRole.SYSTEM_ADMIN]: [
    Permission.SYSTEM_CONFIG,
    Permission.SYSTEM_MONITOR,
    Permission.SYSTEM_BACKUP,
    Permission.USER_CREATE,
    Permission.USER_READ,
    Permission.USER_UPDATE,
    Permission.USER_DELETE,
    Permission.ORG_CREATE,
    Permission.ORG_READ,
    Permission.ORG_UPDATE,
    Permission.ORG_DELETE,
    Permission.REPORTS_READ,
    Permission.REPORTS_EXPORT,
  ],

  [ServiceRole.ADMIN]: [
    Permission.USER_CREATE,
    Permission.USER_READ,
    Permission.USER_UPDATE,
    Permission.USER_INVITE,
    Permission.ORG_READ,
    Permission.ORG_UPDATE,
    Permission.ORG_SETTINGS,
    Permission.REPORTS_READ,
  ],

  [ServiceRole.MEMBER]: [
    Permission.USER_READ,
    Permission.ORG_READ,
    Permission.FINANCE_READ,
    Permission.SHARES_READ,
    Permission.LOAN_READ,
  ],

  // Group-level role permissions

  // Organization roles with elevated privileges (subject to maker-checker)
  [GroupRole.ORG_ADMIN]: [
    Permission.ORG_READ,
    Permission.ORG_UPDATE,
    Permission.ORG_DELETE,
    Permission.ORG_SETTINGS,
    Permission.USER_INVITE,
    Permission.USER_UPDATE,
    Permission.CHAMA_CREATE,
    Permission.CHAMA_READ,
    Permission.CHAMA_UPDATE,
    Permission.CHAMA_DELETE,
    Permission.FINANCE_READ,
    Permission.FINANCE_DEPOSIT,
    Permission.FINANCE_WITHDRAW,
    Permission.FINANCE_TRANSFER,
    Permission.FINANCE_APPROVE,
    Permission.SHARES_CREATE,
    Permission.SHARES_READ,
    Permission.SHARES_TRADE,
    Permission.SHARES_APPROVE,
    Permission.LOAN_READ,
    Permission.LOAN_APPLY,
    Permission.LOAN_APPROVE,
    Permission.LOAN_DISBURSE,
    Permission.REPORTS_READ,
    Permission.REPORTS_EXPORT,
    Permission.GOVERNANCE_VOTE,
    Permission.GOVERNANCE_PROPOSE,
    Permission.GOVERNANCE_MODERATE,
  ],

  // Organization basic membership with safe operations
  [GroupRole.ORG_MEMBER]: [
    Permission.ORG_READ,
    Permission.CHAMA_READ,
    Permission.FINANCE_READ,
    Permission.FINANCE_DEPOSIT,
    Permission.SHARES_READ,
    Permission.SHARES_TRADE,
    Permission.LOAN_READ,
    Permission.LOAN_APPLY,
    Permission.REPORTS_READ,
    Permission.GOVERNANCE_VOTE,
  ],

  // Chama roles with elevated privileges (subject to maker-checker)
  [GroupRole.CHAMA_ADMIN]: [
    Permission.CHAMA_READ,
    Permission.CHAMA_UPDATE,
    Permission.CHAMA_DELETE,
    Permission.CHAMA_INVITE,
    Permission.USER_UPDATE,
    Permission.FINANCE_READ,
    Permission.FINANCE_DEPOSIT,
    Permission.FINANCE_WITHDRAW,
    Permission.FINANCE_TRANSFER,
    Permission.FINANCE_APPROVE,
    Permission.SHARES_READ,
    Permission.SHARES_TRADE,
    Permission.SHARES_APPROVE,
    Permission.LOAN_READ,
    Permission.LOAN_APPLY,
    Permission.LOAN_APPROVE,
    Permission.LOAN_DISBURSE,
    Permission.REPORTS_READ,
    Permission.REPORTS_EXPORT,
    Permission.GOVERNANCE_VOTE,
    Permission.GOVERNANCE_PROPOSE,
    Permission.GOVERNANCE_MODERATE,
  ],

  // Chama basic membership with safe operations
  [GroupRole.CHAMA_MEMBER]: [
    Permission.CHAMA_READ,
    Permission.FINANCE_READ,
    Permission.FINANCE_DEPOSIT,
    Permission.SHARES_READ,
    Permission.SHARES_TRADE,
    Permission.LOAN_READ,
    Permission.LOAN_APPLY,
    Permission.REPORTS_READ,
    Permission.GOVERNANCE_VOTE,
  ],

  // Cross-group read-only access
  [GroupRole.VIEWER]: [
    Permission.ORG_READ,
    Permission.CHAMA_READ,
    Permission.FINANCE_READ,
    Permission.SHARES_READ,
    Permission.LOAN_READ,
    Permission.REPORTS_READ,
  ],
};

/**
 * Permission inheritance rules
 * Higher roles inherit permissions from lower roles within the same scope
 */
export const ROLE_HIERARCHY: Record<
  ServiceRole | GroupRole,
  (ServiceRole | GroupRole)[]
> = {
  // Service-level hierarchy
  [ServiceRole.SYSTEM_ADMIN]: [ServiceRole.ADMIN, ServiceRole.MEMBER],
  [ServiceRole.ADMIN]: [ServiceRole.MEMBER],
  [ServiceRole.MEMBER]: [],

  // Organization-level hierarchy
  [GroupRole.ORG_ADMIN]: [GroupRole.ORG_MEMBER, GroupRole.VIEWER],
  [GroupRole.ORG_MEMBER]: [GroupRole.VIEWER],

  // Chama-level hierarchy
  [GroupRole.CHAMA_ADMIN]: [GroupRole.CHAMA_MEMBER, GroupRole.VIEWER],
  [GroupRole.CHAMA_MEMBER]: [GroupRole.VIEWER],

  // Cross-group roles
  [GroupRole.VIEWER]: [],
};

/**
 * Maker-Checker Configuration for Elevated Privileges
 * Defines which operations require approval workflows for admin roles
 */
export interface MakerCheckerConfig {
  // Financial operation thresholds
  financialThresholds: {
    withdrawalLimit: number; // Amount above which withdrawal requires approval
    transferLimit: number; // Amount above which transfer requires approval
    loanApprovalLimit: number; // Loan amount requiring multiple approvals
  };

  // Administrative operation thresholds
  adminThresholds: {
    memberInviteLimit: number; // Number of invites requiring approval
    organizationSettingsChange: boolean; // Whether org settings changes need approval
    chamaCreationLimit: number; // Number of chamas that can be created without approval
  };

  // Approval requirements
  approvalRequirements: {
    minimumApprovers: number; // Minimum number of approvers required
    sameLevelApproval: boolean; // Whether approvers must have same or higher role level
    timeoutHours: number; // Hours before approval request expires
    allowSelfApproval: boolean; // Whether initiator can approve their own request
  };
}

/**
 * Default Maker-Checker Configuration
 * Conservative defaults for financial oversight
 */
export const DEFAULT_MAKER_CHECKER_CONFIG: MakerCheckerConfig = {
  financialThresholds: {
    withdrawalLimit: 100000, // 100,000 KES
    transferLimit: 50000, // 50,000 KES
    loanApprovalLimit: 500000, // 500,000 KES
  },
  adminThresholds: {
    memberInviteLimit: 5, // More than 5 invites need approval
    organizationSettingsChange: true, // Always require approval for settings
    chamaCreationLimit: 2, // More than 2 chamas need approval
  },
  approvalRequirements: {
    minimumApprovers: 2, // Require 2 approvers
    sameLevelApproval: true, // Approvers must be same level or higher
    timeoutHours: 24, // 24-hour approval window
    allowSelfApproval: false, // No self-approval allowed
  },
};
