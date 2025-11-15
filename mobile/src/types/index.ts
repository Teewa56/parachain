export * from './substrate';
export * from './api';

// Common utility types
export type Nullable<T> = T | null;
export type Optional<T> = T | undefined;
export type AsyncResult<T> = Promise<T>;
export type Timestamp = number;
export type HexString = string;
export type Base64String = string;

// App-wide enums
export enum LoadingState {
  Idle = 'idle',
  Loading = 'loading',
  Success = 'success',
  Error = 'error',
}

export enum Theme {
  Light = 'light',
  Dark = 'dark',
  System = 'system',
}

export enum Language {
  English = 'en',
  Spanish = 'es',
  French = 'fr',
  German = 'de',
}

// Navigation types
export type RouteParams = {
  [key: string]: string | number | boolean | undefined;
};

export type ScreenName = 
  | '/(auth)/login'
  | '/(auth)/register'
  | '/(auth)/recovery'
  | '/(wallet)'
  | '/(wallet)/credentials'
  | '/(wallet)/credentials/[id]'
  | '/(wallet)/identity/create'
  | '/(wallet)/proof'
  | '/(wallet)/settings';

// Error types
export interface AppError {
  code: string;
  message: string;
  details?: any;
}

// Response wrapper
export interface ApiResponse<T> {
  success: boolean;
  data?: T;
  error?: AppError;
}

// Pagination
export interface PaginationParams {
  page: number;
  pageSize: number;
}

export interface PaginatedResponse<T> {
  items: T[];
  total: number;
  page: number;
  pageSize: number;
  hasMore: boolean;
}

// Filter types
export interface FilterOptions {
  sortBy?: string;
  sortOrder?: 'asc' | 'desc';
  search?: string;
}

// Export utility type guards
export const isError = (value: any): value is AppError => {
  return value && typeof value.code === 'string' && typeof value.message === 'string';
};

export const isApiResponse = <T>(value: any): value is ApiResponse<T> => {
  return value && typeof value.success === 'boolean';
};