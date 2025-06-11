import { Prop, Schema, SchemaFactory } from '@nestjs/mongoose';
import { ServiceStatus } from '../types';

@Schema({ timestamps: true })
export class ServiceDocument {
  @Prop({ required: true, unique: true })
  name: string;

  @Prop({ required: true })
  displayName: string;

  @Prop()
  description?: string;

  @Prop({ type: String, enum: ServiceStatus, default: ServiceStatus.ACTIVE })
  status: ServiceStatus;

  @Prop({ required: true })
  baseUrl: string;

  @Prop({ type: [String], default: [] })
  availablePermissions: string[];

  @Prop({ type: Object })
  pricing: {
    model: 'per_request' | 'per_volume' | 'subscription';
    rates: Record<string, number>;
    currency: string;
  };

  @Prop({ type: Object })
  defaultLimits: {
    requestsPerMinute: number;
    requestsPerDay: number;
    monthlyVolume?: number;
  };

  @Prop({ type: Object })
  metadata?: Record<string, any>;
}

@Schema({ timestamps: true })
export class OrganizationServiceDocument {
  @Prop({ required: true })
  organizationId: string;

  @Prop({ required: true })
  serviceId: string;

  @Prop({ default: true })
  isEnabled: boolean;

  @Prop()
  enabledAt: Date;

  @Prop({ type: Object })
  customLimits?: {
    requestsPerMinute?: number;
    requestsPerDay?: number;
    monthlyVolume?: number;
  };

  @Prop({ type: Object })
  billing: {
    balance: number;
    currency: string;
    autoRecharge?: {
      enabled: boolean;
      threshold: number;
      amount: number;
    };
  };

  @Prop({ type: Object })
  usage: {
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
}

export const ServiceSchema = SchemaFactory.createForClass(ServiceDocument);
export const OrganizationServiceSchema = SchemaFactory.createForClass(
  OrganizationServiceDocument,
);
