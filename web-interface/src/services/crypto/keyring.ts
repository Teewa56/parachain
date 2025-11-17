import { Keyring } from '@polkadot/keyring';
import { cryptoWaitReady } from '@polkadot/util-crypto';

class KeyringService {
  private keyring: Keyring | null = null;

  async initialize() {
    await cryptoWaitReady();
    this.keyring = new Keyring({ type: 'sr25519' });
    return this.keyring;
  }

  getKeyring(): Keyring {
    if (!this.keyring) {
      throw new Error('Keyring not initialized');
    }
    return this.keyring;
  }

  addFromUri(uri: string, meta?: Record<string, any>) {
    const keyring = this.getKeyring();
    return keyring.addFromUri(uri, meta);
  }

  addFromMnemonic(mnemonic: string, meta?: Record<string, any>) {
    const keyring = this.getKeyring();
    return keyring.addFromMnemonic(mnemonic, meta);
  }

  generateMnemonic(): string {
    const { mnemonicGenerate } = require('@polkadot/util-crypto');
    return mnemonicGenerate();
  }

  getPairs() {
    const keyring = this.getKeyring();
    return keyring.getPairs();
  }

  getPair(address: string) {
    const keyring = this.getKeyring();
    return keyring.getPair(address);
  }
}

export const keyringService = new KeyringService();
