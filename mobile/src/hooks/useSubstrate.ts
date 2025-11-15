import { useState, useEffect } from 'react';
import { substrateAPI } from '../substrate/api';
import { ConnectionStatus } from '../types/substrate';

export const useSubstrate = () => {
    const [status, setStatus] = useState<ConnectionStatus>(ConnectionStatus.DISCONNECTED);

    useEffect(() => {
        const unsubscribe = substrateAPI.onStatusChange(setStatus);
        return unsubscribe;
    }, []);

    return {
        isConnected: substrateAPI.isConnected(),
        status,
        api: substrateAPI.getApi.bind(substrateAPI),
    };
};