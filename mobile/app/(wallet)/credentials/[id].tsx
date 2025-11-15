import { useEffect, useState } from 'react';
import { View, Text, StyleSheet, ScrollView, TouchableOpacity, Alert } from 'react-native';
import { useLocalSearchParams, router } from 'expo-router';
import { useCredentialStore, useIdentityStore, useUIStore } from '../../../src/store';
import { Card } from '../../../src/components/common/Card';
import { Button } from '../../../src/components/common/Button';
import { formatTimestamp, formatDid, getExpiryStatus } from '../../../src/substrate/utils';
import type { Credential } from '../../../src/types/substrate';

export default function CredentialDetailScreen() {
    const params = useLocalSearchParams();
    const credentialId = params.id as string;

    const getCredentialById = useCredentialStore(state => state.getCredentialById);
    const refreshCredential = useCredentialStore(state => state.refreshCredential);
    const revokeCredential = useCredentialStore(state => state.revokeCredential);
    const { keyPair } = useIdentityStore();
    const showToast = useUIStore(state => state.showToast);
    const setLoading = useUIStore(state => state.setLoading);

    const [credential, setCredential] = useState<Credential | null>(null);
    const [refreshing, setRefreshing] = useState(false);

    useEffect(() => {
        loadCredential();
    }, [credentialId]);

    const loadCredential = () => {
        const cred = getCredentialById(credentialId);
        setCredential(cred);
    };

    const handleRefresh = async () => {
        if (!credential) return;
        
        setRefreshing(true);
        try {
            await refreshCredential(credential.subject);
            loadCredential();
            showToast('Credential refreshed', 'success');
        } catch (error) {
            showToast('Failed to refresh credential', 'error');
        } finally {
            setRefreshing(false);
        }
    };

    const handleRevoke = () => {
        if (!credential) return;

        Alert.alert(
            'Revoke Credential',
            'Are you sure you want to revoke this credential? This action cannot be undone.',
            [
                { text: 'Cancel', style: 'cancel' },
                {
                    text: 'Revoke',
                    style: 'destructive',
                    onPress: performRevoke,
                },
            ]
        );
    };

    const performRevoke = async () => {
        if (!credential || !keyPair) return;

        setLoading(true, 'Revoking credential...');
        try {
            await revokeCredential(credential.subject);
            showToast('Credential revoked successfully', 'success');
            router.back();
        } catch (error) {
            showToast('Failed to revoke credential', 'error');
        } finally {
            setLoading(false);
        }
    };

    const handleGenerateProof = () => {
        if (!credential) return;

        router.push({
            pathname: '/(wallet)/proof',
            params: { credentialId: credential.subject },
        });
    };

    const handleShare = () => {
        if (!credential) return;

        router.push({
            pathname: '/(wallet)/credentials/share',
            params: { credentialId: credential.subject },
        });
    };

    if (!credential) {
        return (
            <View style={styles.container}>
                <View style={styles.errorContainer}>
                    <Text style={styles.errorIcon}>⚠️</Text>
                    <Text style={styles.errorTitle}>Credential Not Found</Text>
                    <Text style={styles.errorText}>
                        The credential you're looking for doesn't exist or has been removed.
                    </Text>
                    <Button
                        onPress={() => router.back()}
                        title="Go Back"
                        variant="secondary"
                    />
                </View>
            </View>
        );
    }

    const expiryStatus = getExpiryStatus(credential.expiresAt);
    const statusColor = {
        Active: '#10B981',
        Revoked: '#EF4444',
        Expired: '#F59E0B',
        Suspended: '#6B7280',
    }[credential.status] || '#6B7280';

    return (
        <View style={styles.container}>
            <ScrollView contentContainerStyle={styles.scrollContent}>
                {/* Header Card */}
                <Card style={styles.headerCard}>
                    <View style={styles.typeContainer}>
                        <Text style={styles.typeIcon}>{getTypeIcon(credential.credentialType)}</Text>
                        <View style={styles.typeInfo}>
                            <Text style={styles.typeName}>{credential.credentialType}</Text>
                            <Text style={styles.typeSubtitle}>Verifiable Credential</Text>
                        </View>
                    </View>
                    <View style={[styles.statusBadge, { backgroundColor: `${statusColor}20` }]}>
                        <Text style={[styles.statusText, { color: statusColor }]}>
                            {credential.status}
                        </Text>
                    </View>
                </Card>

                {/* Expiry Warning */}
                {credential.status === 'Active' && expiryStatus.expiringSoon && (
                    <Card style={styles.warningCard}>
                        <Text style={styles.warningTitle}>⚠️ Expiring Soon</Text>
                        <Text style={styles.warningText}>
                            This credential will expire in {expiryStatus.daysRemaining} days
                        </Text>
                    </Card>
                )}

                {/* Issuer Information */}
                <Card style={styles.card}>
                    <Text style={styles.cardTitle}>Issuer Information</Text>
                    <View style={styles.infoRow}>
                        <Text style={styles.label}>Issuer DID</Text>
                        <Text style={styles.value}>{formatDid(credential.issuer)}</Text>
                    </View>
                </Card>

                {/* Credential Details */}
                <Card style={styles.card}>
                    <Text style={styles.cardTitle}>Credential Details</Text>
                    
                    <View style={styles.infoRow}>
                        <Text style={styles.label}>Subject DID</Text>
                        <Text style={styles.value}>{formatDid(credential.subject)}</Text>
                    </View>

                    <View style={styles.infoRow}>
                        <Text style={styles.label}>Issued Date</Text>
                        <Text style={styles.value}>{formatTimestamp(credential.issuedAt)}</Text>
                    </View>

                    {credential.expiresAt > 0 && (
                        <View style={styles.infoRow}>
                            <Text style={styles.label}>Expires</Text>
                            <Text style={styles.value}>{formatTimestamp(credential.expiresAt)}</Text>
                        </View>
                    )}

                    <View style={styles.infoRow}>
                        <Text style={styles.label}>Data Hash</Text>
                        <Text style={styles.valueMonospace}>
                            {credential.dataHash.slice(0, 16)}...
                        </Text>
                    </View>
                </Card>

                {/* Actions */}
                {credential.status === 'Active' && (
                    <View style={styles.actionsSection}>
                        <Text style={styles.sectionTitle}>Actions</Text>
                        
                        <Button
                            onPress={handleGenerateProof}
                            title="Generate Zero-Knowledge Proof"
                            style={styles.actionButton}
                        />

                        <Button
                            onPress={handleShare}
                            title="Share Credential"
                            variant="secondary"
                            style={styles.actionButton}
                        />

                        <Button
                            onPress={handleRefresh}
                            title="Refresh from Chain"
                            variant="secondary"
                            loading={refreshing}
                            style={styles.actionButton}
                        />

                        <Button
                            onPress={handleRevoke}
                            title="Revoke Credential"
                            variant="danger"
                            style={styles.actionButton}
                        />
                    </View>
                )}

                {/* Technical Info */}
                <Card style={styles.technicalCard}>
                    <Text style={styles.cardTitle}>Technical Information</Text>
                    
                    <View style={styles.technicalRow}>
                        <Text style={styles.technicalLabel}>Signature</Text>
                        <Text style={styles.technicalValue}>
                            {credential.signature.slice(0, 20)}...
                        </Text>
                    </View>

                    <View style={styles.technicalRow}>
                        <Text style={styles.technicalLabel}>Metadata Hash</Text>
                        <Text style={styles.technicalValue}>
                            {credential.metadataHash.slice(0, 20)}...
                        </Text>
                    </View>
                </Card>
            </ScrollView>
        </View>
    );
}

