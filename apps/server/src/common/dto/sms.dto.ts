import { Type } from 'class-transformer';
import { IsString, IsNotEmpty } from 'class-validator';
import { ApiProperty } from '@nestjs/swagger';
import { SendBulkSmsRequest, SendSmsRequest } from '@bitsaccoserver/types';

export class SendSmsDto implements SendSmsRequest {
  @IsNotEmpty()
  @IsString()
  @Type(() => String)
  @ApiProperty({ example: 'hello bitsacco' })
  message: string;

  @IsNotEmpty()
  @IsString()
  @Type(() => String)
  @ApiProperty()
  receiver: string;
}

export class SendBulkSmsDto implements SendBulkSmsRequest {
  @IsNotEmpty()
  @IsString()
  @Type(() => String)
  @ApiProperty()
  message: string;

  @IsNotEmpty()
  @IsString({ each: true })
  @Type(() => String)
  @ApiProperty({ type: [String] })
  receivers: string[];
}
