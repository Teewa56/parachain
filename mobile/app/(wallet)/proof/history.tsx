import { useState, useEffect } from 'react';
import {
    View,
    Text,
    StyleSheet,
    FlatList,
    TouchableOpacity,
    RefreshControl,
    ActivityIndicator,
} from 'react-native';
import { router } from 'expo-router';
import { useUIStore } from '../../../src/store';
import { Card } from '../../../src/components/common/Card';
import { formatTimestamp, formatRelativeTime } from '../../../src/services/substrate/utils';
import AsyncStorage from '@react-native-async-storage/async-storage';
import { STORAGE_KEYS } from '../../../src/utils/constants';

interface ProofRecord {
    id: string;
    credentialId: string;
    credentialType: string;
    proofType: string;
    fieldsRevealed: number[];
    timestamp: number;
    verifier?: string;
    status: 'active' | 'expired' | 'used';
    expiresAt: number;
}

export default function ProofHistoryScreen() {
    const [proofs, setProofs] = useState<ProofRecord[]>([]);
    const [isLoading, setIsLoading] = useState(true);
    const [refreshing, setRefreshing] = useState(false);
    const [filter, setFilter] = useState<'all' | 'active' | 'expired'>('all');

    const showToast = useUIStore(state => state.showToast);

    useEffect(() => {
        loadProofHistory();
    }, []);

    const loadProofHistory = async () => {
        try {
            const cachedProofs = await AsyncStorage.getItem(STORAGE_KEYS.CACHED_PROOFS);
            if (cachedProofs) {
                const parsedProofs: ProofRecord[] = JSON.parse(cachedProofs);
                
                // Update status based on expiration
                const updatedProofs = parsedProofs.map(proof => ({
                    ...proof,
                    status: proof.expiresAt < Date.now() / 1000 ? 'expired' as const : proof.status,
                }));
                
                setProofs(updatedProofs);
            } else {
                // Generate mock data for demonstration
                setProofs(generateMockProofs());
            }
        } catch (error) {
            console.error('Load proof history failed:', error);
            showToast('Failed to load proof history', 'error');
        } finally {
            setIsLoading(false);
        }
    };

    const generateMockProofs = (): ProofRecord[] => {
        const now = Math.floor(Date.now() / 1000);
        return [
            {
                id: 'proof_1',
                credentialId: '0x123abc',
                credentialType: 'Education',
                proofType: 'StudentStatus',
                fieldsRevealed: [0, 2],
                timestamp: now - 7200, // 2 hours ago
                status: 'active',
                expiresAt: now + 1800, // 30 mins from now
                verifier: 'Campus Store',
            },
            {
                id: 'proof_2',
                credentialId: '0x456def',
                credentialType: 'Health',
                proofType: 'VaccinationStatus',
                fieldsRevealed: [1, 2, 5],
                timestamp: now - 86400, // 1 day ago
                status: 'expired',
                expiresAt: now - 82800, // expired 1 hour ago
                verifier: 'Event Venue',
            },
            {
                id: 'proof_3',
                credentialId: '0x789ghi',
                credentialType: 'Employment',
                proofType: 'EmploymentStatus',
                fieldsRevealed: [1, 2, 5],
                timestamp: now - 259200, // 3 days ago
                status: 'used',
                expiresAt: now - 255600,
                verifier: 'Loan Provider',
            },
        ];
    };

    const onRefresh = async () => {
        setRefreshing(true);
        await loadProofHistory();
        setRefreshing(false);
    };

    const getFilteredProofs = (): ProofRecord[] => {
        if (filter === 'all') return proofs;
        return proofs.filter(proof => proof.status === filter);
    };

    const getStatusColor = (status: string): string => {
        switch (status) {
            case 'active':
                return '#10B981';
            case 'expired':
                return '#6B7280';
            case 'used':
                return '#F59E0B';
            default:
                return '#6B7280';
        }
    };

    const getStatusIcon = (status: string): string => {
        switch (status) {
            case 'active':
                return '✓';
            case 'expired':
                return '⏱';
            case 'used':
                return '📋';
            default:
                return '•';
        }
    };

    const getCredentialIcon = (type: string): string => {
        switch (type) {
            case 'Education':
                return '🎓';
            case 'Health':
                return '🏥';
            case 'Employment':
                return '💼';
            case 'Age':
                return '🎂';
            case 'Address':
                return '🏠';
            default:
                return '📄';
        }
    };

    const handleProofPress = (proof: ProofRecord) => {
        // Navigate to proof details (could be implemented)
        showToast('Proof details coming soon', 'info');
    };

    const renderProof = ({ item }: { item: ProofRecord }) => (
        <TouchableOpacity
            onPress={() => handleProofPress(item)}
            activeOpacity={0.7}
        >
            <Card style={styles.proofCard}>
                <View style={styles.cardHeader}>
                    <View style={styles.typeContainer}>
                        <Text style={styles.typeIcon}>{getCredentialIcon(item.credentialType)}</Text>
                        <View>
                            <Text style={styles.typeName}>{item.proofType}</Text>
                            <Text style={styles.credentialType}>{item.credentialType}</Text>
                        </View>
                    </View>
                    <View
                        style={[
                            styles.statusBadge,
                            { backgroundColor: `${getStatusColor(item.status)}20` },
                        ]}
                    >
                        <Text style={[styles.statusIcon, { color: getStatusColor(item.status) }]}>
                            {getStatusIcon(item.status)}
                        </Text>
                        <Text style={[styles.statusText, { color: getStatusColor(item.status) }]}>
                            {item.status.charAt(0).toUpperCase() + item.status.slice(1)}
                        </Text>
                    </View>
                </View>

                <View style={styles.cardBody}>
                    <View style={styles.infoRow}>
                        <Text style={styles.label}>Generated</Text>
                        <Text style={styles.value}>{formatRelativeTime(item.timestamp)}</Text>
                    </View>

                    <View style={styles.infoRow}>
                        <Text style={styles.label}>Fields Revealed</Text>
                        <Text style={styles.value}>{item.fieldsRevealed.length} fields</Text>
                    </View>

                    {item.verifier && (
                        <View style={styles.infoRow}>
                            <Text style={styles.label}>Verifier</Text>
                            <Text style={styles.value}>{item.verifier}</Text>
                        </View>
                    )}

                    {item.status === 'active' && (
                        <View style={styles.expiryContainer}>
                            <Text style={styles.expiryLabel}>Expires</Text>
                            <Text style={styles.expiryValue}>
                                {formatTimestamp(item.expiresAt)}
                            </Text>
                        </View>
                    )}
                </View>
            </Card>
        </TouchableOpacity>
    );

    const renderEmpty = () => (
        <View style={styles.emptyContainer}>
            <Text style={styles.emptyIcon}>🔍</Text>
            <Text style={styles.emptyTitle}>No Proofs Found</Text>
            <Text style={styles.emptyText}>
                {filter === 'all'
                    ? 'You haven\'t generated any proofs yet. Generate a proof to see it here.'
                    : `No ${filter} proofs found. Try changing the filter.`}
            </Text>
        </View>
    );

    const filteredProofs = getFilteredProofs();

    if (isLoading) {
        return (
            <View style={styles.loadingContainer}>
                <ActivityIndicator size="large" color="#6366F1" />
                <Text style={styles.loadingText}>Loading proof history...</Text>
            </View>
        );
    }

    return (
        <View style={styles.container}>
            <View style={styles.header}>
                <Text style={styles.title}>Proof History</Text>
                <Text style={styles.subtitle}>
                    {filteredProofs.length} {filteredProofs.length === 1 ? 'proof' : 'proofs'}
                </Text>
            </View>

            <View style={styles.filterContainer}>
                {(['all', 'active', 'expired'] as const).map(filterType => (
                    <TouchableOpacity
                        key={filterType}
                        style={[
                            styles.filterButton,
                            filter === filterType && styles.filterButtonActive,
                        ]}
                        onPress={() => setFilter(filterType)}
                    >
                        <Text
                            style={[
                                styles.filterText,
                                filter === filterType && styles.filterTextActive,
                            ]}
                        >
                            {filterType.charAt(0).toUpperCase() + filterType.slice(1)}
                        </Text>
                    </TouchableOpacity>
                ))}
            </View>

            <FlatList
                data={filteredProofs}
                renderItem={renderProof}
                keyExtractor={item => item.id}
                contentContainerStyle={styles.listContent}
                ListEmptyComponent={renderEmpty}
                refreshControl={
                    <RefreshControl
                        refreshing={refreshing}
                        onRefresh={onRefresh}
                        tintColor="#6366F1"
                    />
                }
            />
        </View>
    );
}

