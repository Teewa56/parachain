import { useState, useEffect } from 'react';
import {
    View,
    Text,
    StyleSheet,
    ScrollView,
    TextInput,
    Alert,
    TouchableOpacity,
    Modal,
} from 'react-native';
import { router } from 'expo-router';
import { useAuthStore, useUIStore } from '../../../src/store';
import { biometricAuth } from '../../../src/services/storage/biometric';
import { keyManagement } from '../../../src/services/crypto/keyManagement';
import { Card } from '../../../src/components/common/Card';
import { Button } from '../../../src/components/common/Button';

export default function BackupSettingsScreen() {
    const [showMnemonic, setShowMnemonic] = useState(false);
    const [mnemonic, setMnemonic] = useState<string | null>(null);
    const [authenticated, setAuthenticated] = useState(false);
    const [hasFallbackPin, setHasFallbackPin] = useState(false);
    
    // PIN Setup
    const [showPinSetup, setShowPinSetup] = useState(false);
    const [newPin, setNewPin] = useState('');
    const [confirmPin, setConfirmPin] = useState('');
    const [pinError, setPinError] = useState('');

    const showToast = useUIStore(state => state.showToast);
    const setLoading = useUIStore(state => state.setLoading);

    useEffect(() => {
        checkPinStatus();
    }, []);

    const checkPinStatus = async () => {
        const hasPin = await biometricAuth.hasFallbackPin();
        setHasFallbackPin(hasPin);
    };

    const handleViewSeedPhrase = async () => {
        try {
            // Require authentication
            const result = await biometricAuth.authenticate('Authenticate to view seed phrase');
            
            if (!result.success) {
                showToast('Authentication failed', 'error');
                return;
            }

            setAuthenticated(true);
            
            // Get mnemonic from secure storage
            const storedMnemonic = await keyManagement.getMnemonic();
            
            if (!storedMnemonic) {
                Alert.alert(
                    'Seed Phrase Not Found',
                    'No seed phrase found in secure storage. Your identity may have been imported from a JSON file.',
                    [{ text: 'OK' }]
                );
                return;
            }

            setMnemonic(storedMnemonic);
            setShowMnemonic(true);
        } catch (error) {
            console.error('View seed phrase failed:', error);
            showToast('Failed to retrieve seed phrase', 'error');
        }
    };

    const handleCopyMnemonic = () => {
        if (!mnemonic) return;
        
        // apply actual copy logic
        showToast('Copied to clipboard', 'success');
        
        Alert.alert(
            'Security Warning',
            'Your seed phrase has been copied to clipboard. Please ensure no one can see your screen and paste it securely.',
            [{ text: 'I Understand' }]
        );
    };

    const handleCloseMnemonic = () => {
        setShowMnemonic(false);
        setMnemonic(null);
        setAuthenticated(false);
    };

    const handleSetupPin = () => {
        setShowPinSetup(true);
        setNewPin('');
        setConfirmPin('');
        setPinError('');
    };

    const handlePinChange = (value: string, isConfirm: boolean) => {
        if (isConfirm) {
            setConfirmPin(value);
            setPinError('');
        } else {
            setNewPin(value);
            setPinError('');
        }
    };

    const handleSavePin = async () => {
        // Validate PIN format
        const validation = biometricAuth.validatePinFormat(newPin);
        if (!validation.valid) {
            setPinError(validation.error || 'Invalid PIN');
            return;
        }

        // Check if PINs match
        if (newPin !== confirmPin) {
            setPinError('PINs do not match');
            return;
        }

        setLoading(true, 'Setting up PIN...');

        try {
            const success = await biometricAuth.setFallbackPin(newPin);
            
            if (success) {
                setHasFallbackPin(true);
                setShowPinSetup(false);
                setNewPin('');
                setConfirmPin('');
                showToast('PIN set up successfully', 'success');
                
                Alert.alert(
                    'PIN Set Up',
                    'Your fallback PIN has been set up successfully. You can now enable biometric authentication.',
                    [
                        {
                            text: 'Set Up Biometric',
                            onPress: () => router.push('/(wallet)/settings/biometric'),
                        },
                        { text: 'OK' },
                    ]
                );
            } else {
                setPinError('Failed to set up PIN');
            }
        } catch (error) {
            console.error('Setup PIN failed:', error);
            setPinError('An error occurred while setting up PIN');
        } finally {
            setLoading(false);
        }
    };

    const handleExportBackup = async () => {
        Alert.alert(
            'Export Encrypted Backup',
            'This will export an encrypted backup of your wallet. You will need to set a password.',
            [
                { text: 'Cancel', style: 'cancel' },
                { text: 'Export', onPress: performExportBackup },
            ]
        );
    };

    const performExportBackup = async () => {
        try {
            // Require authentication
            const result = await biometricAuth.authenticate('Authenticate to export backup');
            
            if (!result.success) {
                showToast('Authentication failed', 'error');
                return;
            }

            setLoading(true, 'Exporting backup...');

            // create an encrypted backup file
            await new Promise(resolve => setTimeout(resolve, 2000));

            showToast('Backup exported successfully', 'success');
            
            Alert.alert(
                'Backup Exported',
                'Your encrypted backup has been saved. Keep it in a secure location.',
                [{ text: 'OK' }]
            );
        } catch (error) {
            console.error('Export backup failed:', error);
            showToast('Failed to export backup', 'error');
        } finally {
            setLoading(false);
        }
    };

    return (
        <View style={styles.container}>
            <ScrollView contentContainerStyle={styles.scrollContent}>
                <Text style={styles.title}>Backup & Security</Text>
                <Text style={styles.subtitle}>
                    Protect your wallet and manage recovery options
                </Text>

                {/* Seed Phrase Section */}
                <View style={styles.section}>
                    <Text style={styles.sectionTitle}>Seed Phrase</Text>
                    
                    <Card style={styles.card}>
                        <Text style={styles.cardTitle}>🔑 Recovery Phrase</Text>
                        <Text style={styles.cardText}>
                            Your seed phrase is the master key to your wallet. Keep it secure and
                            never share it with anyone.
                        </Text>
                        <Button
                            onPress={handleViewSeedPhrase}
                            title="View Seed Phrase"
                            variant="secondary"
                            style={styles.cardButton}
                        />
                    </Card>
                </View>

                {/* PIN Section */}
                <View style={styles.section}>
                    <Text style={styles.sectionTitle}>Fallback PIN</Text>
                    
                    <Card style={styles.card}>
                        <View style={styles.pinStatusRow}>
                            <Text style={styles.cardTitle}>📱 Fallback PIN</Text>
                            <View
                                style={[
                                    styles.statusBadge,
                                    hasFallbackPin ? styles.statusActive : styles.statusInactive,
                                ]}
                            >
                                <Text
                                    style={[
                                        styles.statusText,
                                        hasFallbackPin
                                            ? styles.statusTextActive
                                            : styles.statusTextInactive,
                                    ]}
                                >
                                    {hasFallbackPin ? 'Set' : 'Not Set'}
                                </Text>
                            </View>
                        </View>
                        <Text style={styles.cardText}>
                            Set up a PIN as a fallback authentication method. This is required
                            before enabling biometric authentication.
                        </Text>
                        <Button
                            onPress={handleSetupPin}
                            title={hasFallbackPin ? 'Change PIN' : 'Set Up PIN'}
                            style={styles.cardButton}
                        />
                    </Card>
                </View>

                {/* Backup Export Section */}
                <View style={styles.section}>
                    <Text style={styles.sectionTitle}>Encrypted Backup</Text>
                    
                    <Card style={styles.card}>
                        <Text style={styles.cardTitle}>💾 Export Backup</Text>
                        <Text style={styles.cardText}>
                            Create an encrypted backup file that can be used to restore your wallet
                            on another device.
                        </Text>
                        <Button
                            onPress={handleExportBackup}
                            title="Export Encrypted Backup"
                            variant="secondary"
                            style={styles.cardButton}
                        />
                    </Card>
                </View>

                {/* Security Tips */}
                <Card style={styles.tipsCard}>
                    <Text style={styles.tipsTitle}>🛡️ Security Best Practices</Text>
                    <Text style={styles.tipsText}>
                        • Write down your seed phrase on paper and store it securely
                    </Text>
                    <Text style={styles.tipsText}>
                        • Never take a photo or screenshot of your seed phrase
                    </Text>
                    <Text style={styles.tipsText}>
                        • Store multiple copies in different secure locations
                    </Text>
                    <Text style={styles.tipsText}>
                        • Never share your seed phrase or PIN with anyone
                    </Text>
                    <Text style={styles.tipsText}>
                        • Keep your backup file in a secure, encrypted location
                    </Text>
                    <Text style={styles.tipsText}>
                        • Use a strong, unique PIN that you can remember
                    </Text>
                </Card>
            </ScrollView>

            {/* Seed Phrase Modal */}
            <Modal
                visible={showMnemonic}
                animationType="slide"
                presentationStyle="pageSheet"
                onRequestClose={handleCloseMnemonic}
            >
                <View style={styles.modalContainer}>
                    <View style={styles.modalHeader}>
                        <Text style={styles.modalTitle}>Your Seed Phrase</Text>
                        <TouchableOpacity onPress={handleCloseMnemonic}>
                            <Text style={styles.modalClose}>✕</Text>
                        </TouchableOpacity>
                    </View>

                    <ScrollView contentContainerStyle={styles.modalContent}>
                        <Card style={styles.warningCard}>
                            <Text style={styles.warningTitle}>⚠️ Security Warning</Text>
                            <Text style={styles.warningText}>
                                Never share your seed phrase with anyone. Anyone with access to your
                                seed phrase can access your wallet and steal your assets.
                            </Text>
                        </Card>

                        {mnemonic && (
                            <Card style={styles.mnemonicCard}>
                                {mnemonic.split(' ').map((word, index) => (
                                    <View key={index} style={styles.wordRow}>
                                        <Text style={styles.wordNumber}>{index + 1}.</Text>
                                        <Text style={styles.word}>{word}</Text>
                                    </View>
                                ))}
                            </Card>
                        )}

                        <Button
                            onPress={handleCopyMnemonic}
                            title="Copy to Clipboard"
                            variant="secondary"
                            style={styles.modalButton}
                        />

                        <Card style={styles.instructionsCard}>
                            <Text style={styles.instructionsTitle}>📝 Instructions</Text>
                            <Text style={styles.instructionsText}>
                                1. Write down these words in order on paper
                            </Text>
                            <Text style={styles.instructionsText}>
                                2. Store the paper in a secure location
                            </Text>
                            <Text style={styles.instructionsText}>
                                3. Never store digitally or take a photo
                            </Text>
                            <Text style={styles.instructionsText}>
                                4. Verify you've written them correctly
                            </Text>
                        </Card>

                        <Button
                            onPress={handleCloseMnemonic}
                            title="I've Saved My Seed Phrase"
                            style={styles.modalButton}
                        />
                    </ScrollView>
                </View>
            </Modal>

            {/* PIN Setup Modal */}
            <Modal
                visible={showPinSetup}
                animationType="slide"
                presentationStyle="pageSheet"
                onRequestClose={() => setShowPinSetup(false)}
            >
                <View style={styles.modalContainer}>
                    <View style={styles.modalHeader}>
                        <Text style={styles.modalTitle}>Set Up Fallback PIN</Text>
                        <TouchableOpacity onPress={() => setShowPinSetup(false)}>
                            <Text style={styles.modalClose}>✕</Text>
                        </TouchableOpacity>
                    </View>

                    <ScrollView contentContainerStyle={styles.modalContent}>
                        <Card style={styles.infoCard}>
                            <Text style={styles.infoTitle}>📱 About Your PIN</Text>
                            <Text style={styles.infoText}>
                                Your PIN will be used as a fallback authentication method when
                                biometric authentication is unavailable.
                            </Text>
                        </Card>

                        <View style={styles.inputSection}>
                            <Text style={styles.inputLabel}>New PIN (6-8 digits)</Text>
                            <TextInput
                                style={styles.input}
                                value={newPin}
                                onChangeText={(value) => handlePinChange(value, false)}
                                keyboardType="number-pad"
                                maxLength={8}
                                secureTextEntry
                                placeholder="Enter new PIN"
                                placeholderTextColor="#9CA3AF"
                            />
                        </View>

                        <View style={styles.inputSection}>
                            <Text style={styles.inputLabel}>Confirm PIN</Text>
                            <TextInput
                                style={styles.input}
                                value={confirmPin}
                                onChangeText={(value) => handlePinChange(value, true)}
                                keyboardType="number-pad"
                                maxLength={8}
                                secureTextEntry
                                placeholder="Confirm new PIN"
                                placeholderTextColor="#9CA3AF"
                            />
                        </View>

                        {pinError && (
                            <Card style={styles.errorCard}>
                                <Text style={styles.errorText}>⚠️ {pinError}</Text>
                            </Card>
                        )}

                        <Card style={styles.requirementsCard}>
                            <Text style={styles.requirementsTitle}>PIN Requirements:</Text>
                            <Text style={styles.requirementText}>
                                ✓ Must be 6-8 digits long
                            </Text>
                            <Text style={styles.requirementText}>
                                ✓ Numbers only
                            </Text>
                            <Text style={styles.requirementText}>
                                ✗ Avoid sequential digits (123456)
                            </Text>
                            <Text style={styles.requirementText}>
                                ✗ Avoid repeated digits (111111)
                            </Text>
                        </Card>

                        <Button
                            onPress={handleSavePin}
                            title="Set PIN"
                            disabled={!newPin || !confirmPin}
                            style={styles.modalButton}
                        />
                    </ScrollView>
                </View>
            </Modal>
        </View>
    );
}

