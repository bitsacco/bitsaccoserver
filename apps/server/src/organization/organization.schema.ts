import { Prop, Schema, SchemaFactory } from '@nestjs/mongoose';

/**
 * Enhanced SACCO Schema
 * Supports hierarchical structure with flexible organizational formations
 */
@Schema({ timestamps: true })
export class OrganizationDocument {
  @Prop({ required: true, unique: true })
  name: string;

  @Prop({ required: true })
  country: string;

  @Prop()
  description?: string;

  @Prop({ required: true })
  ownerId: string;

  @Prop({ required: true })
  ownerEmail: string;

  @Prop({
    type: String,
    enum: ['sacco', 'cooperative', 'chama_federation'],
    default: 'sacco',
  })
  organizationType: string;

  // Enhanced KYB details for SACCO compliance
  @Prop({ type: Object })
  kybDetails?: {
    businessRegistrationNumber?: string;
    taxId?: string;
    businessAddress?: string;
    businessType?: string;
    regulatoryLicense?: string;
    registrationDate?: Date;
    status?: 'pending' | 'verified' | 'rejected';
    verifiedAt?: Date;
    verifiedBy?: string;
  };

  // Organization-specific governance configuration
  @Prop({ type: Object })
  governance: {
    transactionRules: {
      quorumPercentage: number;
      majorityThreshold: number;
      allowProxyVoting: boolean;
    };
    membershipRules: {
      invitationByAdminOnly: number;
      probationaryPeriod?: number; // in days
      autoApproval: boolean;
    };
  };

  // Chama treasury and financial management
  @Prop({ type: Object })
  financialConfig: {
    balance: number;
    currency: string;
    shareValue: number;
    loanPolicy: {
      maximumLoanMultiplier: number; // multiple of shares
      defaultInterestRate: number;
      maximumLoanTerm: number; // in months
      collateralRequirement: boolean;
    };
  };

  @Prop({ default: true })
  isActive: boolean;

  // Enhanced limits and quotas
  @Prop({ type: Object })
  limits: {
    maxMembers?: number;
    maxChamas?: number;
    maxApiKeys: number;
    maxMonthlyVolume: number;
    maxDailyRequests: number;
    maxLoanAmount?: number;
  };

  // Integration and notification settings
  @Prop({ type: Object })
  settings: {
    allowedDomains?: string[];
    webhookUrl?: string;
    notificationEmail?: string;
    smsNotifications: boolean;
    emailNotifications: boolean;
    autoApproveMembers: boolean;
  };

  // Compliance and audit trail
  @Prop({ type: Object })
  compliance: {
    regulatoryReporting: boolean;
    auditTrail: boolean;
    dataRetentionPeriod: number; // in days
    lastAuditDate?: Date;
    nextAuditDate?: Date;
    complianceScore?: number;
  };
}

export const OrganizationSchema =
  SchemaFactory.createForClass(OrganizationDocument);
