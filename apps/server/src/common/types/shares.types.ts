// Shares domain types

export interface SharesMetricData {
  memberId: string;
  offerId?: string;
  quantity: number;
  success: boolean;
  duration: number;
  errorType?: string;
}

export interface SharesOwnershipMetricData {
  memberId: string;
  quantity: number;
  percentageOfTotal: number;
  limitReached: boolean;
}

export interface SharesTransferData {
  fromMemberId: string;
  toMemberId: string;
  quantity: number;
  success: boolean;
  duration: number;
  errorType?: string;
}
