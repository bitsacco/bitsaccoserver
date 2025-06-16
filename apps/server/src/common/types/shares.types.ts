// Shares domain types

export interface SharesMetricData {
  userId: string;
  offerId?: string;
  quantity: number;
  success: boolean;
  duration: number;
  errorType?: string;
}

export interface SharesOwnershipMetricData {
  userId: string;
  quantity: number;
  percentageOfTotal: number;
  limitReached: boolean;
}

export interface SharesTransferData {
  fromUserId: string;
  toUserId: string;
  quantity: number;
  success: boolean;
  duration: number;
  errorType?: string;
}
