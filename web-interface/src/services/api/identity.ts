export const identityService = {
  async createIdentity(api: any, did: string, publicKey: string): Promise<string> {
    const tx = api.tx.identityRegistry.createIdentity(did, publicKey);

    return new Promise((resolve, reject) => {
      tx.signAndSend((result: any) => {
        if (result.status.isInBlock) {
          const didHash = result.events
            .find((e: any) => e.event.method === 'IdentityCreated')
            ?.event.data[0].toString();
          resolve(didHash);
        } else if (result.isError) {
          reject(new Error('Identity creation failed'));
        }
      }).catch(reject);
    });
  },

  async getIdentity(api: any, didHash: string) {
    const identity = await api.query.identityRegistry.identities(didHash);
    
    if (identity.isNone) {
      return null;
    }

    return identity.unwrap().toJSON();
  },

  async updateIdentity(api: any, newPublicKey: string): Promise<void> {
    const tx = api.tx.identityRegistry.updateIdentity(newPublicKey);

    return new Promise((resolve, reject) => {
      tx.signAndSend((result: any) => {
        if (result.status.isFinalized) {
          resolve();
        } else if (result.isError) {
          reject(new Error('Identity update failed'));
        }
      }).catch(reject);
    });
  },

  async deactivateIdentity(api: any): Promise<void> {
    const tx = api.tx.identityRegistry.deactivateIdentity();

    return new Promise((resolve, reject) => {
      tx.signAndSend((result: any) => {
        if (result.status.isFinalized) {
          resolve();
        } else if (result.isError) {
          reject(new Error('Deactivation failed'));
        }
      }).catch(reject);
    });
  },
};