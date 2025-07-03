import { Prop, Schema, SchemaFactory } from '@nestjs/mongoose';

/**
 * Chamas - Sub-groups within Bitsacco
 * Bitsaccoserver supports both independent chamas and organization-affiliated chamas
 */
@Schema({ timestamps: true })
export class ChamaDocument {
  @Prop({ required: true })
  name: string;

  @Prop()
  description?: string;

  @Prop({ required: true })
  leaderId: string;

  @Prop()
  parentOrganizationId?: string; // Optional - for organization affiliated chamas

  @Prop({
    type: String,
    enum: ['independent', 'org_affiliated'],
    default: 'independent',
  })
  chamaType: string;

  // Chama-specific governance
  @Prop({ type: Object })
  governance: {
    leadership: {
      leader?: string;
      treasurer?: string;
      secretary?: string;
    };
    meetingSchedule: {
      frequency: 'weekly' | 'bi-weekly' | 'monthly';
      dayOfWeek?: number; // 0-6 (Sunday-Saturday)
      timeOfDay?: string; // HH:MM format
      nextMeetingDate?: Date;
    };
    contributionRules: {
      minimumContribution: number;
      contributionFrequency: 'weekly' | 'monthly';
      penaltyRate?: number; // for late contributions
      gracePeriod?: number; // in days
    };
    membershipRules: {
      maximumMembers?: number;
      inviteOnly: boolean;
      approvalRequired: boolean;
      probationaryPeriod?: number; // in days
    };
  };

  // Chama treasury and financial management
  @Prop({ type: Object })
  treasury: {
    balance: number;
    currency: string;
    monthlyTarget?: number;
    contributionHistory: {
      totalContributed: number;
      averageMonthlyContribution: number;
      lastContributionDate?: Date;
    };
    loanFund: {
      totalFund: number;
      availableFund: number;
      outstandingLoans: number;
    };
    savingsAccount?: {
      accountNumber?: string;
      bankName?: string;
      accountBalance: number;
    };
  };

  @Prop({ default: true })
  isActive: boolean;

  // Chama activity and performance metrics
  @Prop({ type: Object })
  metrics: {
    memberCount: number;
    averageAttendance: number;
    contributionComplianceRate: number;
    loanDefaultRate: number;
    monthlyGrowthRate: number;
    lastActivityDate?: Date;
  };

  // External integrations (mobile money, banking)
  @Prop({ type: Object })
  integrations: {
    mobileMoney?: {
      provider: 'mpesa' | 'airtel' | 'mtn';
      accountNumber?: string;
      isActive: boolean;
    };
    banking?: {
      bankName?: string;
      accountNumber?: string;
      isActive: boolean;
    };
  };
}

export const ChamaSchema = SchemaFactory.createForClass(ChamaDocument);
