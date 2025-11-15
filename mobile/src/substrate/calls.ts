import { substrateAPI } from './api';
import { PALLETS, EXTRINSICS } from '../../config/substrate';
import type {
  TransactionResult,
  ExtrinsicStatus,
  CredentialType,
  ProofType,
} from '../../types/substrate';
import type { KeyringPair } from '@polkadot/keyring/types';
import type { SubmittableExtrinsic } from '@polkadot/api/types';
import { hexToU8a } from '@polkadot/util';

class SubstrateTransactionService {
    /**
     * Create a new identity on-chain
     */
    async createIdentity(
        keyPair: KeyringPair,
        did: string,
        publicKey: string
    ): Promise<TransactionResult> {
        const api = substrateAPI.getApi();

        try {
        const extrinsic = api.tx[PALLETS.IDENTITY_REGISTRY][
            EXTRINSICS.IDENTITY.CREATE
        ](did, publicKey);

        return await this.signAndSend(extrinsic, keyPair);
        } catch (error) {
        console.error('Error creating identity:', error);
        return {
            success: false,
            error: error instanceof Error ? error.message : 'Unknown error',
        };
        }
    }

    /**
     * Update identity public key
     */
    async updateIdentity(
        keyPair: KeyringPair,
        newPublicKey: string
    ): Promise<TransactionResult> {
        const api = substrateAPI.getApi();

        try {
        const extrinsic = api.tx[PALLETS.IDENTITY_REGISTRY][
            EXTRINSICS.IDENTITY.UPDATE
        ](newPublicKey);

        return await this.signAndSend(extrinsic, keyPair);
        } catch (error) {
        console.error('Error updating identity:', error);
        return {
            success: false,
            error: error instanceof Error ? error.message : 'Unknown error',
        };
        }
    }

    /**
     * Deactivate identity
     */
    async deactivateIdentity(keyPair: KeyringPair): Promise<TransactionResult> {
        const api = substrateAPI.getApi();

        try {
        const extrinsic = api.tx[PALLETS.IDENTITY_REGISTRY][
            EXTRINSICS.IDENTITY.DEACTIVATE
        ]();

        return await this.signAndSend(extrinsic, keyPair);
        } catch (error) {
        console.error('Error deactivating identity:', error);
        return {
            success: false,
            error: error instanceof Error ? error.message : 'Unknown error',
        };
        }
    }

    /**
     * Reactivate identity
     */
    async reactivateIdentity(keyPair: KeyringPair): Promise<TransactionResult> {
        const api = substrateAPI.getApi();

        try {
        const extrinsic = api.tx[PALLETS.IDENTITY_REGISTRY][
            EXTRINSICS.IDENTITY.REACTIVATE
        ]();

        return await this.signAndSend(extrinsic, keyPair);
        } catch (error) {
        console.error('Error reactivating identity:', error);
        return {
            success: false,
            error: error instanceof Error ? error.message : 'Unknown error',
        };
        }
    }

    /**
     * Verify a credential
     */
    async verifyCredential(
        keyPair: KeyringPair,
        credentialId: string
    ): Promise<TransactionResult> {
        const api = substrateAPI.getApi();

        try {
        const extrinsic = api.tx[PALLETS.VERIFIABLE_CREDENTIALS][
            EXTRINSICS.CREDENTIALS.VERIFY
        ](credentialId);

        return await this.signAndSend(extrinsic, keyPair);
        } catch (error) {
        console.error('Error verifying credential:', error);
        return {
            success: false,
            error: error instanceof Error ? error.message : 'Unknown error',
        };
        }
    }

    /**
     * Selective disclosure - Reveal specific fields of a credential
     */
    async selectiveDisclosure(
        keyPair: KeyringPair,
        credentialId: string,
        fieldsToReveal: number[],
        proof: string
    ): Promise<TransactionResult> {
        const api = substrateAPI.getApi();

        try {
        const extrinsic = api.tx[PALLETS.VERIFIABLE_CREDENTIALS][
            EXTRINSICS.CREDENTIALS.SELECTIVE_DISCLOSURE
        ](credentialId, fieldsToReveal, proof);

        return await this.signAndSend(extrinsic, keyPair);
        } catch (error) {
        console.error('Error in selective disclosure:', error);
        return {
            success: false,
            error: error instanceof Error ? error.message : 'Unknown error',
        };
        }
    }

    /**
     * Verify a ZK proof
     */
    async verifyZkProof(
        keyPair: KeyringPair,
        proofType: ProofType,
        proofData: Uint8Array,
        publicInputs: Uint8Array[],
        credentialHash: string,
        createdAt: number,
        nonce: string
    ): Promise<TransactionResult> {
        const api = substrateAPI.getApi();

        try {
        const proof = {
            proofType,
            proofData,
            publicInputs,
            credentialHash,
            createdAt,
            nonce,
        };

        const extrinsic = api.tx[PALLETS.ZK_CREDENTIALS][
            EXTRINSICS.ZK_PROOFS.VERIFY_PROOF
        ](proof);

        return await this.signAndSend(extrinsic, keyPair);
        } catch (error) {
        console.error('Error verifying ZK proof:', error);
        return {
            success: false,
            error: error instanceof Error ? error.message : 'Unknown error',
        };
        }
    }

