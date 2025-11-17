export class EventListener {
    private api: any;
    private unsubscribe: (() => void) | null = null;

    constructor(api: any) {
        this.api = api;
    }

    async subscribeToEvents(callback: (event: any) => void) {
        this.unsubscribe = await this.api.query.system.events((events: any[]) => {
        events.forEach((record) => {
            const { event } = record;
            callback({
            section: event.section,
            method: event.method,
            data: event.data.toJSON(),
            meta: event.meta.toJSON(),
            });
        });
        });
    }

    async subscribeToCredentialEvents(callback: (event: any) => void) {
        this.unsubscribe = await this.api.query.system.events((events: any[]) => {
        events.forEach((record) => {
            const { event } = record;
            if (event.section === 'verifiableCredentials') {
            callback({
                type: event.method,
                data: event.data.toJSON(),
                blockNumber: record.phase.asApplyExtrinsic,
            });
            }
        });
        });
    }

    async subscribeToGovernanceEvents(callback: (event: any) => void) {
        this.unsubscribe = await this.api.query.system.events((events: any[]) => {
        events.forEach((record) => {
            const { event } = record;
            if (event.section === 'credentialGovernance') {
            callback({
                type: event.method,
                data: event.data.toJSON(),
                blockNumber: record.phase.asApplyExtrinsic,
            });
            }
        });
        });
    }

    unsubscribeAll() {
        if (this.unsubscribe) {
            this.unsubscribe();
            this.unsubscribe = null;
        }
    }
}
