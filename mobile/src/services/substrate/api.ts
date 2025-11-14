import { ApiPromise, WsProvider } from '@polkadot/api';
import type { ApiOptions } from '@polkadot/api/types';
import { ConnectionStatus, type Unsubscribe } from '../../types/substrate';
import { CONNECTION_CONFIG, CUSTOM_TYPES } from '../../config/substrate';

class SubstrateAPIService {
    private api: ApiPromise | null = null;
    private wsProvider: WsProvider | null = null;
    private connectionStatus: ConnectionStatus = ConnectionStatus.DISCONNECTED;
    private reconnectAttempts = 0;
    private reconnectTimer: NodeJS.Timeout | null = null;
    private statusListeners: Set<(status: ConnectionStatus) => void> = new Set();

    /**
     * Connect to the Substrate node
     */
    async connect(endpoint: string): Promise<void> {
        if (this.connectionStatus === ConnectionStatus.CONNECTING) {
        throw new Error('Connection already in progress');
        }

        if (this.connectionStatus === ConnectionStatus.CONNECTED && this.api) {
        console.log('Already connected to', endpoint);
        return;
        }

        this.setConnectionStatus(ConnectionStatus.CONNECTING);

        try {
            this.wsProvider = new WsProvider(endpoint, false);

            this.setupProviderListeners();

            const apiOptions: ApiOptions = {
                provider: this.wsProvider,
                types: CUSTOM_TYPES,
                throwOnConnect: true,
            };

            this.api = await ApiPromise.create(apiOptions);
            await this.api.isReady;

            this.setConnectionStatus(ConnectionStatus.CONNECTED);
            this.reconnectAttempts = 0;

            console.log('✅ Connected to parachain:', endpoint);
            console.log('Chain:', (await this.api.rpc.system.chain()).toString());
            console.log('Node version:', (await this.api.rpc.system.version()).toString());
        } catch (error) {
            console.error('❌ Connection failed:', error);
            this.setConnectionStatus(ConnectionStatus.ERROR);
            this.scheduleReconnect(endpoint);
            throw error;
        }
    }

    /**
     * Disconnect from the node
     */
    async disconnect(): Promise<void> {
        if (this.reconnectTimer) {
            clearTimeout(this.reconnectTimer);
            this.reconnectTimer = null;
            }

            if (this.api) {
            try {
                await this.api.disconnect();
                this.api = null;
            } catch (error) {
                console.error('Error disconnecting:', error);
            }
            }

            if (this.wsProvider) {
            try {
                await this.wsProvider.disconnect();
                this.wsProvider = null;
            } catch (error) {
                console.error('Error disconnecting provider:', error);
            }
        }

        this.setConnectionStatus(ConnectionStatus.DISCONNECTED);
        console.log('Disconnected from parachain');
    }

    /**
     * Get the API instance
     */
    getApi(): ApiPromise {
        if (!this.api) {
        throw new Error('API not initialized. Call connect() first.');
        }
        if (!this.isConnected()) {
        throw new Error('API not connected');
        }
        return this.api;
    }

    /**
     * Check if connected
     */
    isConnected(): boolean {
        return this.connectionStatus === ConnectionStatus.CONNECTED && this.api !== null;
    }

    /**
     * Get current connection status
     */
    getConnectionStatus(): ConnectionStatus {
        return this.connectionStatus;
    }

    /**
     * Subscribe to connection status changes
     */
    onStatusChange(callback: (status: ConnectionStatus) => void): Unsubscribe {
        this.statusListeners.add(callback);
        callback(this.connectionStatus);
        return () => {
            this.statusListeners.delete(callback);
        };
    }

    /**
     * Subscribe to new blocks
     */
    async subscribeToBlocks(
        callback: (blockNumber: number, blockHash: string) => void
    ): Promise<Unsubscribe> {
        const api = this.getApi();

        const unsubscribe = await api.rpc.chain.subscribeNewHeads((header) => {
            const blockNumber = header.number.toNumber();
            const blockHash = header.hash.toString();
            callback(blockNumber, blockHash);
        });

        return unsubscribe;
    }

    /**
     * Subscribe to finalized blocks
     */
    async subscribeToFinalizedBlocks(
        callback: (blockNumber: number, blockHash: string) => void
    ): Promise<Unsubscribe> {
        const api = this.getApi();

        const unsubscribe = await api.rpc.chain.subscribeFinalizedHeads((header) => {
            const blockNumber = header.number.toNumber();
            const blockHash = header.hash.toString();
            callback(blockNumber, blockHash);
        });

        return unsubscribe;
    }

    /**
     * Get current block number
     */
    async getCurrentBlockNumber(): Promise<number> {
        const api = this.getApi();
        const header = await api.rpc.chain.getHeader();
        return header.number.toNumber();
    }

    /**
     * Get chain properties
     */
    async getChainProperties() {
        const api = this.getApi();
        const [chain, nodeName, nodeVersion] = await Promise.all([
            api.rpc.system.chain(),
            api.rpc.system.name(),
            api.rpc.system.version(),
        ]);

        return {
            chain: chain.toString(),
            nodeName: nodeName.toString(),
            nodeVersion: nodeVersion.toString(),
        };
    }

    /**
     * Setup provider event listeners
     */
    private setupProviderListeners(): void {
        if (!this.wsProvider) return;

        this.wsProvider.on('connected', () => {
            console.log('WebSocket connected');
        });

        this.wsProvider.on('disconnected', () => {
            console.log('WebSocket disconnected');
            this.setConnectionStatus(ConnectionStatus.DISCONNECTED);
        });

        this.wsProvider.on('error', (error) => {
            console.error('WebSocket error:', error);
            this.setConnectionStatus(ConnectionStatus.ERROR);
        });
    }

    /**
     * Schedule reconnection attempt
     */
    private scheduleReconnect(endpoint: string): void {
        if (this.reconnectAttempts >= CONNECTION_CONFIG.reconnectAttempts) {
        console.error('Max reconnection attempts reached');
        return;
        }

        this.reconnectAttempts++;
        const delay = CONNECTION_CONFIG.reconnectDelay * this.reconnectAttempts;

        console.log(
        `Scheduling reconnect attempt ${this.reconnectAttempts}/${CONNECTION_CONFIG.reconnectAttempts} in ${delay}ms`
        );

        this.reconnectTimer = setTimeout(() => {
            console.log('Attempting to reconnect...');
            this.connect(endpoint).catch((error) => {
                console.error('Reconnection failed:', error);
            });
        }, delay);
    }

    /**
     * Set connection status and notify listeners
     */
    private setConnectionStatus(status: ConnectionStatus): void {
        this.connectionStatus = status;
        this.statusListeners.forEach((listener) => {
        try {
            listener(status);
        } catch (error) {
            console.error('Error in status listener:', error);
        }
        });
    }
}

export const substrateAPI = new SubstrateAPIService();