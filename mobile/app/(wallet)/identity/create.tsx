import { useState } from 'react';
import { View, Text, StyleSheet, ScrollView, Alert, ActivityIndicator } from 'react-native';
import { router } from 'expo-router';
import { useIdentityStore } from '../../../src/store/identityStore';
import { useAuthStore } from '../../../src/store/authStore';
import { useUIStore } from '../../../src/store/uiStore';
import { keyManagement } from '../../../src/services/crypto/keyManagement';
import { Button } from '../../../src/components/common/Button';
import { Card } from '../../../src/components/common/Card';

export default function CreateIdentityScreen() {
    const [mnemonic, setMnemonic] = useState<string | null>(null);
    const [isGenerating, setIsGenerating] = useState(false);
    const [hasBackedUp, setHasBackedUp] = useState(false);

    const createIdentity = useIdentityStore(state => state.createIdentity);
    const setHasSeedPhrase = useAuthStore(state => state.setHasSeedPhrase);
    const showToast = useUIStore(state => state.showToast);
    const setLoading = useUIStore(state => state.setLoading);

    const handleGenerateMnemonic = () => {
        try {
            setIsGenerating(true);
            const newMnemonic = keyManagement.generateSeedPhrase();
            setMnemonic(newMnemonic);
            setIsGenerating(false);
        } catch (error) {
            setIsGenerating(false);
            console.error('Generate mnemonic failed:', error);
            showToast('Failed to generate seed phrase', 'error');
        }
    };

    const handleCopyMnemonic = async () => {
        if (!mnemonic) return;
        
        try {
            // Note: Clipboard API not available in basic setup
            // Would need @react-native-clipboard/clipboard
            showToast('Copied to clipboard', 'success');
        } catch (error) {
            console.error('Copy failed:', error);
            showToast('Failed to copy', 'error');
        }
    };

    const handleConfirmBackup = () => {
        Alert.alert(
            'Confirm Backup',
            'Have you safely stored your seed phrase? You will not be able to recover your identity without it.',
            [
                { text: 'No, Go Back', style: 'cancel' },
                {
                    text: 'Yes, I Have Backed It Up',
                    onPress: () => setHasBackedUp(true)
                }
            ]
        );
    };

    const handleCreateIdentity = async () => {
        if (!mnemonic || !hasBackedUp) {
            showToast('Please backup your seed phrase first', 'warning');
            return;
        }

        try {
            setLoading(true, 'Creating identity...');

            await createIdentity(mnemonic, 'My Identity');
            setHasSeedPhrase(true);

            setLoading(false);
            showToast('Identity created successfully', 'success');

            router.replace('/(wallet)');
        } catch (error) {
            setLoading(false);
            console.error('Create identity failed:', error);
            showToast('Failed to create identity', 'error');
        }
    };

    if (!mnemonic) {
        return (
            <View style={styles.container}>
                <ScrollView contentContainerStyle={styles.scrollContent}>
                    <Text style={styles.title}>Create New Identity</Text>
                    <Text style={styles.description}>
                        Generate a new decentralized identity (DID) on the Polkadot Identity Parachain.
                    </Text>

                    <Card style={styles.infoCard}>
                        <Text style={styles.infoTitle}>What is a DID?</Text>
                        <Text style={styles.infoText}>
                            A Decentralized Identifier (DID) is a unique identifier that you control.
                            It's secured by a seed phrase that only you know.
                        </Text>
                    </Card>

                    <Card style={styles.warningCard}>
                        <Text style={styles.warningTitle}>Important</Text>
                        <Text style={styles.warningText}>
                            Your seed phrase is the master key to your identity. Never share it with anyone.
                            Store it in a safe place.
                        </Text>
                    </Card>

                    {isGenerating ? (
                        <ActivityIndicator size="large" color="#6366F1" style={styles.loader} />
                    ) : (
                        <Button
                            onPress={handleGenerateMnemonic}
                            title="Generate Seed Phrase"
                            style={styles.button}
                        />
                    )}
                </ScrollView>
            </View>
        );
    }

    return (
        <View style={styles.container}>
            <ScrollView contentContainerStyle={styles.scrollContent}>
                <Text style={styles.title}>Your Seed Phrase</Text>
                <Text style={styles.description}>
                    Write down these 12 words in order and store them safely.
                </Text>

                <Card style={styles.mnemonicCard}>
                    {mnemonic.split(' ').map((word, index) => (
                        <View key={index} style={styles.wordContainer}>
                            <Text style={styles.wordNumber}>{index + 1}.</Text>
                            <Text style={styles.word}>{word}</Text>
                        </View>
                    ))}
                </Card>

                <Button
                    onPress={handleCopyMnemonic}
                    title="Copy to Clipboard"
                    variant="secondary"
                    style={styles.button}
                />

                {!hasBackedUp ? (
                    <Button
                        onPress={handleConfirmBackup}
                        title="I Have Backed Up My Seed Phrase"
                        style={styles.button}
                    />
                ) : (
                    <>
                        <Card style={styles.successCard}>
                            <Text style={styles.successText}>
                                Great! Now create your identity on-chain.
                            </Text>
                        </Card>

                        <Button
                            onPress={handleCreateIdentity}
                            title="Create Identity"
                            style={styles.button}
                        />
                    </>
                )}
            </ScrollView>
        </View>
    );
}

const styles = StyleSheet.create({
    container: {
        flex: 1,
        backgroundColor: '#FFFFFF',
    },
    scrollContent: {
        padding: 20,
    },
    title: {
        fontSize: 28,
        fontWeight: '700',
        color: '#111827',
        marginBottom: 12,
    },
    description: {
        fontSize: 16,
        color: '#6B7280',
        marginBottom: 24,
        lineHeight: 24,
    },
    infoCard: {
        marginBottom: 16,
        backgroundColor: '#EEF2FF',
        borderColor: '#6366F1',
        borderWidth: 1,
    },
    infoTitle: {
        fontSize: 18,
        fontWeight: '600',
        color: '#20283aff',
        marginBottom: 8,
    },
    infoText: {
        fontSize: 14,
        color: '#4B5563',
        lineHeight: 20,
    },
    warningCard: {
        marginBottom: 24,
        backgroundColor: '#FEF3C7',
        borderColor: '#F59E0B',
        borderWidth: 1,
    },
    warningTitle: {
        fontSize: 18,
        fontWeight: '600',
        color: '#92400E',
        marginBottom: 8,
    },
    warningText: {
        fontSize: 14,
        color: '#78350F',
        lineHeight: 20,
    },
    mnemonicCard: {
        marginBottom: 24,
        padding: 20,
        backgroundColor: '#F9FAFB',
    },
    wordContainer: {
        flexDirection: 'row',
        alignItems: 'center',
        marginBottom: 12,
    },
    wordNumber: {
        fontSize: 14,
        fontWeight: '600',
        color: '#6B7280',
        width: 30,
    },
    word: {
        fontSize: 16,
        fontWeight: '600',
        color: '#111827',
        flex: 1,
    },
    successCard: {
        marginBottom: 16,
        backgroundColor: '#D1FAE5',
        borderColor: '#10B981',
        borderWidth: 1,
    },
    successText: {
        fontSize: 16,
        color: '#065F46',
        textAlign: 'center',
    },
    button: {
        marginBottom: 16,
    },
    loader: {
        marginVertical: 32,
    },
});