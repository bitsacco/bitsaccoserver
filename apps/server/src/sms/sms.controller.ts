import {
  ApiOperation,
  ApiBody,
  ApiBearerAuth,
  ApiCookieAuth,
  ApiTags,
} from '@nestjs/swagger';
import { Body, Controller, Logger, Post, UseGuards } from '@nestjs/common';
import { SendBulkSmsDto, SendSmsDto, UnifiedAuthGuard } from '@/common';
import { SmsService } from './sms.service';

@ApiTags('sms')
@ApiBearerAuth()
@UseGuards(UnifiedAuthGuard)
@Controller('organizations')
export class SmsController {
  private readonly logger = new Logger(SmsController.name);

  constructor(private readonly smsService: SmsService) {
    this.logger.log('SmsController initialized');
  }

  @Post('send-message')
  @ApiBearerAuth()
  @ApiCookieAuth()
  @ApiOperation({ summary: 'Send a single sms' })
  @ApiBody({
    type: SendSmsDto,
  })
  configureSmsRelays(@Body() req: SendSmsDto) {
    return this.smsService.sendSms(req);
  }

  @Post('send-bulk-message')
  @ApiBearerAuth()
  @ApiCookieAuth()
  @ApiOperation({ summary: 'Send multiple sms' })
  @ApiBody({
    type: SendBulkSmsDto,
  })
  send(@Body() req: SendBulkSmsDto) {
    return this.smsService.sendBulkSms(req);
  }
}
