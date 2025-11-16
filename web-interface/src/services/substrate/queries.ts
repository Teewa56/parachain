export const substrateQueries = {
  async getIdentity(did: string) {
    // Query pallet-identity-registry
    // Call: IdentityRegistry::Identities(did_hash)
    return new Promise((resolve) => {
      setTimeout(() => resolve({ active: true, did }), 500);
    });
  },

  async getCredential(credentialId: string) {
    // Query pallet-verifiable-credentials
    // Call: VerifiableCredentials::Credentials(credential_id)
    return new Promise((resolve) => {
      setTimeout(
        () =>
          resolve({
            id: credentialId,
            status: 'Active',
            subject: '0xabc123',
          }),
        500
      );
    });
  },
  async getCredentialsOf(subjectDid: string) {
    // Query pallet-verifiable-credentials
    // Call: VerifiableCredentials::CredentialsOf(subject_did)
    return new Promise((resolve) => {
      setTimeout(() => resolve([]), 500);
    });
  },

  async getTrustedIssuers(credentialType: string) {
    // Query pallet-credential-governance
    // Call: CredentialGovernance::TrustedIssuers(credential_type)
    return new Promise((resolve) => {
      setTimeout(() => resolve([]), 500);
    });
  },
async getProposals() {
    // Query pallet-credential-governance
    // Call: CredentialGovernance::Proposals()
    return new Promise((resolve) => {
      setTimeout(
        () =>
          resolve([
            { id: 1, status: 'Active', votes: '18/25' },
            { id: 2, status: 'Approved', votes: '22/25' },
          ]),
        500
      );
    });
  },
};
