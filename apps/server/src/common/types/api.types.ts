// API Request/Response DTOs and related types
// Note: PaginationDto moved to dto folder to avoid conflicts

export interface ApiResponse<T = any> {
  success: boolean;
  data?: T;
  message?: string;
  errors?: string[];
  meta?: {
    total?: number;
    page?: number;
    limit?: number;
    totalPages?: number;
  };
}

export interface ErrorResponse {
  success: false;
  message: string;
  errors?: string[];
  statusCode: number;
  timestamp: string;
  path: string;
}
