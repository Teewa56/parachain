import { substrateAPI } from './api';
import { PALLETS, QUERIES } from '../../config/substrate';
import type {
    Identity,
    DidDocument,
    Credential,
    IdentityQueryResult,
    CredentialQueryResult,
    ZkProof,
} from '../../types/substrate';
import { hexToU8a, u8aToHex } from '@polkadot/util';

class SubstrateQueryService {
    /**
     * Query identity by DID hash
     */
    async getIdentity(didHash: string): Promise<IdentityQueryResult> {
        const api = substrateAPI.getApi();

        try {
        const identityOption = await api.query[PALLETS.IDENTITY_REGISTRY][
            QUERIES.IDENTITY.IDENTITIES
        ](didHash);

        if (identityOption.isNone) {
            return { exists: false };
        }

        const identityData = identityOption.unwrap();
        const identity: Identity = {
            controller: identityData.controller.toString(),
            publicKey: identityData.publicKey.toHex(),
            createdAt: identityData.createdAt.toNumber(),
            updatedAt: identityData.updatedAt.toNumber(),
            active: identityData.active.isTrue,
        };

        // Also get DID document
        const didDocOption = await api.query[PALLETS.IDENTITY_REGISTRY][
            QUERIES.IDENTITY.DID_DOCUMENTS
        ](didHash);

        let didDocument: DidDocument | undefined;
        if (didDocOption.isSome) {
            const docData = didDocOption.unwrap();
            didDocument = {
            did: docData.did.toUtf8(),
            publicKeys: docData.publicKeys.map((key: any) => key.toHex()),
            authentication: docData.authentication.map((key: any) => key.toHex()),
            services: docData.services.map((service: any) => service.toUtf8()),
            };
        }

        return {
            exists: true,
            identity,
            didDocument,
        };
        } catch (error) {
        console.error('Error querying identity:', error);
        throw error;
        }
    }

    /**
     * Get DID hash for an account
     */
    async getAccountDid(accountId: string): Promise<string | null> {
        const api = substrateAPI.getApi();

        try {
        const didHashOption = await api.query[PALLETS.IDENTITY_REGISTRY][
            QUERIES.IDENTITY.ACCOUNT_DIDS
        ](accountId);

        if (didHashOption.isNone) {
            return null;
        }

        return didHashOption.unwrap().toHex();
        } catch (error) {
        console.error('Error querying account DID:', error);
        throw error;
        }
    }

    /**
     * Query credential by ID
     */
    async getCredential(credentialId: string): Promise<CredentialQueryResult> {
        const api = substrateAPI.getApi();

        try {
        const credentialOption = await api.query[PALLETS.VERIFIABLE_CREDENTIALS][
            QUERIES.CREDENTIALS.CREDENTIALS
        ](credentialId);

        if (credentialOption.isNone) {
            return { exists: false };
        }

        const credData = credentialOption.unwrap();
        const credential: Credential = {
            subject: credData.subject.toHex(),
            issuer: credData.issuer.toHex(),
            credentialType: credData.credentialType.toString() as any,
            dataHash: credData.dataHash.toHex(),
            issuedAt: credData.issuedAt.toNumber(),
            expiresAt: credData.expiresAt.toNumber(),
            status: credData.status.toString() as any,
            signature: credData.signature.toHex(),
            metadataHash: credData.metadataHash.toHex(),
        };

        return {
            exists: true,
            credential,
        };
        } catch (error) {
        console.error('Error querying credential:', error);
        throw error;
        }
    }

    /**
     * Get all credentials for a subject DID
     */
    async getCredentialsForSubject(subjectDid: string): Promise<string[]> {
        const api = substrateAPI.getApi();

        try {
        const credentialIds = await api.query[PALLETS.VERIFIABLE_CREDENTIALS][
            QUERIES.CREDENTIALS.CREDENTIALS_OF
        ](subjectDid);

        return credentialIds.map((id: any) => id.toHex());
        } catch (error) {
        console.error('Error querying credentials for subject:', error);
        throw error;
        }
    }

