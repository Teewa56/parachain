import { useEffect, useState } from 'react';
import { 
    View, 
    Text, 
    StyleSheet, 
    FlatList, 
    TouchableOpacity,
    RefreshControl,
    ActivityIndicator 
} from 'react-native';
import { router } from 'expo-router';
import { useCredentialStore } from '../../../src/store/credentialStore';
import { useUIStore } from '../../../src/store/uiStore';
import { Card } from '../../../src/components/common/Card';
import type { Credential, CredentialType } from '../../../src/types/substrate';
import { formatTimestamp, formatDid } from '../../../src/substrate/utils';

export default function CredentialsListScreen() {
    const [refreshing, setRefreshing] = useState(false);
    const [filterType, setFilterType] = useState<CredentialType | 'all'>('all');

    const credentials = useCredentialStore(state => state.credentials);
    const isLoading = useCredentialStore(state => state.isLoading);
    const fetchCredentials = useCredentialStore(state => state.fetchCredentials);
    const filterByType = useCredentialStore(state => state.filterByType);
    const showToast = useUIStore(state => state.showToast);

    useEffect(() => {
        loadCredentials();
    }, []);

    const loadCredentials = async () => {
        try {
            await fetchCredentials();
        } catch (error) {
            console.error('Load credentials failed:', error);
            showToast('Failed to load credentials', 'error');
        }
    };

    const onRefresh = async () => {
        setRefreshing(true);
        await loadCredentials();
        setRefreshing(false);
    };

    const handleCredentialPress = (credential: Credential) => {
        // Navigate to credential details
        router.push({
            pathname: '/(wallet)/credentials/[id]',
            params: { id: credential.subject }
        });
    };

    const getFilteredCredentials = (): Credential[] => {
        if (filterType === 'all') {
            return credentials;
        }
        return filterByType(filterType);
    };

    const getStatusColor = (status: string): string => {
        switch (status) {
            case 'Active':
                return '#10B981';
            case 'Expired':
                return '#F59E0B';
            case 'Revoked':
                return '#EF4444';
            case 'Suspended':
                return '#6B7280';
            default:
                return '#6B7280';
        }
    };

    const getTypeIcon = (type: string): string => {
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

    const renderCredential = ({ item }: { item: Credential }) => (
        <TouchableOpacity
            onPress={() => handleCredentialPress(item)}
            activeOpacity={0.7}
        >
            <Card style={styles.credentialCard}>
                <View style={styles.cardHeader}>
                    <View style={styles.typeContainer}>
                        <Text style={styles.typeIcon}>{getTypeIcon(item.credentialType)}</Text>
                        <Text style={styles.typeText}>{item.credentialType}</Text>
                    </View>
                    <View style={[
                        styles.statusBadge,
                        { backgroundColor: `${getStatusColor(item.status)}20` }
                    ]}>
                        <Text style={[
                            styles.statusText,
                            { color: getStatusColor(item.status) }
                        ]}>
                            {item.status}
                        </Text>
                    </View>
                </View>

                <View style={styles.cardBody}>
                    <Text style={styles.label}>Issuer</Text>
                    <Text style={styles.value}>{formatDid(item.issuer)}</Text>

                    <Text style={styles.label}>Issued</Text>
                    <Text style={styles.value}>{formatTimestamp(item.issuedAt)}</Text>

                    {item.expiresAt > 0 && (
                        <>
                            <Text style={styles.label}>Expires</Text>
                            <Text style={styles.value}>{formatTimestamp(item.expiresAt)}</Text>
                        </>
                    )}
                </View>
            </Card>
        </TouchableOpacity>
    );

    const renderEmpty = () => (
        <View style={styles.emptyContainer}>
            <Text style={styles.emptyIcon}>📭</Text>
            <Text style={styles.emptyTitle}>No Credentials</Text>
            <Text style={styles.emptyText}>
                You don't have any credentials yet. Credentials will appear here once issued by trusted organizations.
            </Text>
        </View>
    );

    if (isLoading && !refreshing && credentials.length === 0) {
        return (
            <View style={styles.loadingContainer}>
                <ActivityIndicator size="large" color="#6366F1" />
                <Text style={styles.loadingText}>Loading credentials...</Text>
            </View>
        );
    }

    const filteredCredentials = getFilteredCredentials();

    return (
        <View style={styles.container}>
            <View style={styles.header}>
                <Text style={styles.title}>My Credentials</Text>
                <Text style={styles.subtitle}>
                    {filteredCredentials.length} {filteredCredentials.length === 1 ? 'credential' : 'credentials'}
                </Text>
            </View>

            <View style={styles.filterContainer}>
                <TouchableOpacity
                    style={[
                        styles.filterButton,
                        filterType === 'all' && styles.filterButtonActive
                    ]}
                    onPress={() => setFilterType('all')}
                >
                    <Text style={[
                        styles.filterText,
                        filterType === 'all' && styles.filterTextActive
                    ]}>
                        All
                    </Text>
                </TouchableOpacity>

                {['Education', 'Health', 'Employment', 'Age', 'Address'].map((type) => (
                    <TouchableOpacity
                        key={type}
                        style={[
                            styles.filterButton,
                            filterType === type && styles.filterButtonActive
                        ]}
                        onPress={() => setFilterType(type as CredentialType)}
                    >
                        <Text style={[
                            styles.filterText,
                            filterType === type && styles.filterTextActive
                        ]}>
                            {type}
                        </Text>
                    </TouchableOpacity>
                ))}
            </View>

            <FlatList
                data={filteredCredentials}
                renderItem={renderCredential}
                keyExtractor={(item, index) => `${item.subject}-${index}`}
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
    credentialCard: {
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
        gap: 8,
    },
    typeIcon: {
        fontSize: 24,
    },
    typeText: {
        fontSize: 18,
        fontWeight: '600',
        color: '#111827',
    },
    statusBadge: {
        paddingHorizontal: 12,
        paddingVertical: 6,
        borderRadius: 12,
    },
    statusText: {
        fontSize: 12,
        fontWeight: '600',
    },
    cardBody: {
        gap: 8,
    },
    label: {
        fontSize: 12,
        fontWeight: '600',
        color: '#6B7280',
        textTransform: 'uppercase',
        marginTop: 8,
    },
    value: {
        fontSize: 14,
        color: '#111827',
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