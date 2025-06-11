import { Prop, Schema, SchemaFactory } from '@nestjs/mongoose';

export enum ApiKeyStatus {
  ACTIVE = 'active',
  DISABLED = 'disabled',
  EXPIRED = 'expired',
}

@Schema({ timestamps: true })
export class ApiKeyDocument {
  createdAt?: Date;
  updatedAt?: Date;

  @Prop({ required: true, unique: true })
  keyId: string;

  @Prop({ required: true })
  hashedKey: string;

  @Prop({ required: true })
  name: string;

  @Prop()
  description?: string;

  @Prop({ required: true })
  organizationId: string;

  @Prop({ required: true })
  createdBy: string;

  @Prop({ type: [String], required: true })
  serviceIds: string[];

  @Prop({ type: [String], default: [] })
  permissions: string[];

  @Prop({ type: String, enum: ApiKeyStatus, default: ApiKeyStatus.ACTIVE })
  status: ApiKeyStatus;

  @Prop()
  lastUsedAt?: Date;

  @Prop()
  expiresAt?: Date;

  @Prop({ type: Object })
  limits: {
    requestsPerMinute?: number;
    requestsPerDay?: number;
    monthlyVolume?: number;
  };

  @Prop({ type: Object })
  usage: {
    totalRequests: number;
    currentMonth: {
      requests: number;
      volume: number;
      costs: number;
    };
    lastMonth: {
      requests: number;
      volume: number;
      costs: number;
    };
  };

  @Prop({ type: [String], default: [] })
  allowedIps?: string[];

  @Prop({ type: [String], default: [] })
  allowedDomains?: string[];

  @Prop({ type: Object })
  metadata?: Record<string, any>;
}

export const ApiKeySchema = SchemaFactory.createForClass(ApiKeyDocument);