    /**
     * Create a proof schema
     */
    async createProofSchema(
        keyPair: KeyringPair,
        proofType: ProofType,
        fieldDescriptions: string[]
    ): Promise<TransactionResult> {
        const api = substrateAPI.getApi();

        try {
        const extrinsic = api.tx[PALLETS.ZK_CREDENTIALS][
            EXTRINSICS.ZK_PROOFS.CREATE_SCHEMA
        ](proofType, fieldDescriptions);

        return await this.signAndSend(extrinsic, keyPair);
        } catch (error) {
        console.error('Error creating proof schema:', error);
        return {
            success: false,
            error: error instanceof Error ? error.message : 'Unknown error',
        };
        }
    }

    /**
     * Transfer tokens
     */
    async transfer(
        keyPair: KeyringPair,
        dest: string,
        amount: string
    ): Promise<TransactionResult> {
        const api = substrateAPI.getApi();

        try {
        const extrinsic = api.tx.balances.transferKeepAlive(dest, amount);
        return await this.signAndSend(extrinsic, keyPair);
        } catch (error) {
        console.error('Error transferring:', error);
        return {
            success: false,
            error: error instanceof Error ? error.message : 'Unknown error',
        };
        }
    }

    /**
     * Sign and send an extrinsic
     */
    private async signAndSend(
        extrinsic: SubmittableExtrinsic<'promise'>,
        keyPair: KeyringPair
    ): Promise<TransactionResult> {
        return new Promise((resolve) => {
        let unsubscribe: (() => void) | null = null;

        extrinsic
            .signAndSend(keyPair, (result) => {
            const status = this.parseExtrinsicStatus(result);

            if (status.isFinalized) {
                if (unsubscribe) unsubscribe();

                if (status.error) {
                resolve({
                    success: false,
                    error: status.error,
                });
                } else {
                resolve({
                    success: true,
                    hash: result.txHash,
                    blockNumber: result.status.isInBlock
                    ? result.status.asInBlock
                    : result.status.asFinalized,
                });
                }
            }
            })
            .then((unsub) => {
            unsubscribe = unsub;
            })
            .catch((error) => {
            console.error('Error signing/sending:', error);
            resolve({
                success: false,
                error: error instanceof Error ? error.message : 'Unknown error',
            });
            });
        });
    }

    /**
     * Parse extrinsic status from result
     */
    private parseExtrinsicStatus(result: any): ExtrinsicStatus {
        const status: ExtrinsicStatus = {
        isFinalized: false,
        isInBlock: false,
        hash: result.txHash?.toString() || '',
        };

        if (result.status.isFinalized) {
        status.isFinalized = true;
        status.hash = result.status.asFinalized.toString();
        }

        if (result.status.isInBlock) {
        status.isInBlock = true;
        status.hash = result.status.asInBlock.toString();
        }

        // Check for errors in events
        result.events.forEach(({ event }: any) => {
        if (event.section === 'system') {
            if (event.method === 'ExtrinsicFailed') {
            const [dispatchError] = event.data;
            let errorInfo = dispatchError.toString();

            if (dispatchError.isModule) {
                try {
                const decoded = dispatchError.registry.findMetaError(
                    dispatchError.asModule
                );
                errorInfo = `${decoded.section}.${decoded.name}: ${decoded.docs.join(' ')}`;
                } catch (e) {
                errorInfo = 'Module error';
                }
            }

            status.error = errorInfo;
            }
        }
        });

        return status;
    }

    /**
     * Estimate transaction fees
     */
    async estimateFee(
        extrinsic: SubmittableExtrinsic<'promise'>,
        address: string
    ): Promise<string> {
        try {
        const paymentInfo = await extrinsic.paymentInfo(address);
        return paymentInfo.partialFee.toString();
        } catch (error) {
        console.error('Error estimating fee:', error);
        throw error;
        }
    }

    /**
     * Dry run an extrinsic to check if it will succeed
     */
    async dryRun(
        extrinsic: SubmittableExtrinsic<'promise'>,
        keyPair: KeyringPair
    ): Promise<boolean> {
        const api = substrateAPI.getApi();

        try {
        const signedExtrinsic = await extrinsic.signAsync(keyPair);
        const result = await api.rpc.system.dryRun(signedExtrinsic.toHex());

        return result.isOk;
        } catch (error) {
        console.error('Error in dry run:', error);
        return false;
        }
    }
}

export const substrateCalls = new SubstrateTransactionService();