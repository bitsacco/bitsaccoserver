import { Document } from 'mongoose';
import { Prop, Schema, SchemaFactory } from '@nestjs/mongoose';

export enum UserStatus {
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

  @Prop({ type: String, enum: UserStatus, default: UserStatus.PENDING })
  status: UserStatus;

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

export type UserDocument = Member & Document;
export const UserSchema = SchemaFactory.createForClass(Member);
