export interface Proposal {
  id: number;
  proposer: string;
  issuerDid: string;
  proposalType: ProposalType;
  credentialTypes: CredentialType[];
  description: string;
  deposit: string;
  createdAt: number;
  votingEndsAt: number;
  status: ProposalStatus;
  yesVotes: number;
  noVotes: number;
}

export enum ProposalType {
  AddTrustedIssuer = 'AddTrustedIssuer',
  RemoveTrustedIssuer = 'RemoveTrustedIssuer',
  UpdateIssuerPermissions = 'UpdateIssuerPermissions',
}

export enum ProposalStatus {
  Active = 'Active',
  Approved = 'Approved',
  Rejected = 'Rejected',
  Executed = 'Executed',
  Cancelled = 'Cancelled',
}

export interface Vote {
  voteType: VoteType;
  voter: string;
  votingPower: number;
}

export enum VoteType {
  Yes = 'Yes',
  No = 'No',
  Abstain = 'Abstain',
}
