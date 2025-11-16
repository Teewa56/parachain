import { useState, useCallback } from 'react';
import { Proposal } from '../types/governance';
import { substrateCalls, substrateQueries } from '../services/substrate';

export function useGovernance() {
  const [proposals, setProposals] = useState<Proposal[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchProposals = useCallback(async () => {
    setLoading(true);
    try {
      const data = await substrateQueries.getProposals();
      setProposals(data as unknown as Proposal[]);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to fetch proposals');
    } finally {
      setLoading(false);
    }
  }, []);

  const vote = useCallback(async (proposalId: number, vote: string) => {
    setLoading(true);
    setError(null);
    try {
      return await substrateCalls.voteProposal(proposalId, vote);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to vote');
      throw err;
    } finally {
      setLoading(false);
    }
  }, []);

  return { proposals, loading, error, fetchProposals, vote };
}
