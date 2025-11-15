/**
 * API Request wrapper
 */
export interface ApiRequest<T = any> {
    endpoint?: string;
    params?: T;
    headers?: Record<string, string>;
    timeout?: number;
}

/**
 * API Response wrapper
 */
export interface ApiResponse<T = any> {
    success: boolean;
    data?: T;
    error?: ApiError;
    metadata?: ResponseMetadata;
}

/**
 * API Error structure
 */
export interface ApiError {
    code: string;
    message: string;
    details?: any;
    stack?: string;
    timestamp?: number;
}

/**
 * Response metadata
 */
export interface ResponseMetadata {
    timestamp: number;
    requestId?: string;
    duration?: number;
    blockNumber?: number;
}

/**
 * Pagination parameters
 */
export interface PaginationParams {
    page: number;
    pageSize: number;
    sortBy?: string;
    sortOrder?: 'asc' | 'desc';
}

/**
 * Paginated response
 */
export interface PaginatedResponse<T> {
    items: T[];
    total: number;
    page: number;
    pageSize: number;
    totalPages: number;
    hasMore: boolean;
}

/**
 * API call options
 */
export interface ApiCallOptions {
    retry?: boolean;
    retryAttempts?: number;
    retryDelay?: number;
    timeout?: number;
    signal?: AbortSignal;
}

/**
 * Blockchain API specific types
 */
export interface BlockchainApiConfig {
    endpoint: string;
    paraId: number;
    networkName: string;
    reconnectAttempts?: number;
    reconnectDelay?: number;
    timeout?: number;
}

/**
 * Transaction status
 */
export enum TransactionStatus {
    PENDING = 'pending',
    IN_BLOCK = 'in_block',
    FINALIZED = 'finalized',
    FAILED = 'failed',
    DROPPED = 'dropped',
}

/**
 * Transaction receipt
 */
export interface TransactionReceipt {
    hash: string;
    blockHash?: string;
    blockNumber?: number;
    status: TransactionStatus;
    timestamp: number;
    events?: TransactionEvent[];
    error?: string;
}

/**
 * Transaction event
 */
export interface TransactionEvent {
    section: string;
    method: string;
    data: any[];
    index: number;
}

/**
 * WebSocket message types
 */
export enum WebSocketMessageType {
    SUBSCRIBE = 'subscribe',
    UNSUBSCRIBE = 'unsubscribe',
    DATA = 'data',
    ERROR = 'error',
    CONNECTED = 'connected',
    DISCONNECTED = 'disconnected',
}

/**
 * WebSocket message
 */
export interface WebSocketMessage {
    type: WebSocketMessageType;
    channel?: string;
    data?: any;
    error?: string;
    timestamp: number;
}

/**
 * Rate limit info
 */
export interface RateLimitInfo {
    limit: number;
    remaining: number;
    reset: number;
}

/**
 * API health status
 */
export interface ApiHealthStatus {
    status: 'healthy' | 'degraded' | 'down';
    latency: number;
    uptime: number;
    version?: string;
    lastCheck: number;
}