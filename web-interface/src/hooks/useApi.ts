import { useState, useCallback } from 'react';
import { ApiPromise, WsProvider } from '@polkadot/api';

let apiInstance: ApiPromise | null = null;

export function useApi() {
  const [api, setApi] = useState<ApiPromise | null>(null);
  const [isConnected, setIsConnected] = useState(false);
  const [isConnecting, setIsConnecting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const connect = useCallback(async (wsEndpoint: string = 'ws://127.0.0.1:9944') => {
    if (apiInstance && apiInstance.isConnected) {
      setApi(apiInstance);
      setIsConnected(true);
      return apiInstance;
    }

    setIsConnecting(true);
    setError(null);

    try {
      const provider = new WsProvider(wsEndpoint);
      const newApi = await ApiPromise.create({ provider });

      await newApi.isReady;

      apiInstance = newApi;
      setApi(newApi);
      setIsConnected(true);
      setIsConnecting(false);

      newApi.on('connected', () => setIsConnected(true));
      newApi.on('disconnected', () => setIsConnected(false));
      newApi.on('error', (err) => setError(err.toString()));

      return newApi;
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Failed to connect';
      setError(message);
      setIsConnecting(false);
      setIsConnected(false);
      throw new Error(message);
    }
  }, []);

  const disconnect = useCallback(async () => {
    if (apiInstance) {
      await apiInstance.disconnect();
      apiInstance = null;
      setApi(null);
      setIsConnected(false);
    }
  }, []);

  return {
    api,
    isConnected,
    isConnecting,
    error,
    connect,
    disconnect,
  };
}