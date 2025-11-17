import { ApiPromise } from '@polkadot/api';

export const substrateUtils = {
    async waitForBlock(api: ApiPromise, blockNumber: number): Promise<void> {
        return new Promise((resolve) => {
        const unsubscribe = api.rpc.chain.subscribeNewHeads((header) => {
            if (header.number.toNumber() >= blockNumber) {
            unsubscribe.then((unsub) => unsub());
            resolve();
            }
        });
        });
    },

    async getBlockTimestamp(api: ApiPromise, blockHash: string): Promise<number> {
        const timestamp = await api.query.timestamp.now.at(blockHash);
        return timestamp.toNumber();
    },

    async getExtrinsicStatus(api: ApiPromise, txHash: string): Promise<any> {
        const signedBlock = await api.rpc.chain.getBlock();
        const extrinsic = signedBlock.block.extrinsics.find(
        (ext) => ext.hash.toHex() === txHash
        );
        
        if (!extrinsic) {
        return null;
        }

        return {
        hash: extrinsic.hash.toHex(),
        method: extrinsic.method.toJSON(),
        signature: extrinsic.signature.toJSON(),
        };
    },

    encodeMetadata(data: any): Uint8Array {
        return new TextEncoder().encode(JSON.stringify(data));
    },

    decodeMetadata(data: Uint8Array): any {
        const text = new TextDecoder().decode(data);
        return JSON.parse(text);
    },
};