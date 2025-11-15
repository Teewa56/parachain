import { useEffect } from 'react';
import { Stack } from 'expo-router';
import { useAuthStore, useIdentityStore } from '../src/store';
import { substrateAPI } from '../src/substrate/api';
import { config } from '../src/config/env';

export default function RootLayout() {
  const loadAuthState = useAuthStore(state => state.loadAuthState);
  const loadIdentity = useIdentityStore(state => state.loadIdentity);

  useEffect(() => {
    // Initialize app
    const initializeApp = async () => {
      await loadAuthState();
      await loadIdentity();
      
      // Connect to blockchain
      try {
        await substrateAPI.connect(config.wsEndpoint);
      } catch (error) {
        console.error('Failed to connect to blockchain:', error);
      }
    };

    initializeApp();
  }, []);

  return (
    <Stack screenOptions={{ headerShown: false }}>
      <Stack.Screen name="(auth)" options={{ headerShown: false }} />
      <Stack.Screen name="(wallet)" options={{ headerShown: false }} />
      <Stack.Screen name="+not-found" />
    </Stack>
  );
}