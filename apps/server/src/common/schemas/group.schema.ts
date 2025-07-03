import { Prop, Schema, SchemaFactory } from '@nestjs/mongoose';
import { GroupRole } from '../types';

/**
 * Enhanced Group Member Schema with dual-scope membership
 * Supports membership in both Organization and Chamas
 */
@Schema({ timestamps: true })
export class GroupMemberDocument {
  @Prop({ required: true })
  memberId: string;

  @Prop({ required: true })
  email: string;

  @Prop()
  phoneNumber?: string;

  @Prop()
  firstName?: string;

  @Prop()
  lastName?: string;

  // Service-level role (system-wide)
  @Prop({
    type: String,
    enum: GroupRole,
    default: GroupRole.GROUP_MEMBER,
  })
  groupRole: GroupRole;

  // Member profile and KYC information
  @Prop({ type: Object })
  profile: {
    username?: string;
    residentialAddress?: string;
    nextOfKin?: {
      name: string;
      relationship: string;
      phoneNumber: string;
    };
  };

  // KYC status and documentation
  @Prop({ type: Object })
  kyc: {
    status: 'pending' | 'verified' | 'rejected';
    documentType?: 'national_id' | 'passport' | 'driving_license';
    documentNumber?: string;
    documentExpiryDate?: Date;
    verifiedAt?: Date;
    verifiedBy?: string;
    rejectionReason?: string;
  };

  @Prop({ default: true })
  isActive: boolean;

  @Prop({ default: false })
  isEmailVerified: boolean;

  @Prop({ default: false })
  isPhoneVerified: boolean;

  // Member preferences and settings
  @Prop({ type: Object })
  preferences: {
    language: string;
    timezone: string;
    notificationChannels: ('sms' | 'email' | 'push' | 'whatsapp')[];
    privacySettings: {
      shareProfileWithChama: boolean;
      shareContactWithMembers: boolean;
      allowDirectMessages: boolean;
    };
  };

  // Financial overview (aggregated across all memberships)
  @Prop({ type: Object })
  financialSummary: {
    totalShares: number;
    totalSavings: number;
    totalLoans: number;
    totalContributions: number;
    creditScore?: number;
    lastTransactionDate?: Date;
  };
}

export const GroupMemberSchema =
  SchemaFactory.createForClass(GroupMemberDocument);

/**
 * Cross-group relationships and interactions
 */
@Schema({ timestamps: true })
export class GroupRelationshipDocument {
  @Prop({ required: true })
  parentGroupId: string;

  @Prop({ required: true })
  childGroupId: string;

  @Prop({
    type: String,
    enum: ['parent-child', 'affiliate', 'partner'],
    required: true,
  })
  relationshipType: string;

  @Prop({ type: Object })
  relationshipDetails: {
    establishedDate: Date;
    establishedBy: string;
    terms?: string;
    benefits?: string[];
    obligations?: string[];
    reviewDate?: Date;
  };

  @Prop({ default: true })
  isActive: boolean;

  // Shared services and integrations
  @Prop([String])
  sharedServices: string[];

  @Prop({ type: Object })
  financialArrangements?: {
    feeSharing?: number; // percentage
    resourceSharing?: boolean;
    jointPrograms?: string[];
  };
}

export const GroupRelationshipSchema = SchemaFactory.createForClass(
  GroupRelationshipDocument,
);
