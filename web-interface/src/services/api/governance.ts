import { Proposal } from '../../types/governance';

export const governanceService = {
  async proposeAddIssuer(
    api: any,
    issuerDid: string,
    credentialTypes: string[],
    description: string
  ): Promise<number> {
    const tx = api.tx.credentialGovernance.proposeAddIssuer(
      issuerDid,
      credentialTypes,
      description
    );

    return new Promise((resolve, reject) => {
      tx.signAndSend((result: any) => {
        if (result.status.isInBlock) {
          const proposalId = result.events
            .find((e: any) => e.event.method === 'ProposalCreated')
            ?.event.data[0].toNumber();
          resolve(proposalId);
        } else if (result.isError) {
          reject(new Error('Proposal submission failed'));
        }
      }).catch(reject);
    });
  },

  async vote(api: any, proposalId: number, vote: 'Yes' | 'No' | 'Abstain'): Promise<void> {
    const tx = api.tx.credentialGovernance.vote(proposalId, vote);

    return new Promise((resolve, reject) => {
      tx.signAndSend((result: any) => {
        if (result.status.isFinalized) {
          resolve();
        } else if (result.isError) {
          reject(new Error('Vote failed'));
        }
      }).catch(reject);
    });
  },

  async getProposal(api: any, proposalId: number): Promise<Proposal | null> {
    const proposal = await api.query.credentialGovernance.proposals(proposalId);
    
    if (proposal.isNone) {
      return null;
    }

    return proposal.unwrap().toJSON() as Proposal;
  },

  async getAllProposals(api: any): Promise<Proposal[]> {
    const entries = await api.query.credentialGovernance.proposals.entries();
    return entries.map(([_, proposal]: any) => proposal.toJSON());
  },

  async finalizeProposal(api: any, proposalId: number): Promise<void> {
    const tx = api.tx.credentialGovernance.finalizeProposal(proposalId);

    return new Promise((resolve, reject) => {
      tx.signAndSend((result: any) => {
        if (result.status.isFinalized) {
          resolve();
        } else if (result.isError) {
          reject(new Error('Finalization failed'));
        }
      }).catch(reject);
    });
  },
};
