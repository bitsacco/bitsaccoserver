import { Document } from 'mongoose';
import { Prop, Schema, SchemaFactory } from '@nestjs/mongoose';

export enum MemberStatus {
  PENDING = 'pending',
  ACTIVE = 'active',
  SUSPENDED = 'suspended',
}

@Schema({ timestamps: true })
export class Member {
  @Prop({ required: true, unique: true })
  email: string;

  @Prop({ required: true })
  keycloakId: string;

  @Prop()
  firstName?: string;

  @Prop()
  lastName?: string;

  @Prop({ type: String, enum: MemberStatus, default: MemberStatus.PENDING })
  status: MemberStatus;

  @Prop()
  emailVerifiedAt?: Date;

  @Prop()
  lastLoginAt?: Date;

  @Prop({ type: Object })
  profile?: {
    avatar?: string;
    timezone?: string;
    language?: string;
  };
}

export type MemberDocument = Member & Document;
export const MemberSchema = SchemaFactory.createForClass(Member);