const styles = StyleSheet.create({
    container: {
        flex: 1,
        backgroundColor: '#F9FAFB',
    },
    header: {
        padding: 20,
        backgroundColor: '#FFFFFF',
        borderBottomWidth: 1,
        borderBottomColor: '#E5E7EB',
    },
    title: {
        fontSize: 28,
        fontWeight: '700',
        color: '#111827',
        marginBottom: 4,
    },
    subtitle: {
        fontSize: 14,
        color: '#6B7280',
    },
    filterContainer: {
        flexDirection: 'row',
        padding: 16,
        backgroundColor: '#FFFFFF',
        gap: 8,
        borderBottomWidth: 1,
        borderBottomColor: '#E5E7EB',
    },
    filterButton: {
        paddingHorizontal: 16,
        paddingVertical: 8,
        borderRadius: 20,
        backgroundColor: '#F3F4F6',
    },
    filterButtonActive: {
        backgroundColor: '#6366F1',
    },
    filterText: {
        fontSize: 14,
        fontWeight: '600',
        color: '#6B7280',
    },
    filterTextActive: {
        color: '#FFFFFF',
    },
    listContent: {
        padding: 16,
    },
    proofCard: {
        marginBottom: 16,
    },
    cardHeader: {
        flexDirection: 'row',
        justifyContent: 'space-between',
        alignItems: 'center',
        marginBottom: 16,
    },
    typeContainer: {
        flexDirection: 'row',
        alignItems: 'center',
        gap: 12,
    },
    typeIcon: {
        fontSize: 28,
    },
    typeName: {
        fontSize: 16,
        fontWeight: '600',
        color: '#111827',
    },
    credentialType: {
        fontSize: 12,
        color: '#6B7280',
        marginTop: 2,
    },
    statusBadge: {
        flexDirection: 'row',
        alignItems: 'center',
        paddingHorizontal: 10,
        paddingVertical: 6,
        borderRadius: 12,
        gap: 4,
    },
    statusIcon: {
        fontSize: 12,
    },
    statusText: {
        fontSize: 12,
        fontWeight: '600',
    },
    cardBody: {
        gap: 10,
    },
    infoRow: {
        flexDirection: 'row',
        justifyContent: 'space-between',
        alignItems: 'center',
    },
    label: {
        fontSize: 13,
        color: '#6B7280',
    },
    value: {
        fontSize: 13,
        fontWeight: '500',
        color: '#111827',
    },
    expiryContainer: {
        marginTop: 8,
        paddingTop: 12,
        borderTopWidth: 1,
        borderTopColor: '#E5E7EB',
        flexDirection: 'row',
        justifyContent: 'space-between',
        alignItems: 'center',
    },
    expiryLabel: {
        fontSize: 12,
        fontWeight: '600',
        color: '#F59E0B',
    },
    expiryValue: {
        fontSize: 12,
        fontWeight: '600',
        color: '#F59E0B',
    },
    emptyContainer: {
        alignItems: 'center',
        justifyContent: 'center',
        paddingVertical: 60,
    },
    emptyIcon: {
        fontSize: 64,
        marginBottom: 16,
    },
    emptyTitle: {
        fontSize: 20,
        fontWeight: '600',
        color: '#111827',
        marginBottom: 8,
    },
    emptyText: {
        fontSize: 14,
        color: '#6B7280',
        textAlign: 'center',
        paddingHorizontal: 40,
        lineHeight: 20,
    },
    loadingContainer: {
        flex: 1,
        alignItems: 'center',
        justifyContent: 'center',
        backgroundColor: '#F9FAFB',
    },
    loadingText: {
        marginTop: 16,
        fontSize: 16,
        color: '#6B7280',
    },
});
