import { useState, useEffect } from 'react';
import { View, Text, StyleSheet, ScrollView, RefreshControl, TouchableOpacity } from 'react-native';
import { router } from 'expo-router';
import { useIdentityStore, useUIStore } from '../../../src/store';
import { Card } from '../../../src/components/common/Card';
import { Button } from '../../../src/components/common/Button';
import { DIDDisplay } from '../../../src/components/identity/DIDDisplay';
import { formatTimestamp } from '../../../src/substrate/utils';

export default function IdentityIndexScreen() {
    const [refreshing, setRefreshing] = useState(false);

    const { did, didHash, address, identity, refreshIdentity } = useIdentityStore();
    const showToast = useUIStore(state => state.showToast);

    useEffect(() => {
        if (did) {
            loadIdentity();
        }
    }, [did]);

    const loadIdentity = async () => {
        try {
            await refreshIdentity();
        } catch (error) {
            console.error('Failed to load identity:', error);
        }
    };

    const onRefresh = async () => {
        setRefreshing(true);
        await loadIdentity();
        setRefreshing(false);
    };

    const handleCopyDID = async () => {
        if (!did) return;
        // In production, implement clipboard copy
        showToast('DID copied to clipboard', 'success');
    };

    if (!did) {
        return (
            <View style={styles.container}>
                <View style={styles.emptyContainer}>
                    <Text style={styles.emptyIcon}>👤</Text>
                    <Text style={styles.emptyTitle}>No Identity Found</Text>
                    <Text style={styles.emptyText}>
                        Create your decentralized identity to get started with verifiable credentials.
                    </Text>
                    <Button
                        onPress={() => router.push('/(wallet)/identity/create')}
                        title="Create Identity"
                        style={styles.createButton}
                    />
                </View>
            </View>
        );
    }

    return (
        <ScrollView
            style={styles.container}
            contentContainerStyle={styles.scrollContent}
            refreshControl={
                <RefreshControl refreshing={refreshing} onRefresh={onRefresh} tintColor="#6366F1" />
            }
        >
            {/* DID Display Card */}
            <Card style={styles.didCard}>
                <DIDDisplay did={did} onCopy={handleCopyDID} />
            </Card>

            {/* Identity Status */}
            {identity && (
                <Card style={styles.statusCard}>
                    <View style={styles.statusHeader}>
                        <Text style={styles.cardTitle}>Identity Status</Text>
                        <View style={[
                            styles.statusBadge,
                            { backgroundColor: identity.active ? '#D1FAE5' : '#FEE2E2' }
                        ]}>
                            <Text style={[
                                styles.statusText,
                                { color: identity.active ? '#065F46' : '#991B1B' }
                            ]}>
                                {identity.active ? '✓ Active' : '✗ Inactive'}
                            </Text>
                        </View>
                    </View>

                    <View style={styles.infoRow}>
                        <Text style={styles.infoLabel}>Controller</Text>
                        <Text style={styles.infoValue}>{address?.slice(0, 10)}...</Text>
                    </View>

                    <View style={styles.infoRow}>
                        <Text style={styles.infoLabel}>Created</Text>
                        <Text style={styles.infoValue}>
                            {formatTimestamp(identity.createdAt)}
                        </Text>
                    </View>

                    <View style={styles.infoRow}>
                        <Text style={styles.infoLabel}>Last Updated</Text>
                        <Text style={styles.infoValue}>
                            {formatTimestamp(identity.updatedAt)}
                        </Text>
                    </View>
                </Card>
            )}

            {/* Actions */}
            <View style={styles.actionsSection}>
                <Text style={styles.sectionTitle}>Actions</Text>
                
                <TouchableOpacity
                    style={styles.actionItem}
                    onPress={() => router.push('/(wallet)/identity/manage')}
                >
                    <Text style={styles.actionIcon}>⚙️</Text>
                    <View style={styles.actionContent}>
                        <Text style={styles.actionTitle}>Manage Identity</Text>
                        <Text style={styles.actionDescription}>
                            Update or deactivate your identity
                        </Text>
                    </View>
                    <Text style={styles.actionArrow}>›</Text>
                </TouchableOpacity>

                <TouchableOpacity
                    style={styles.actionItem}
                    onPress={() => router.push({
                        pathname: '/(wallet)/identity/[id]',
                        params: { id: didHash || '' }
                    })}
                >
                    <Text style={styles.actionIcon}>🔍</Text>
                    <View style={styles.actionContent}>
                        <Text style={styles.actionTitle}>View Details</Text>
                        <Text style={styles.actionDescription}>
                            See complete identity information
                        </Text>
                    </View>
                    <Text style={styles.actionArrow}>›</Text>
                </TouchableOpacity>
            </View>

            {/* Information Card */}
            <Card style={styles.infoCard}>
                <Text style={styles.infoCardTitle}>ℹ️ About DIDs</Text>
                <Text style={styles.infoCardText}>
                    Your Decentralized Identifier (DID) is a unique identifier that you control. 
                    It's stored on the blockchain and can be used to receive verifiable credentials.
                </Text>
            </Card>
        </ScrollView>
    );
}

const styles = StyleSheet.create({
    container: { flex: 1, backgroundColor: '#F9FAFB' },
    scrollContent: { padding: 20 },
    emptyContainer: { flex: 1, alignItems: 'center', justifyContent: 'center', padding: 40 },
    emptyIcon: { fontSize: 64, marginBottom: 16 },
    emptyTitle: { fontSize: 20, fontWeight: '600', color: '#111827', marginBottom: 8, textAlign: 'center' },
    emptyText: { fontSize: 14, color: '#6B7280', textAlign: 'center', lineHeight: 20, marginBottom: 24 },
    createButton: { minWidth: 200 },
    didCard: { marginBottom: 16 },
    statusCard: { marginBottom: 16 },
    statusHeader: { flexDirection: 'row', justifyContent: 'space-between', alignItems: 'center', marginBottom: 16 },
    cardTitle: { fontSize: 18, fontWeight: '600', color: '#111827' },
    statusBadge: { paddingHorizontal: 12, paddingVertical: 6, borderRadius: 12 },
    statusText: { fontSize: 12, fontWeight: '600' },
    infoRow: { flexDirection: 'row', justifyContent: 'space-between', paddingVertical: 12, borderTopWidth: 1, borderTopColor: '#E5E7EB' },
    infoLabel: { fontSize: 14, color: '#6B7280' },
    infoValue: { fontSize: 14, fontWeight: '500', color: '#111827' },
    actionsSection: { marginBottom: 16 },
    sectionTitle: { fontSize: 18, fontWeight: '600', color: '#111827', marginBottom: 12 },
    actionItem: { flexDirection: 'row', alignItems: 'center', backgroundColor: '#FFFFFF', padding: 16, borderRadius: 12, marginBottom: 8, shadowColor: '#000', shadowOffset: { width: 0, height: 1 }, shadowOpacity: 0.05, shadowRadius: 2, elevation: 1 },
    actionIcon: { fontSize: 24, marginRight: 12 },
    actionContent: { flex: 1 },
    actionTitle: { fontSize: 16, fontWeight: '600', color: '#111827', marginBottom: 2 },
    actionDescription: { fontSize: 13, color: '#6B7280' },
    actionArrow: { fontSize: 24, color: '#D1D5DB' },
    infoCard: { backgroundColor: '#EEF2FF', borderColor: '#6366F1', borderWidth: 1 },
    infoCardTitle: { fontSize: 16, fontWeight: '600', color: '#3730A3', marginBottom: 8 },
    infoCardText: { fontSize: 14, color: '#4338CA', lineHeight: 20 },
});