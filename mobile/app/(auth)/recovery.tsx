import { useState } from 'react';
import { View, Text, StyleSheet, TextInput, ScrollView } from 'react-native';
import { router } from 'expo-router';
import { useAuthStore, useIdentityStore, useUIStore } from '../../src/store';
import { keyManagement } from '../../src/services/crypto/keyManagement';
import { Button } from '../../src/components/common/Button';
import { Card } from '../../src/components/common/Card';

export default function RecoveryScreen() {
    const [mnemonic, setMnemonic] = useState('');

    const { login, setHasSeedPhrase } = useAuthStore();
    const importIdentity = useIdentityStore(state => state.importIdentity);
    const showToast = useUIStore(state => state.showToast);
    const setLoading = useUIStore(state => state.setLoading);

    const handleRecover = async () => {
        const trimmedMnemonic = mnemonic.trim().toLowerCase();
        
        if (!keyManagement.validateSeedPhrase(trimmedMnemonic)) {
        showToast('Invalid seed phrase', 'error');
        return;
        }

        setLoading(true, 'Recovering your identity...');

        try {
        await importIdentity(trimmedMnemonic, 'Recovered Identity');
        setHasSeedPhrase(true);
        await login();
        
        showToast('Identity recovered successfully!', 'success');
        router.replace('/(wallet)');
        } catch (error) {
        showToast('Failed to recover identity', 'error');
        } finally {
        setLoading(false);
        }
    };

    return (
        <View style={styles.container}>
        <ScrollView contentContainerStyle={styles.scrollContent}>
            <Text style={styles.title}>Recover Identity</Text>
            <Text style={styles.description}>
            Enter your 12-word seed phrase to recover your identity.
            </Text>

            <Card style={styles.inputCard}>
            <Text style={styles.inputLabel}>Seed Phrase</Text>
            <TextInput
                style={styles.input}
                value={mnemonic}
                onChangeText={setMnemonic}
                placeholder="word1 word2 word3 ..."
                placeholderTextColor="#9CA3AF"
                multiline
                numberOfLines={4}
                autoCapitalize="none"
                autoCorrect={false}
            />
            <Text style={styles.hint}>
                Enter all 12 words separated by spaces
            </Text>
            </Card>

            <Card style={styles.warningCard}>
            <Text style={styles.warningTitle}>🔒 Security Notice</Text>
            <Text style={styles.warningText}>
                Make sure you're in a private location. Never enter your seed phrase on a public or shared device.
            </Text>
            </Card>

            <Button
            onPress={handleRecover}
            title="Recover Identity"
            disabled={!mnemonic.trim()}
            style={styles.button}
            />

            <Button
            onPress={() => router.back()}
            title="Cancel"
            variant="secondary"
            style={styles.cancelButton}
            />
        </ScrollView>
        </View>
    );
}

const styles = StyleSheet.create({
  container: { flex: 1, backgroundColor: '#FFFFFF' },
  scrollContent: { padding: 20 },
  title: { fontSize: 28, fontWeight: '700', color: '#111827', marginBottom: 12 },
  description: { fontSize: 16, color: '#6B7280', marginBottom: 24, lineHeight: 24 },
  inputCard: { marginBottom: 20 },
  inputLabel: { fontSize: 14, fontWeight: '600', color: '#111827', marginBottom: 8 },
  input: { backgroundColor: '#F9FAFB', borderWidth: 1, borderColor: '#D1D5DB', borderRadius: 8, padding: 16, fontSize: 16, minHeight: 120, textAlignVertical: 'top', marginBottom: 8 },
  hint: { fontSize: 12, color: '#6B7280' },
  warningCard: { marginBottom: 24, backgroundColor: '#EEF2FF', borderColor: '#6366F1', borderWidth: 1 },
  warningTitle: { fontSize: 16, fontWeight: '600', color: '#3730A3', marginBottom: 8 },
  warningText: { fontSize: 14, color: '#4338CA', lineHeight: 20 },
  button: { marginBottom: 12 },
  cancelButton: {},
});