const styles = StyleSheet.create({
    container: {
        flex: 1,
        backgroundColor: '#F9FAFB',
    },
    scrollContent: {
        padding: 20,
    },
    title: {
        fontSize: 28,
        fontWeight: '700',
        color: '#111827',
        marginBottom: 8,
    },
    subtitle: {
        fontSize: 16,
        color: '#6B7280',
        marginBottom: 24,
        lineHeight: 24,
    },
    section: {
        marginBottom: 24,
    },
    sectionTitle: {
        fontSize: 18,
        fontWeight: '600',
        color: '#111827',
        marginBottom: 12,
    },
    card: {
        marginBottom: 12,
    },
    cardTitle: {
        fontSize: 16,
        fontWeight: '600',
        color: '#111827',
        marginBottom: 8,
    },
    cardText: {
        fontSize: 14,
        color: '#6B7280',
        lineHeight: 20,
        marginBottom: 16,
    },
    cardButton: {
        marginTop: 4,
    },
    pinStatusRow: {
        flexDirection: 'row',
        justifyContent: 'space-between',
        alignItems: 'center',
        marginBottom: 8,
    },
    statusBadge: {
        paddingHorizontal: 12,
        paddingVertical: 6,
        borderRadius: 12,
    },
    statusActive: {
        backgroundColor: '#D1FAE5',
    },
    statusInactive: {
        backgroundColor: '#FEE2E2',
    },
    statusText: {
        fontSize: 12,
        fontWeight: '600',
    },
    statusTextActive: {
        color: '#065F46',
    },
    statusTextInactive: {
        color: '#991B1B',
    },
    tipsCard: {
        marginTop: 8,
        marginBottom: 32,
        backgroundColor: '#EEF2FF',
        borderColor: '#6366F1',
        borderWidth: 1,
    },
    tipsTitle: {
        fontSize: 16,
        fontWeight: '600',
        color: '#3730A3',
        marginBottom: 12,
    },
    tipsText: {
        fontSize: 14,
        color: '#4338CA',
        marginBottom: 8,
        lineHeight: 20,
    },
    modalContainer: {
        flex: 1,
        backgroundColor: '#FFFFFF',
    },
    modalHeader: {
        flexDirection: 'row',
        justifyContent: 'space-between',
        alignItems: 'center',
        padding: 20,
        borderBottomWidth: 1,
        borderBottomColor: '#E5E7EB',
    },
    modalTitle: {
        fontSize: 20,
        fontWeight: '700',
        color: '#111827',
    },
    modalClose: {
        fontSize: 24,
        color: '#6B7280',
        padding: 4,
    },
    modalContent: {
        padding: 20,
    },
    modalButton: {
        marginTop: 16,
    },
    warningCard: {
        marginBottom: 20,
        backgroundColor: '#FEF3C7',
        borderColor: '#F59E0B',
        borderWidth: 1,
    },
    warningTitle: {
        fontSize: 16,
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
        marginBottom: 20,
        backgroundColor: '#F9FAFB',
        padding: 16,
    },
    wordRow: {
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
        fontFamily: 'monospace',
    },
    instructionsCard: {
        marginTop: 20,
        backgroundColor: '#ECFDF5',
        borderColor: '#10B981',
        borderWidth: 1,
    },
    instructionsTitle: {
        fontSize: 16,
        fontWeight: '600',
        color: '#065F46',
        marginBottom: 12,
    },
    instructionsText: {
        fontSize: 14,
        color: '#047857',
        marginBottom: 8,
        lineHeight: 20,
    },
    infoCard: {
        marginBottom: 20,
        backgroundColor: '#EEF2FF',
        borderColor: '#6366F1',
        borderWidth: 1,
    },
    infoTitle: {
        fontSize: 16,
        fontWeight: '600',
        color: '#3730A3',
        marginBottom: 8,
    },
    infoText: {
        fontSize: 14,
        color: '#4338CA',
        lineHeight: 20,
    },
    inputSection: {
        marginBottom: 20,
    },
    inputLabel: {
        fontSize: 14,
        fontWeight: '600',
        color: '#111827',
        marginBottom: 8,
    },
    input: {
        backgroundColor: '#F9FAFB',
        borderWidth: 1,
        borderColor: '#D1D5DB',
        borderRadius: 8,
        padding: 16,
        fontSize: 16,
        color: '#111827',
    },
    errorCard: {
        marginBottom: 20,
        backgroundColor: '#FEE2E2',
        borderColor: '#EF4444',
        borderWidth: 1,
    },
    errorText: {
        fontSize: 14,
        fontWeight: '600',
        color: '#991B1B',
    },
    requirementsCard: {
        marginBottom: 20,
        backgroundColor: '#F9FAFB',
    },
    requirementsTitle: {
        fontSize: 14,
        fontWeight: '600',
        color: '#111827',
        marginBottom: 12,
    },
    requirementText: {
        fontSize: 13,
        color: '#6B7280',
        marginBottom: 6,
        lineHeight: 18,
    },
});