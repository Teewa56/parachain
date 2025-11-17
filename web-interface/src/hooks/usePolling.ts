import { useState, useEffect, useCallback, useRef } from 'react';

interface PollingOptions<T> {
  fetchFn: () => Promise<T>;
  interval: number;
  enabled?: boolean;
  onSuccess?: (data: T) => void;
  onError?: (error: Error) => void;
}

export function usePolling<T>({
  fetchFn,
  interval,
  enabled = true,
  onSuccess,
  onError,
}: PollingOptions<T>) {
  const [data, setData] = useState<T | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<Error | null>(null);
  const timeoutRef = useRef<NodeJS.Timeout>();
  const mountedRef = useRef(true);

  const poll = useCallback(async () => {
    if (!enabled || !mountedRef.current) return;

    try {
      setLoading(true);
      const result = await fetchFn();
      
      if (mountedRef.current) {
        setData(result);
        setError(null);
        onSuccess?.(result);
      }
    } catch (err) {
      const error = err instanceof Error ? err : new Error('Polling failed');
      if (mountedRef.current) {
        setError(error);
        onError?.(error);
      }
    } finally {
      if (mountedRef.current) {
        setLoading(false);
      }
    }
  }, [fetchFn, enabled, onSuccess, onError]);

  const startPolling = useCallback(() => {
    const runPoll = async () => {
      await poll();
      if (mountedRef.current && enabled) {
        timeoutRef.current = setTimeout(runPoll, interval);
      }
    };
    runPoll();
  }, [poll, interval, enabled]);

  const stopPolling = useCallback(() => {
    if (timeoutRef.current) {
      clearTimeout(timeoutRef.current);
    }
  }, []);

  useEffect(() => {
    mountedRef.current = true;
    if (enabled) {
      startPolling();
    }

    return () => {
      mountedRef.current = false;
      stopPolling();
    };
  }, [enabled, startPolling, stopPolling]);

  return {
    data,
    loading,
    error,
    refetch: poll,
    startPolling,
    stopPolling,
  };
}