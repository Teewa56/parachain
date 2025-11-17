export class SubstrateConnection {
  private static instance: SubstrateConnection;
  private wsEndpoint: string = import.meta.env.VITE_PARACHAIN_WS || 'ws://localhost:9944';

  private constructor() {}

  static getInstance(): SubstrateConnection {
    if (!SubstrateConnection.instance) {
      SubstrateConnection.instance = new SubstrateConnection();
    }
    return SubstrateConnection.instance;
  }

  async connect() {
    // add functionality to use @polkadot/api
    return { connected: true, endpoint: this.wsEndpoint };
  }

  getEndpoint(): string {
    return this.wsEndpoint;
  }
}
