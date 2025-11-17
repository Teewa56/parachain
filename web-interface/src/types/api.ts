export interface ApiResponse<T = any> {
  success: boolean;
  data?: T;
  error?: string;
  message?: string;
}

export interface PaginatedResponse<T> {
  items: T[];
  total: number;
  page: number;
  pageSize: number;
  hasMore: boolean;
}

export interface BlockInfo {
  number: number;
  hash: string;
  timestamp: number;
  parentHash: string;
  extrinsicsRoot: string;
}

export interface TransactionResult {
  hash: string;
  blockNumber: number;
  blockHash: string;
  success: boolean;
  events: any[];
}
