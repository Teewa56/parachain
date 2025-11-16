import { useState, useCallback } from 'react';
import { Credential, CredentialType } from '../types/credential';
import { substrateCalls, substrateQueries } from '../services/substrate';

export function useCredentials() {
  const [credentials, setCredentials] = useState<Credential[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const issue = useCallback(
    async (subjectDid: string, type: CredentialType, dataHash: string) => {
      setLoading(true);
      setError(null);
      try {
        const result = await substrateCalls.issueCredential(
          subjectDid,
          type,
          dataHash
        );
        const newCredential: Credential = {
          id: result.credentialId,
          subject: subjectDid,
          issuer: 'current-issuer',
          type,
          dataHash,
          issuedAt: Date.now(),
          expiresAt: Date.now() + 365 * 24 * 60 * 60 * 1000,
          status: 'Active',
          signature: '0x',
        };
        setCredentials((prev) => [newCredential, ...prev]);
        return result;
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to issue credential');
        throw err;
      } finally {
        setLoading(false);
      }
    },
    []
  );

  const revoke = useCallback(async (credentialId: string) => {
    setLoading(true);
    setError(null);
    try {
      await substrateCalls.revokeCredential(credentialId);
      setCredentials((prev) =>
        prev.map((c) =>
          c.id === credentialId ? { ...c, status: 'Revoked' } : c
        )
      );
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to revoke credential');
      throw err;
    } finally {
      setLoading(false);
    }
  }, []);

  return { credentials, loading, error, issue, revoke };
}
