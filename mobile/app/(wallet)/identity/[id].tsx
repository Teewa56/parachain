import { useEffect, useState } from 'react';
import { View, Text, StyleSheet, ScrollView } from 'react-native';
import { useLocalSearchParams } from 'expo-router';
import { useIdentityStore } from '../../../src/store';
import { Card } from '../../../src/components/common/Card';
import { IdentityCard } from '../../../src/components/identity/IdentityCard';
import { formatTimestamp } from '../../../src/substrate/utils';

export default function IdentityDetailScreen() {
    const params = useLocalSearchParams();
    const identityId = params.id as string;

    const { identity, didDocument, did } = useIdentityStore();

    if (!identity) {
        return (
            <View style={styles.container}>
                <View style={styles.errorContainer}>
                    <Text style={styles.errorIcon}>⚠️</Text>
                    <Text style={styles.errorTitle}>Identity Not Found</Text>
                </View>
            </View>
        );
    }

    return (
        <ScrollView style={styles.container} contentContainerStyle={styles.scrollContent}>
            <IdentityCard identity={identity} did={did} />

            {/* DID Document */}
            {didDocument && (
                <Card style={styles.card}>
                    <Text style={styles.cardTitle}>DID Document</Text>
                    
                    <View style={styles.section}>
                        <Text style={styles.sectionLabel}>Public Keys</Text>
                        {didDocument.publicKeys.map((key, index) => (
                            <Text key={index} style={styles.valueText}>
                                {key.slice(0, 20)}...
                            </Text>
                        ))}
                    </View>

                    <View style={styles.section}>
                        <Text style={styles.sectionLabel}>Authentication Methods</Text>
                        <Text style={styles.valueText}>
                            {didDocument.authentication.length} method(s)
                        </Text>
                    </View>

                    {didDocument.services.length > 0 && (
                        <View style={styles.section}>
                            <Text style={styles.sectionLabel}>Services</Text>
                            {didDocument.services.map((service, index) => (
                                <Text key={index} style={styles.valueText}>{service}</Text>
                            ))}
                        </View>
                    )}
                </Card>
            )}

            {/* Technical Details */}
            <Card style={styles.card}>
                <Text style={styles.cardTitle}>Technical Details</Text>
                
                <View style={styles.detailRow}>
                    <Text style={styles.detailLabel}>Controller Address</Text>
                    <Text style={styles.detailValue}>{identity.controller}</Text>
                </View>

                <View style={styles.detailRow}>
                    <Text style={styles.detailLabel}>Public Key</Text>
                    <Text style={styles.detailValue}>
                        {identity.publicKey.slice(0, 16)}...
                    </Text>
                </View>

                <View style={styles.detailRow}>
                    <Text style={styles.detailLabel}>Created At</Text>
                    <Text style={styles.detailValue}>
                        {formatTimestamp(identity.createdAt)}
                    </Text>
                </View>

                <View style={styles.detailRow}>
                    <Text style={styles.detailLabel}>Last Updated</Text>
                    <Text style={styles.detailValue}>
                        {formatTimestamp(identity.updatedAt)}
                    </Text>
                </View>

                <View style={styles.detailRow}>
                    <Text style={styles.detailLabel}>Status</Text>
                    <Text style={[
                        styles.detailValue,
                        { color: identity.active ? '#10B981' : '#EF4444' }
                    ]}>
                        {identity.active ? 'Active' : 'Inactive'}
                    </Text>
                </View>
            </Card>
        </ScrollView>
    );
}

const styles = StyleSheet.create({
    container: { flex: 1, backgroundColor: '#F9FAFB' },
    scrollContent: { padding: 20 },
    card: { marginBottom: 16 },
    cardTitle: { fontSize: 18, fontWeight: '600', color: '#111827', marginBottom: 16 },
    section: { marginBottom: 16, paddingBottom: 16, borderBottomWidth: 1, borderBottomColor: '#E5E7EB' },
    sectionLabel: { fontSize: 14, fontWeight: '600', color: '#6B7280', marginBottom: 8, textTransform: 'uppercase' },
    valueText: { fontSize: 14, color: '#111827', marginBottom: 4, fontFamily: 'monospace' },
    detailRow: { flexDirection: 'row', justifyContent: 'space-between', paddingVertical: 12, borderTopWidth: 1, borderTopColor: '#E5E7EB' },
    detailLabel: { fontSize: 14, color: '#6B7280', flex: 1 },
    detailValue: { fontSize: 14, fontWeight: '500', color: '#111827', flex: 1, textAlign: 'right' },
    errorContainer: { flex: 1, alignItems: 'center', justifyContent: 'center', padding: 40 },
    errorIcon: { fontSize: 64, marginBottom: 16 },
    errorTitle: { fontSize: 20, fontWeight: '600', color: '#111827', textAlign: 'center' },
});