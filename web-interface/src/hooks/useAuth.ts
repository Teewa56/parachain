import { useState, useCallback, useEffect } from 'react';
import { web3Accounts, web3Enable, web3FromAddress } from '@polkadot/extension-dapp';
import { InjectedAccountWithMeta } from '@polkadot/extension-inject/types';

export function useAuth() {
  const [accounts, setAccounts] = useState<InjectedAccountWithMeta[]>([]);
  const [selectedAccount, setSelectedAccount] = useState<InjectedAccountWithMeta | null>(null);
  const [isInitialized, setIsInitialized] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const initialize = useCallback(async () => {
    try {
      const extensions = await web3Enable('PortableID');
      
      if (extensions.length === 0) {
        throw new Error('No Polkadot extension found. Please install Polkadot.js extension.');
      }

      const allAccounts = await web3Accounts();
      setAccounts(allAccounts);
      
      if (allAccounts.length > 0) {
        const stored = localStorage.getItem('selectedAccount');
        const account = stored 
          ? allAccounts.find(acc => acc.address === stored) || allAccounts[0]
          : allAccounts[0];
        setSelectedAccount(account);
      }

      setIsInitialized(true);
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Failed to initialize';
      setError(message);
      throw new Error(message);
    }
  }, []);

  const selectAccount = useCallback((account: InjectedAccountWithMeta) => {
    setSelectedAccount(account);
    localStorage.setItem('selectedAccount', account.address);
  }, []);

  const signMessage = useCallback(async (message: string) => {
    if (!selectedAccount) {
      throw new Error('No account selected');
    }

    const injector = await web3FromAddress(selectedAccount.address);
    const signRaw = injector?.signer?.signRaw;

    if (!signRaw) {
      throw new Error('Signing not supported');
    }

    const { signature } = await signRaw({
      address: selectedAccount.address,
      data: message,
      type: 'bytes',
    });

    return signature;
  }, [selectedAccount]);

  const disconnect = useCallback(() => {
    setSelectedAccount(null);
    localStorage.removeItem('selectedAccount');
  }, []);

  useEffect(() => {
    const stored = localStorage.getItem('selectedAccount');
    if (stored && !isInitialized) {
      initialize().catch(console.error);
    }
  }, [isInitialized, initialize]);

  return {
    accounts,
    selectedAccount,
    isInitialized,
    error,
    initialize,
    selectAccount,
    signMessage,
    disconnect,
  };
}