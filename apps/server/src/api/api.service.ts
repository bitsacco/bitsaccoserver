import { Injectable } from '@nestjs/common';

@Injectable()
export class ApiService {
  getHealth(): object {
    return {
      status: 'ok',
      timestamp: new Date().toISOString(),
      service: 'bitsaccoserver-bitsaccoserver',
      version: '1.0.0',
    };
  }

  getInfo(): object {
    return {
      name: 'Bitsaccoserver API',
      description:
        'Comprehensive SACCO (Savings and Credit Cooperative) management platform with multi-tenant organization support, financial operations, compliance, and integrated services',
      version: '1.0.0',
      documentation: '/api/docs',
      features: [
        'Multi-tenant Organization Management',
        'SACCO Financial Operations (Deposits, Withdrawals, Loans)',
        'Shares Management & Trading',
        'Compliance & Risk Management',
        'Maker-Checker Workflows',
        'API Key Lifecycle Management',
        'Role-based Access Control',
        'SMS Services Integration',
        'System Administration & Monitoring',
        'Audit Logging & Regulatory Reporting',
        'Real-time Rate Limiting',
        'Unified Authentication (JWT + API Keys)',
      ],
      endpoints: {
        // Core Services
        health: '/api/v1/health',
        info: '/api/v1/',

        // Authentication & Security
        auth: {
          base: '/api/v1/auth',
          login: '/api/v1/auth/login',
          register: '/api/v1/auth/register',
          validate: '/api/v1/auth/validate',
          refresh: '/api/v1/auth/refresh',
          logout: '/api/v1/auth/logout',
        },

        // Organization Management
        organizations: {
          base: '/api/v1/organizations',
          details: '/api/v1/organizations/{id}',
          members: '/api/v1/organizations/{id}/members',
          apiKeys: '/api/v1/organizations/{id}/api-keys',
          services: '/api/v1/organizations/{id}/services',
          usage: '/api/v1/organizations/{id}/usage',
          billing: '/api/v1/organizations/{id}/billing',
        },

        // SACCO Operations
        sacco: {
          balance: '/api/v1/sacco/balance',
          deposits: '/api/v1/sacco/{scope}/{id}/deposit',
          withdrawals: '/api/v1/sacco/{scope}/{id}/withdraw',
          transfers: '/api/v1/sacco/transfer',
          loans: '/api/v1/sacco/loans',
          loanApplication: '/api/v1/sacco/{scope}/{id}/loans/apply',
          shares: '/api/v1/sacco/organization/{id}/shares',
          sharesPurchase: '/api/v1/sacco/organization/{id}/shares/purchase',
          chamaManagement: '/api/v1/sacco/organization/{id}/chamas',
          reports: '/api/v1/sacco/{scope}/{id}/reports/{type}',
        },

        // Shares Management
        shares: {
          base: '/api/v1/shares',
          offers: '/api/v1/shares/offers',
          subscribe: '/api/v1/shares/subscribe',
          transfer: '/api/v1/shares/transfer',
          transactions: '/api/v1/shares/transactions',
        },

        // Compliance & Risk
        compliance: {
          workflows: '/api/v1/compliance/workflows',
          pending: '/api/v1/compliance/workflows/pending',
          segregation: '/api/v1/compliance/sod',
          riskAssessment: '/api/v1/compliance/risk/assess',
          auditLogs: '/api/v1/compliance/audit',
          reports: '/api/v1/compliance/reports',
        },

        // SMS Services
        sms: {
          sendMessage: '/api/v1/sms/send-message',
          sendBulk: '/api/v1/sms/send-bulk-message',
        },

        // System Administration
        admin: {
          health: '/api/v1/admin/health',
          config: '/api/v1/admin/config',
          integrations: '/api/v1/admin/integrations',
          telemetry: '/api/v1/admin/telemetry',
          metrics: '/api/v1/admin/metrics',
          members: '/api/v1/admin/members',
          backup: '/api/v1/admin/backup',
          auditLogs: '/api/v1/admin/audit-logs',
        },
      },

      // Authentication Methods
      authentication: {
        jwt: {
          description: 'JWT Bearer token authentication for members',
          header: 'Authorization: Bearer <token>',
          obtain: 'POST /api/v1/auth/login',
        },
        apiKey: {
          description: 'API key authentication for service-to-service calls',
          header: 'x-api-key: <api-key>',
          obtain: 'POST /api/v1/organizations/{id}/api-keys',
        },
      },

      // Access Control
      accessControl: {
        roles: ['SYSTEM_ADMIN', 'ADMIN', 'MANAGER', 'USER'],
        groupRoles: [
          'ORG_ADMIN',
          'ORG_MEMBER',
          'CHAMA_ADMIN',
          'CHAMA_MEMBER',
          'VIEWER',
        ],
        scopes: ['GLOBAL', 'ORGANIZATION', 'CHAMA', 'PERSONAL'],
        permissions: [
          'FINANCE_READ',
          'FINANCE_DEPOSIT',
          'FINANCE_WITHDRAW',
          'FINANCE_TRANSFER',
          'SHARES_READ',
          'SHARES_TRADE',
          'SHARES_CREATE',
          'LOAN_READ',
          'LOAN_APPLY',
          'ORG_READ',
          'USER_INVITE',
          'REPORTS_READ',
          'SYSTEM_CONFIG',
        ],
      },

      // Compliance Features
      compliance: {
        makerChecker:
          'Approval workflows for financial operations with configurable thresholds',
        segregationOfDuties: 'Role-based operation restrictions',
        riskManagement: 'Transaction risk assessment and limits',
        auditTrail: 'Comprehensive audit logging',
        regulatoryReporting: 'Automated compliance reporting',
      },

      // Role-Based Access Control Details
      roleManagement: {
        description:
          'Simplified role hierarchy with maker-checker for elevated privileges',
        organizationRoles: {
          ORG_ADMIN: {
            description:
              'Full organization management with elevated privileges',
            features: [
              'Complete financial oversight',
              'Member management',
              'Chama creation and management',
              'Settings configuration',
              'All operations subject to maker-checker approval',
            ],
            makerCheckerRequired: true,
          },
          ORG_MEMBER: {
            description: 'Basic organization participation',
            features: [
              'Read access to organization data',
              'Safe transactional operations',
              'Participation in governance',
              'Standard deposits and shares trading',
            ],
            makerCheckerRequired: false,
          },
        },
        chamaRoles: {
          CHAMA_ADMIN: {
            description: 'Full chama management with elevated privileges',
            features: [
              'Complete chama financial oversight',
              'Member management within chama',
              'Settings configuration',
              'Loan approval and disbursement',
              'All elevated operations subject to maker-checker approval',
            ],
            makerCheckerRequired: true,
          },
          CHAMA_MEMBER: {
            description: 'Basic chama participation',
            features: [
              'Read access to chama data',
              'Safe transactional operations',
              'Loan applications',
              'Standard deposits and contributions',
            ],
            makerCheckerRequired: false,
          },
        },
        crossGroupRoles: {
          VIEWER: {
            description: 'Read-only access to groups',
            features: [
              'View organization and chama data',
              'Access to reports and analytics',
              'No write permissions',
              'Audit and compliance oversight',
            ],
            makerCheckerRequired: false,
          },
        },
        makerCheckerConfig: {
          financialThresholds: {
            withdrawalLimit: '100,000 KES',
            transferLimit: '50,000 KES',
            loanApprovalLimit: '500,000 KES',
          },
          approvalRequirements: {
            minimumApprovers: 2,
            timeoutHours: 24,
            allowSelfApproval: false,
          },
        },
      },
    };
  }
}
