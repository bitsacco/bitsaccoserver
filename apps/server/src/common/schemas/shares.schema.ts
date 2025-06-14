import { Prop, Schema, SchemaFactory } from '@nestjs/mongoose';
import { Document } from 'mongoose';

export enum SharesTxStatus {
  UNRECOGNIZED = 0,
  PROPOSED = 1,
  PROCESSING = 2,
  APPROVED = 3,
  COMPLETE = 4,
  FAILED = 5,
}

export interface SharesTxTransferMeta {
  fromUserId: string;
  toUserId: string;
  quantity: number;
  reason?: string;
}

@Schema({ timestamps: true, versionKey: false })
export class SharesDocument extends Document {
  @Prop({ type: String, required: true })
  userId: string;

  @Prop({ type: String, required: true })
  offerId: string;

  @Prop({
    type: String,
    required: true,
    enum: Object.values(SharesTxStatus),
    default: SharesTxStatus.PROPOSED,
  })
  status: SharesTxStatus;

  @Prop({
    type: Object,
    required: false,
  })
  transfer?: SharesTxTransferMeta;

  @Prop({ type: Number, required: true })
  quantity: number;

  createdAt: Date;
  updatedAt: Date;
}

export const SharesSchema = SchemaFactory.createForClass(SharesDocument);
