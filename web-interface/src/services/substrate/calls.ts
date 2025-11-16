export const substrateCalls = {
  async createIdentity(did: string, publicKey: string) {
    // Extrinsic: IdentityRegistry::create_identity(did, public_key)
    return new Promise((resolve) => {
      setTimeout(() => resolve({ success: true }), 800);
    });
  },

  async issueCredential(
    subjectDid: string,
    credentialType: string,
    dataHash: string
  ) {
    // Extrinsic: VerifiableCredentials::issue_credential(...)
    return new Promise((resolve) => {
      setTimeout(
        () =>
          resolve({
            credentialId: `0x${Math.random().toString(16).slice(2)}`,
            transactionHash: `0x${Math.random().toString(16).slice(2)}`,
          }),
        1000
      );
    });
  },
  async revokeCredential(credentialId: string) {
    // Extrinsic: VerifiableCredentials::revoke_credential(credential_id)
    return new Promise((resolve) => {
      setTimeout(() => resolve({ success: true }), 800);
    });
  },

  async proposeAddIssuer(issuerDid: string, types: string[]) {
    // Extrinsic: CredentialGovernance::propose_add_issuer(...)
    return new Promise((resolve) => {
      setTimeout(() => resolve({ proposalId: 1, success: true }), 800);
    });
  },

  async voteProposal(proposalId: number, vote: string) {
    // Extrinsic: CredentialGovernance::vote(proposal_id, vote)
    return new Promise((resolve) => {
      setTimeout(() => resolve({ success: true }), 600);
    });
  },
};
