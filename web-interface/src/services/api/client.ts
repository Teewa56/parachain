import { ApiPromise } from '@polkadot/api';

class ApiClient {
    private api: ApiPromise | null = null;
    private baseUrl: string;

    constructor(baseUrl: string = '') {
        this.baseUrl = baseUrl;
    }

    setApi(api: ApiPromise) {
        this.api = api;
    }

    getApi(): ApiPromise {
        if (!this.api) {
        throw new Error('API not initialized');
        }
        return this.api;
    }

    async get<T>(endpoint: string): Promise<T> {
        const response = await fetch(`${this.baseUrl}${endpoint}`);
        if (!response.ok) {
        throw new Error(`API error: ${response.statusText}`);
        }
        return response.json();
    }

    async post<T>(endpoint: string, data: any): Promise<T> {
        const response = await fetch(`${this.baseUrl}${endpoint}`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(data),
        });
        if (!response.ok) {
            throw new Error(`API error: ${response.statusText}`);
        }
        return response.json();
    }
}

export const apiClient = new ApiClient();