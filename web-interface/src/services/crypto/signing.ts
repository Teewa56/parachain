import { u8aToHex, hexToU8a } from '@polkadot/util';
import { signatureVerify } from '@polkadot/util-crypto';

export const signingService = {
  async sign(message: string | Uint8Array, keyPair: any): Promise<string> {
    const messageU8a = typeof message === 'string' 
      ? new TextEncoder().encode(message)
      : message;
    
    const signature = keyPair.sign(messageU8a);
    return u8aToHex(signature);
  },

  verify(message: string | Uint8Array, signature: string, address: string): boolean {
    const messageU8a = typeof message === 'string'
      ? new TextEncoder().encode(message)
      : message;

    const { isValid } = signatureVerify(messageU8a, signature, address);
    return isValid;
  },

  createMessage(data: any): string {
    return JSON.stringify({
      data,
      timestamp: Date.now(),
      nonce: Math.random().toString(36).substring(7),
    });
  },

  hashData(data: string): string {
    const { blake2AsHex } = require('@polkadot/util-crypto');
    return blake2AsHex(data);
  },
};

