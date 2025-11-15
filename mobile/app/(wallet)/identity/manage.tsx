import { useState } from 'react';
import { View, Text, StyleSheet, ScrollView, TextInput, Alert } from 'react-native';
import { router } from 'expo-router';
import { useIdentityStore, useUIStore } from '../../../src/store';
import { Card } from '../../../src/components/common/Card';
import { Button } from '../../../src/components/common/Button';

export default function ManageIdentityScreen() {
    const [newPublicKey, setNewPublicKey] = useState('');

    const { identity, updateIdentity, deactivateIdentity, reactivateIdentity } = useIdentityStore();
    const showToast = useUIStore(state => state.showToast);
    const setLoading = useUIStore(state => state.setLoading);

    const handleUpdateKey = async () => {
        if (!newPublicKey.trim()) {
            showToast('Please enter a new public key', 'warning');
            return;
        }

        Alert.alert(
            'Update Public Key',
            'Are you sure you want to update your public key?',
            [
                { text: 'Cancel', style: 'cancel' },
                { text: 'Update', onPress: performUpdate }
            ]
        );
    };

    const performUpdate = async () => {
        setLoading(true, 'Updating identity...');
        
        try {
            await updateIdentity(newPublicKey);
            showToast('Identity updated successfully', 'success');
            setNewPublicKey('');
        } catch (error) {
            showToast('Failed to update identity', 'error');
        } finally {
            setLoading(false);
        }
    };

    const handleDeactivate = () => {
        Alert.alert(
            'Deactivate Identity',
            'Are you sure? You can reactivate it later.',
            [
                { text: 'Cancel', style: 'cancel' },
                { text: 'Deactivate', style: 'destructive', onPress: performDeactivate }
            ]
        );
    };

    const performDeactivate = async () => {
        setLoading(true, 'Deactivating identity...');
        
        try {
            await deactivateIdentity();
            showToast('Identity deactivated', 'success');
            router.back();
        } catch (error) {
            showToast('Failed to deactivate identity', 'error');
        } finally {
            setLoading(false);
        }
    };

    const handleReactivate = async () => {
        setLoading(true, 'Reactivating identity...');
        
        try {
            await reactivateIdentity();
            showToast('Identity reactivated', 'success');
        } catch (error) {
            showToast('Failed to reactivate identity', 'error');
        } finally {
            setLoading(false);
        }
    };

    if (!identity) {
        return (
            <View style={styles.container}>
                <Text style={styles.errorText}>No identity found</Text>
            </View>
        );
    }

    return (
        <ScrollView style={styles.container} contentContainerStyle={styles.scrollContent}>
            <Text style={styles.title}>Manage Identity</Text>

            {/* Update Public Key */}
            <Card style={styles.card}>
                <Text style={styles.cardTitle}>Update Public Key</Text>
                <Text style={styles.cardDescription}>
                    Update the public key associated with your identity.
                </Text>
                
                <Text style={styles.inputLabel}>New Public Key</Text>
                <TextInput
                    style={styles.input}
                    value={newPublicKey}
                    onChangeText={setNewPublicKey}
                    placeholder="0x..."
                    placeholderTextColor="#9CA3AF"
                />
                
                <Button
                    onPress={handleUpdateKey}
                    title="Update Key"
                    disabled={!newPublicKey.trim()}
                />
            </Card>

            {/* Identity Status */}
            <Card style={styles.card}>
                <Text style={styles.cardTitle}>Identity Status</Text>
                <View style={styles.statusRow}>
                    <Text style={styles.statusLabel}>Current Status:</Text>
                    <Text style={[
                        styles.statusValue,
                        { color: identity.active ? '#10B981' : '#EF4444' }
                    ]}>
                        {identity.active ? 'Active' : 'Inactive'}
                    </Text>
                </View>

                {identity.active ? (
                    <Button
                        onPress={handleDeactivate}
                        title="Deactivate Identity"
                        variant="danger"
                    />
                ) : (
                    <Button
                        onPress={handleReactivate}
                        title="Reactivate Identity"
                    />
                )}
            </Card>

            {/* Warning Card */}
            <Card style={styles.warningCard}>
                <Text style={styles.warningTitle}>⚠️ Important</Text>
                <Text style={styles.warningText}>
                    • Deactivating your identity will prevent credential verification{'\n'}
                    • You can reactivate your identity at any time{'\n'}
                    • Your credentials will remain on-chain
                </Text>
            </Card>
        </ScrollView>
    );
}

const styles = StyleSheet.create({
    container: { flex: 1, backgroundColor: '#F9FAFB' },
    scrollContent: { padding: 20 },
    title: { fontSize: 28, fontWeight: '700', color: '#111827', marginBottom: 24 },
    card: { marginBottom: 16 },
    cardTitle: { fontSize: 18, fontWeight: '600', color: '#111827', marginBottom: 8 },
    cardDescription: { fontSize: 14, color: '#6B7280', marginBottom: 16 },
    inputLabel: { fontSize: 14, fontWeight: '600', color: '#111827', marginBottom: 8 },
    input: { backgroundColor: '#F9FAFB', borderWidth: 1, borderColor: '#D1D5DB', borderRadius: 8, padding: 16, fontSize: 16, marginBottom: 16 },
    statusRow: { flexDirection: 'row', justifyContent: 'space-between', alignItems: 'center', marginBottom: 16 },
    statusLabel: { fontSize: 16, color: '#6B7280' },
    statusValue: { fontSize: 16, fontWeight: '600' },
    warningCard: { backgroundColor: '#FEF3C7', borderColor: '#F59E0B', borderWidth: 1 },
    warningTitle: { fontSize: 16, fontWeight: '600', color: '#92400E', marginBottom: 8 },
    warningText: { fontSize: 14, color: '#78350F', lineHeight: 20 },
    errorText: { fontSize: 16, color: '#EF4444', textAlign: 'center', marginTop: 40 },
});
