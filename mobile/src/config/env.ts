import Constants from 'expo-constants';

// Get environment from Expo config or default to development
const ENV = Constants.expoConfig?.extra?.env || 'development';

interface Config {
    env: string;
    wsEndpoint: string;
    paraId: number;
    networkName: string;
    apiTimeout: number;
    enableLogging: boolean;
}

const development: Config = {
    env: 'development',
    wsEndpoint: 'ws://127.0.0.1:9944',
    paraId: 1000,
    networkName: 'Local Development',
    apiTimeout: 30000,
    enableLogging: true,
};

const testnet: Config = {
    env: 'testnet',
    wsEndpoint: 'wss://rococo-parachain-testnet.example.com',
    paraId: 1000,
    networkName: 'Rococo Testnet',
    apiTimeout: 30000,
    enableLogging: true,
};

const production: Config = {
    env: 'production',
    wsEndpoint: 'wss://identity-parachain.polkadot.network',
    paraId: 2000,
    networkName: 'Identity Parachain',
    apiTimeout: 30000,
    enableLogging: false,
};

const configs: Record<string, Config> = {
    development,
    testnet,
    production,
};

// Export current config
export const config = configs[ENV] || development;

// Export all configs for network switching
export const allConfigs = configs;

// Export individual values for convenience
export const { wsEndpoint, paraId, networkName, apiTimeout, enableLogging } = config;

// App version from package.json
export const APP_VERSION = Constants.expoConfig?.version || '1.0.0';

// Platform info
export const IS_IOS = Constants.platform?.ios !== undefined;
export const IS_ANDROID = Constants.platform?.android !== undefined;
export const PLATFORM = IS_IOS ? 'ios' : IS_ANDROID ? 'android' : 'web';

// Feature flags
export const FEATURES = {
    BIOMETRIC_AUTH: true,
    QR_CODE_SHARING: true,
    MULTI_IDENTITY: false, // Coming soon
    CROSS_CHAIN: false, // Coming soon
    PUSH_NOTIFICATIONS: false, // Coming soon
};

// Debug mode
export const DEBUG = __DEV__ || enableLogging;

// Log configuration on startup
if (DEBUG) {
    console.log('Environment Configuration:');
    console.log(`Environment: ${config.env}`);
    console.log(`Network: ${config.networkName}`);
    console.log(`Endpoint: ${config.wsEndpoint}`);
    console.log(`Platform: ${PLATFORM}`);
    console.log(`App Version: ${APP_VERSION}`);
}