import React from 'react';
import { View, Text, StyleSheet, TouchableOpacity } from 'react-native';
import { router } from 'expo-router';
import { Card } from '../common/Card';
import type { Credential } from '../../types/substrate';
import { formatTimestamp, formatDid } from '../../substrate/utils';

interface CredentialCardProps {
    credential: Credential;
}

export const CredentialCard: React.FC<CredentialCardProps> = ({ credential }) => {
    const statusColor = {
        Active: '#10B981',
        Revoked: '#EF4444',
        Expired: '#F59E0B',
        Suspended: '#6B7280',
    }[credential.status] || '#6B7280';

    const typeIcon = {
        Education: '🎓',
        Health: '🏥',
        Employment: '💼',
        Age: '🎂',
        Address: '🏠',
    }[credential.credentialType] || '📄';

    return (
        <TouchableOpacity
            onPress={() => router.push({
                pathname: '/(wallet)/credentials/[id]',
                params: { id: credential.subject },
            })}
            activeOpacity={0.7}
        >
            <Card style={styles4.card}>
                <View style={styles4.header}>
                    <View style={styles4.typeContainer}>
                        <Text style={styles4.icon}>{typeIcon}</Text>
                        <Text style={styles4.type}>{credential.credentialType}</Text>
                    </View>
                    <View style={[styles4.statusBadge, { backgroundColor: `${statusColor}20` }]}>
                        <Text style={[styles4.statusText, { color: statusColor }]}>
                            {credential.status}
                        </Text>
                    </View>
                </View>
                <Text style={styles4.label}>Issuer</Text>
                <Text style={styles4.value}>{formatDid(credential.issuer)}</Text>
                <Text style={styles4.label}>Issued</Text>
                <Text style={styles4.value}>{formatTimestamp(credential.issuedAt)}</Text>
            </Card>
        </TouchableOpacity>
    );
};

const styles4 = StyleSheet.create({
    card: { marginBottom: 12 },
    header: { flexDirection: 'row', justifyContent: 'space-between', alignItems: 'center', marginBottom: 12 },
    typeContainer: { flexDirection: 'row', alignItems: 'center', gap: 8 },
    icon: { fontSize: 24 },
    type: { fontSize: 16, fontWeight: '600', color: '#111827' },
    statusBadge: { paddingHorizontal: 10, paddingVertical: 4, borderRadius: 12 },
    statusText: { fontSize: 11, fontWeight: '600' },
    label: { fontSize: 12, fontWeight: '600', color: '#6B7280', marginTop: 8 },
    value: { fontSize: 14, color: '#111827', marginTop: 2 },
});