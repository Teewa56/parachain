import { CredentialType, Credential } from '../../types/credential';

export const credentialsService = {
    async issue(
        api: any,
        subjectDid: string,
        credentialType: CredentialType,
        dataHash: string,
        expiresAt: number,
        signature: string
    ): Promise<string> {
        const tx = api.tx.verifiableCredentials.issueCredential(
        subjectDid,
        credentialType,
        dataHash,
        expiresAt,
        signature
        );

        return new Promise((resolve, reject) => {
        tx.signAndSend((result: any) => {
            if (result.status.isInBlock) {
            const credentialId = result.events
                .find((e: any) => e.event.method === 'CredentialIssued')
                ?.event.data[0].toString();
            resolve(credentialId);
            } else if (result.status.isFinalized) {
            // Transaction finalized
            } else if (result.isError) {
            reject(new Error('Transaction failed'));
            }
        }).catch(reject);
        });
    },

    async revoke(api: any, credentialId: string): Promise<void> {
        const tx = api.tx.verifiableCredentials.revokeCredential(credentialId);
        
        return new Promise((resolve, reject) => {
        tx.signAndSend((result: any) => {
            if (result.status.isFinalized) {
            resolve();
            } else if (result.isError) {
            reject(new Error('Revocation failed'));
            }
        }).catch(reject);
        });
    },

    async getCredential(api: any, credentialId: string): Promise<Credential | null> {
        const credential = await api.query.verifiableCredentials.credentials(credentialId);
        
        if (credential.isNone) {
        return null;
        }

        return credential.unwrap().toJSON() as Credential;
    },

    async getCredentialsOf(api: any, subjectDid: string): Promise<string[]> {
        const credentials = await api.query.verifiableCredentials.credentialsOf(subjectDid);
        return credentials.toJSON() as string[];
    },
};