function getTypeIcon(type: string): string {
    const icons: Record<string, string> = {
        Education: '🎓',
        Health: '🏥',
        Employment: '💼',
        Age: '🎂',
        Address: '🏠',
        Custom: '📄',
    };
    return icons[type] || '📄';
}

const styles = StyleSheet.create({
    container: { flex: 1, backgroundColor: '#F9FAFB' },
    scrollContent: { padding: 20 },
    headerCard: { marginBottom: 16 },
    typeContainer: { flexDirection: 'row', alignItems: 'center', marginBottom: 16 },
    typeIcon: { fontSize: 48, marginRight: 16 },
    typeInfo: { flex: 1 },
    typeName: { fontSize: 24, fontWeight: '700', color: '#111827', marginBottom: 4 },
    typeSubtitle: { fontSize: 14, color: '#6B7280' },
    statusBadge: { paddingHorizontal: 16, paddingVertical: 8, borderRadius: 16, alignSelf: 'flex-start' },
    statusText: { fontSize: 14, fontWeight: '600' },
    warningCard: { marginBottom: 16, backgroundColor: '#FEF3C7', borderColor: '#F59E0B', borderWidth: 1 },
    warningTitle: { fontSize: 16, fontWeight: '600', color: '#92400E', marginBottom: 8 },
    warningText: { fontSize: 14, color: '#78350F' },
    card: { marginBottom: 16 },
    cardTitle: { fontSize: 18, fontWeight: '600', color: '#111827', marginBottom: 16 },
    infoRow: { flexDirection: 'row', justifyContent: 'space-between', alignItems: 'flex-start', paddingVertical: 12, borderBottomWidth: 1, borderBottomColor: '#E5E7EB' },
    label: { fontSize: 14, fontWeight: '600', color: '#6B7280', flex: 1 },
    value: { fontSize: 14, color: '#111827', flex: 1, textAlign: 'right' },
    valueMonospace: { fontSize: 12, color: '#111827', fontFamily: 'monospace', flex: 1, textAlign: 'right' },
    actionsSection: { marginBottom: 16 },
    sectionTitle: { fontSize: 18, fontWeight: '600', color: '#111827', marginBottom: 12 },
    actionButton: { marginBottom: 12 },
    technicalCard: { marginBottom: 32, backgroundColor: '#F9FAFB' },
    technicalRow: { flexDirection: 'row', justifyContent: 'space-between', paddingVertical: 8 },
    technicalLabel: { fontSize: 12, color: '#6B7280', flex: 1 },
    technicalValue: { fontSize: 12, fontFamily: 'monospace', color: '#111827', flex: 1, textAlign: 'right' },
    errorContainer: { flex: 1, alignItems: 'center', justifyContent: 'center', padding: 40 },
    errorIcon: { fontSize: 64, marginBottom: 16 },
    errorTitle: { fontSize: 20, fontWeight: '600', color: '#111827', marginBottom: 8, textAlign: 'center' },
    errorText: { fontSize: 14, color: '#6B7280', textAlign: 'center', marginBottom: 24, lineHeight: 20 },
});