import { useState } from 'react';
import { View, Text, StyleSheet, ScrollView, Alert } from 'react-native';
import { router } from 'expo-router';
import { useAuthStore, useIdentityStore, useUIStore } from '../../src/store';
import { keyManagement } from '../../src/services/crypto/keyManagement';
import { biometricAuth } from '../../src/services/storage/biometric';
import { Button } from '../../src/components/common/Button';
import { Card } from '../../src/components/common/Card';
import * as ScreenCapture from 'expo-screen-capture';

export default function RegisterScreen() {
    const [mnemonic, setMnemonic] = useState<string | null>(null);
    const [hasBackedUp, setHasBackedUp] = useState(false);
    ScreenCapture.usePreventScreenCapture();
    const { login, setHasSeedPhrase } = useAuthStore();
    const createIdentity = useIdentityStore(state => state.createIdentity);
    const showToast = useUIStore(state => state.showToast);
    const setLoading = useUIStore(state => state.setLoading);

    const handleGenerateMnemonic = () => {
        try {
        const newMnemonic = keyManagement.generateSeedPhrase();
        setMnemonic(newMnemonic);
        } catch (error) {
        showToast('Failed to generate seed phrase', 'error');
        }
    };

    const handleConfirmBackup = () => {
        Alert.alert(
        'Confirm Backup',
        'Have you safely stored your seed phrase?',
        [
            { text: 'No', style: 'cancel' },
            { text: 'Yes', onPress: () => setHasBackedUp(true) }
        ]
        );
    };

    const handleComplete = async () => {
        if (!mnemonic || !hasBackedUp) return;

        setLoading(true, 'Setting up your identity...');

        try {
        await createIdentity(mnemonic, 'My Identity');
        setHasSeedPhrase(true);
        await login();

        // Prompt for PIN setup
        router.push('/(wallet)/settings/backup');
        showToast('Identity created successfully!', 'success');
        } catch (error) {
        showToast('Failed to create identity', 'error');
        } finally {
        setLoading(false);
        }
    };

    if (!mnemonic) {
        return (
        <View style={styles.container}>
            <ScrollView contentContainerStyle={styles.scrollContent}>
            <Text style={styles.title}>Create Identity</Text>
            <Text style={styles.description}>
                Generate a new decentralized identity with a secure seed phrase.
            </Text>

            <Card style={styles.infoCard}>
                <Text style={styles.infoTitle}>What is a Seed Phrase?</Text>
                <Text style={styles.infoText}>
                Your seed phrase is a 12-word key that gives you complete control over your identity. Never share it with anyone.
                </Text>
            </Card>

            <Button onPress={handleGenerateMnemonic} title="Generate Seed Phrase" />
            </ScrollView>
        </View>
        );
    }

    return (
        <View style={styles.container}>
        <ScrollView contentContainerStyle={styles.scrollContent}>
            <Text style={styles.title}>Your Seed Phrase</Text>
            <Text style={styles.description}>Write down these 12 words in order:</Text>

            <Card style={styles.mnemonicCard}>
            {mnemonic.split(' ').map((word, index) => (
                <View key={index} style={styles.wordRow}>
                <Text style={styles.wordNumber}>{index + 1}.</Text>
                <Text style={styles.word}>{word}</Text>
                </View>
            ))}
            </Card>

            <Card style={styles.warningCard}>
                <Text style={styles.warningTitle}>⚠️ Important</Text>
                <Text style={styles.warningText}>
                    • Write these words on paper{'\n'}
                    • Store in a secure location{'\n'}
                    • Never take a screenshot{'\n'}
                    • Never share with anyone
                </Text>
            </Card>

            {!hasBackedUp ? (
            <Button onPress={handleConfirmBackup} title="I've Backed Up My Seed Phrase" />
            ) : (
            <Button onPress={handleComplete} title="Complete Setup" />
            )}
        </ScrollView>
        </View>
    );
}

const styles = StyleSheet.create({
  container: { flex: 1, backgroundColor: '#FFFFFF' },
  scrollContent: { padding: 20 },
  title: { fontSize: 28, fontWeight: '700', color: '#111827', marginBottom: 12 },
  description: { fontSize: 16, color: '#6B7280', marginBottom: 24, lineHeight: 24 },
  infoCard: { marginBottom: 24, backgroundColor: '#EEF2FF', borderColor: '#6366F1', borderWidth: 1 },
  infoTitle: { fontSize: 18, fontWeight: '600', color: '#111827', marginBottom: 8 },
  infoText: { fontSize: 14, color: '#4B5563', lineHeight: 20 },
  mnemonicCard: { marginBottom: 20, padding: 20, backgroundColor: '#F9FAFB' },
  wordRow: { flexDirection: 'row', alignItems: 'center', marginBottom: 12 },
  wordNumber: { fontSize: 14, fontWeight: '600', color: '#6B7280', width: 30 },
  word: { fontSize: 16, fontWeight: '600', color: '#111827', fontFamily: 'monospace' },
  warningCard: { marginBottom: 24, backgroundColor: '#FEF3C7', borderColor: '#F59E0B', borderWidth: 1 },
  warningTitle: { fontSize: 16, fontWeight: '600', color: '#92400E', marginBottom: 8 },
  warningText: { fontSize: 14, color: '#78350F', lineHeight: 20 },
});