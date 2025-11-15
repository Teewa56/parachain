import { useState, useEffect } from 'react';
import { View, Text, StyleSheet, ScrollView, RefreshControl } from 'react-native';
import { router } from 'expo-router';
import { useIdentityStore, useCredentialStore, useUIStore } from '../../src/store';
import { Card } from '../../src/components/common/Card';
import { Button } from '../../src/components/common/Button';
import { formatDid } from '../../src/substrate/utils';

export default function DashboardScreen() {
  const [refreshing, setRefreshing] = useState(false);
  
  const { did, didHash, identity } = useIdentityStore();
  const credentials = useCredentialStore(state => state.credentials);
  const getActiveCredentials = useCredentialStore(state => state.getActiveCredentials);
  const fetchCredentials = useCredentialStore(state => state.fetchCredentials);
  const showToast = useUIStore(state => state.showToast);

  useEffect(() => {
    loadData();
  }, []);

  const loadData = async () => {
    try {
      if (did) {
        await fetchCredentials();
      }
    } catch (error) {
      console.error('Failed to load dashboard data:', error);
    }
  };

  const onRefresh = async () => {
    setRefreshing(true);
    await loadData();
    setRefreshing(false);
  };

  const activeCredentials = getActiveCredentials();

  return (
    <ScrollView
      style={styles.container}
      contentContainerStyle={styles.scrollContent}
      refreshControl={
        <RefreshControl refreshing={refreshing} onRefresh={onRefresh} tintColor="#6366F1" />
      }
    >
      <View style={styles.header}>
        <Text style={styles.greeting}>Welcome back! 👋</Text>
        <Text style={styles.subtitle}>Manage your decentralized identity</Text>
      </View>

      {/* Identity Card */}
      {did ? (
        <Card style={styles.identityCard}>
          <View style={styles.identityHeader}>
            <Text style={styles.identityIcon}>👤</Text>
            <View style={styles.identityInfo}>
              <Text style={styles.identityLabel}>Your DID</Text>
              <Text style={styles.identityValue}>{formatDid(did)}</Text>
            </View>
          </View>
          {identity?.active && (
            <View style={styles.statusBadge}>
              <Text style={styles.statusText}>✓ Active</Text>
            </View>
          )}
          <Button
            onPress={() => router.push('/(wallet)/identity')}
            title="View Identity"
            variant="secondary"
            style={styles.cardButton}
          />
        </Card>
      ) : (
        <Card style={styles.createCard}>
          <Text style={styles.createTitle}>No Identity Found</Text>
          <Text style={styles.createText}>Create your decentralized identity to get started</Text>
          <Button
            onPress={() => router.push('/(wallet)/identity/create')}
            title="Create Identity"
            style={styles.cardButton}
          />
        </Card>
      )}

      {/* Stats Grid */}
      <View style={styles.statsGrid}>
        <Card style={styles.statCard}>
          <Text style={styles.statNumber}>{activeCredentials.length}</Text>
          <Text style={styles.statLabel}>Active Credentials</Text>
        </Card>
        <Card style={styles.statCard}>
          <Text style={styles.statNumber}>{credentials.length}</Text>
          <Text style={styles.statLabel}>Total Credentials</Text>
        </Card>
      </View>

      {/* Quick Actions */}
      <View style={styles.section}>
        <Text style={styles.sectionTitle}>Quick Actions</Text>
        <Card style={styles.actionCard}>
          <Button
            onPress={() => router.push('/(wallet)/credentials')}
            title="View Credentials"
            variant="secondary"
            style={styles.actionButton}
          />
          <Button
            onPress={() => router.push('/(wallet)/proof')}
            title="Generate Proof"
            style={styles.actionButton}
          />
        </Card>
      </View>
    </ScrollView>
  );
}

const styles = StyleSheet.create({
  container: { flex: 1, backgroundColor: '#F9FAFB' },
  scrollContent: { padding: 20 },
  header: { marginBottom: 24 },
  greeting: { fontSize: 28, fontWeight: '700', color: '#111827', marginBottom: 4 },
  subtitle: { fontSize: 16, color: '#6B7280' },
  identityCard: { marginBottom: 20 },
  identityHeader: { flexDirection: 'row', alignItems: 'center', marginBottom: 16 },
  identityIcon: { fontSize: 40, marginRight: 16 },
  identityInfo: { flex: 1 },
  identityLabel: { fontSize: 14, fontWeight: '600', color: '#6B7280', marginBottom: 4 },
  identityValue: { fontSize: 16, fontWeight: '600', color: '#111827' },
  statusBadge: { paddingHorizontal: 12, paddingVertical: 6, backgroundColor: '#D1FAE5', borderRadius: 12, alignSelf: 'flex-start', marginBottom: 16 },
  statusText: { fontSize: 12, fontWeight: '600', color: '#065F46' },
  createCard: { marginBottom: 20, backgroundColor: '#EEF2FF' },
  createTitle: { fontSize: 18, fontWeight: '600', color: '#111827', marginBottom: 8 },
  createText: { fontSize: 14, color: '#6B7280', marginBottom: 16 },
  cardButton: { marginTop: 8 },
  statsGrid: { flexDirection: 'row', gap: 12, marginBottom: 24 },
  statCard: { flex: 1, alignItems: 'center', paddingVertical: 20 },
  statNumber: { fontSize: 32, fontWeight: '700', color: '#6366F1', marginBottom: 4 },
  statLabel: { fontSize: 12, color: '#6B7280', textAlign: 'center' },
  section: { marginBottom: 24 },
  sectionTitle: { fontSize: 18, fontWeight: '600', color: '#111827', marginBottom: 12 },
  actionCard: { padding: 16 },
  actionButton: { marginBottom: 12 },
});