import { Prop, Schema, SchemaFactory } from '@nestjs/mongoose';
import { Document } from 'mongoose';

@Schema({ timestamps: true, versionKey: false })
export class SharesOfferDocument extends Document {
  @Prop({ type: Number, required: true })
  quantity: number;

  @Prop({ type: Number, required: true, default: 0 })
  subscribedQuantity: number;

  @Prop({ type: Date, required: true, default: Date.now })
  availableFrom: Date;

  @Prop({ type: Date, required: false })
  availableTo?: Date;

  createdAt: Date;
  updatedAt: Date;
}

export const SharesOfferSchema =
  SchemaFactory.createForClass(SharesOfferDocument);
