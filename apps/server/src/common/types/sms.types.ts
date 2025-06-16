// SMS service types and DTOs
// Note: SendSmsDto and SendBulkSmsDto moved to dto folder to avoid conflicts

export interface SmsMetricData {
  receiver: string;
  messageLength: number;
  success: boolean;
  duration: number;
  errorType?: string;
}

export interface SmsBulkMetricData {
  receiverCount: number;
  messageLength: number;
  success: boolean;
  duration: number;
  errorType?: string;
}