    /**
     * Get all credentials issued by an issuer
     */
    async getCredentialsByIssuer(issuerDid: string): Promise<string[]> {
        const api = substrateAPI.getApi();

        try {
        const credentialIds = await api.query[PALLETS.VERIFIABLE_CREDENTIALS][
            QUERIES.CREDENTIALS.ISSUED_BY
        ](issuerDid);

        return credentialIds.map((id: any) => id.toHex());
        } catch (error) {
        console.error('Error querying credentials by issuer:', error);
        throw error;
        }
    }

    /**
     * Check if issuer is trusted for a credential type
     */
    async isIssuerTrusted(
        credentialType: string,
        issuerDid: string
    ): Promise<boolean> {
        const api = substrateAPI.getApi();

        try {
        const isTrusted = await api.query[PALLETS.VERIFIABLE_CREDENTIALS][
            QUERIES.CREDENTIALS.TRUSTED_ISSUERS
        ]([credentialType, issuerDid]);

        return isTrusted.isTrue;
        } catch (error) {
        console.error('Error checking trusted issuer:', error);
        throw error;
        }
    }

    /**
     * Get verification key for proof type
     */
    async getVerificationKey(proofType: string): Promise<any | null> {
        const api = substrateAPI.getApi();

        try {
        const vkOption = await api.query[PALLETS.ZK_CREDENTIALS][
            QUERIES.ZK.VERIFYING_KEYS
        ](proofType);

        if (vkOption.isNone) {
            return null;
        }

        const vkData = vkOption.unwrap();
        return {
            proofType: vkData.proofType.toString(),
            vkData: vkData.vkData.toU8a(),
            registeredBy: vkData.registeredBy.toHex(),
        };
        } catch (error) {
        console.error('Error querying verification key:', error);
        throw error;
        }
    }

    /**
     * Check if a proof has been verified
     */
    async isProofVerified(proofHash: string): Promise<boolean> {
        const api = substrateAPI.getApi();

        try {
        const verifiedOption = await api.query[PALLETS.ZK_CREDENTIALS][
            QUERIES.ZK.VERIFIED_PROOFS
        ](proofHash);

        return verifiedOption.isSome;
        } catch (error) {
        console.error('Error checking proof verification:', error);
        throw error;
        }
    }

    /**
     * Get account balance
     */
    async getBalance(accountId: string): Promise<{
        free: string;
        reserved: string;
        frozen: string;
    }> {
        const api = substrateAPI.getApi();

        try {
        const account = await api.query.system.account(accountId);
        const data = account.data;

        return {
            free: data.free.toString(),
            reserved: data.reserved.toString(),
            frozen: data.frozen.toString(),
        };
        } catch (error) {
        console.error('Error querying balance:', error);
        throw error;
        }
    }

    /**
     * Get account nonce
     */
    async getNonce(accountId: string): Promise<number> {
        const api = substrateAPI.getApi();

        try {
        const nonce = await api.rpc.system.accountNextIndex(accountId);
        return nonce.toNumber();
        } catch (error) {
        console.error('Error querying nonce:', error);
        throw error;
        }
    }

    /**
     * Batch query multiple credentials
     */
    async batchGetCredentials(credentialIds: string[]): Promise<Credential[]> {
        const credentials: Credential[] = [];

        for (const id of credentialIds) {
        try {
            const result = await this.getCredential(id);
            if (result.exists && result.credential) {
            credentials.push(result.credential);
            }
        } catch (error) {
            console.error(`Error fetching credential ${id}:`, error);
        }
        }

        return credentials;
    }

    /**
     * Get all credentials for an account (by fetching their DID first)
     */
    async getCredentialsForAccount(accountId: string): Promise<Credential[]> {
        try {
        // Get DID hash for account
        const didHash = await this.getAccountDid(accountId);
        if (!didHash) {
            return [];
        }

        // Get credential IDs
        const credentialIds = await this.getCredentialsForSubject(didHash);
        if (credentialIds.length === 0) {
            return [];
        }

        // Batch fetch credentials
        return await this.batchGetCredentials(credentialIds);
        } catch (error) {
        console.error('Error getting credentials for account:', error);
        throw error;
        }
    }
}

export const substrateQueries = new SubstrateQueryService();