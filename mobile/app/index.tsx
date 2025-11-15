import { useEffect } from 'react';
import { View, Text, ActivityIndicator, StyleSheet } from 'react-native';
import { router } from 'expo-router';
import { useAuthStore, useIdentityStore } from '../src/store';

export default function Index() {
  const isAuthenticated = useAuthStore(state => state.isAuthenticated);
  const did = useIdentityStore(state => state.did);

  useEffect(() => {
    // Determine initial route
    const timeout = setTimeout(() => {
      if (isAuthenticated && did) {
        router.replace('/(wallet)');
      } else {
        router.replace('/(auth)/login');
      }
    }, 1000);

    return () => clearTimeout(timeout);
  }, [isAuthenticated, did]);

  return (
    <View style={styles.container}>
      <Text style={styles.title}>PortableID</Text>
      <Text style={styles.subtitle}>True Control, zero exposure</Text>
      <ActivityIndicator size="large" color="#6366F1" style={styles.loader} />
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    backgroundColor: '#FFFFFF',
  },
  title: {
    fontSize: 32,
    fontWeight: '700',
    color: '#111827',
    marginBottom: 8,
  },
  subtitle: {
    fontSize: 16,
    color: '#6B7280',
    marginBottom: 32,
  },
  loader: {
    marginTop: 20,
  },
});