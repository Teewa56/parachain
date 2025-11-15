import React from 'react';
import { View, Text, StyleSheet } from 'react-native';
import { Card } from '../common/Card';
import type { Identity } from '../../types/substrate';
import { formatTimestamp } from '../../substrate/utils';

interface IdentityCardProps {
    identity: Identity;
    did: string | null;
}

export const IdentityCard: React.FC<IdentityCardProps> = ({ identity, did }) => {
    return (
        <Card style={styles6.card}>
            <View style={styles6.header}>
                <Text style={styles6.title}>Identity Information</Text>
                <View style={[
                    styles6.statusBadge,
                    { backgroundColor: identity.active ? '#D1FAE5' : '#FEE2E2' }
                ]}>
                    <Text style={[
                        styles6.statusText,
                        { color: identity.active ? '#065F46' : '#991B1B' }
                    ]}>
                        {identity.active ? 'Active' : 'Inactive'}
                    </Text>
                </View>
            </View>
            {did && (
                <>
                    <Text style={styles6.label}>DID</Text>
                    <Text style={styles6.value} numberOfLines={1}>{did}</Text>
                </>
            )}
            <Text style={styles6.label}>Controller</Text>
            <Text style={styles6.value}>{identity.controller.slice(0, 10)}...</Text>
            <Text style={styles6.label}>Created</Text>
            <Text style={styles6.value}>{formatTimestamp(identity.createdAt)}</Text>
            <Text style={styles6.label}>Last Updated</Text>
            <Text style={styles6.value}>{formatTimestamp(identity.updatedAt)}</Text>
        </Card>
    );
};

const styles6 = StyleSheet.create({
    card: {},
    header: { flexDirection: 'row', justifyContent: 'space-between', alignItems: 'center', marginBottom: 16 },
    title: { fontSize: 18, fontWeight: '600', color: '#111827' },
    statusBadge: { paddingHorizontal: 12, paddingVertical: 6, borderRadius: 12 },
    statusText: { fontSize: 12, fontWeight: '600' },
    label: { fontSize: 12, fontWeight: '600', color: '#6B7280', marginTop: 12 },
    value: { fontSize: 14, color: '#111827', marginTop: 4 },
});