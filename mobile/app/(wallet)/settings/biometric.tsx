import { useState, useEffect } from 'react';
import {
    View,
    Text,
    StyleSheet,
    ScrollView,
    Switch,
    Alert,
    ActivityIndicator,
    TouchableOpacity,
} from 'react-native';
import { router } from 'expo-router';
import { useAuthStore, useUIStore } from '../../../src/store';
import { biometricAuth } from '../../../src/services/storage/biometric';
import { Card } from '../../../src/components/common/Card';
import { Button } from '../../../src/components/common/Button';

export default function BiometricSettingsScreen() {
    const [isLoading, setIsLoading] = useState(true);
    const [biometricEnabled, setBiometricEnabled] = useState(false);
    const [isAvailable, setIsAvailable] = useState(false);
    const [isEnrolled, setIsEnrolled] = useState(false);
    const [supportedTypes, setSupportedTypes] = useState<string[]>([]);
    const [securityLevel, setSecurityLevel] = useState<string>('none');
    const [hasFallbackPin, setHasFallbackPin] = useState(false);
    const [isToggling, setIsToggling] = useState(false);

    const authStore = useAuthStore();
    const showToast = useUIStore(state => state.showToast);

    useEffect(() => {
        loadBiometricStatus();
    }, []);

    const loadBiometricStatus = async () => {
        try {
            setIsLoading(true);

            // Check capabilities
            const capabilities = await biometricAuth.checkCapabilities();
            setIsAvailable(capabilities.isAvailable);
            setIsEnrolled(capabilities.isEnrolled);
            setSupportedTypes(capabilities.supportedTypes);
            setSecurityLevel(capabilities.securityLevel);

            // Check if enabled
            const enabled = await biometricAuth.isBiometricEnabled();
            setBiometricEnabled(enabled);

            // Check if PIN exists
            const hasPin = await biometricAuth.hasFallbackPin();
            setHasFallbackPin(hasPin);
        } catch (error) {
            console.error('Load biometric status failed:', error);
            showToast('Failed to load biometric settings', 'error');
        } finally {
            setIsLoading(false);
        }
    };

    const handleToggleBiometric = async (value: boolean) => {
        if (value) {
            // Enable biometric
            if (!isEnrolled) {
                Alert.alert(
                    'No Biometrics Enrolled',
                    'Please set up Face ID or Touch ID in your device settings first.',
                    [{ text: 'OK' }]
                );
                return;
            }

            if (!hasFallbackPin) {
                Alert.alert(
                    'Set Up PIN First',
                    'Please set up a fallback PIN before enabling biometric authentication.',
                    [
                        { text: 'Cancel', style: 'cancel' },
                        {
                            text: 'Set Up PIN',
                            onPress: () => router.push('/(wallet)/settings/backup'),
                        },
                    ]
                );
                return;
            }

            await enableBiometric();
        } else {
            // Disable biometric
            Alert.alert(
                'Disable Biometric',
                'Are you sure you want to disable biometric authentication? You will need to use your PIN to access the app.',
                [
                    { text: 'Cancel', style: 'cancel' },
                    {
                        text: 'Disable',
                        style: 'destructive',
                        onPress: disableBiometric,
                    },
                ]
            );
        }
    };

    const enableBiometric = async () => {
        setIsToggling(true);

        try {
            const result = await biometricAuth.enableBiometric();

            if (result.success) {
                setBiometricEnabled(true);
                authStore.enableBiometric();
                showToast(`${result.biometricType} enabled successfully!`, 'success');
            } else {
                showToast(result.error || 'Failed to enable biometric', 'error');
            }
        } catch (error) {
            console.error('Enable biometric failed:', error);
            showToast('Failed to enable biometric authentication', 'error');
        } finally {
            setIsToggling(false);
        }
    };

    const disableBiometric = async () => {
        setIsToggling(true);

        try {
            const success = await biometricAuth.disableBiometric();

            if (success) {
                setBiometricEnabled(false);
                authStore.disableBiometric();
                showToast('Biometric authentication disabled', 'success');
            } else {
                showToast('Failed to disable biometric', 'error');
            }
        } catch (error) {
            console.error('Disable biometric failed:', error);
            showToast('Failed to disable biometric authentication', 'error');
        } finally {
            setIsToggling(false);
        }
    };

    const handleTestBiometric = async () => {
        try {
            const result = await biometricAuth.testBiometric();

            if (result.success) {
                Alert.alert(
                    'Test Successful',
                    `${result.biometricType} authentication is working correctly.`,
                    [{ text: 'OK' }]
                );
            } else {
                Alert.alert(
                    'Test Failed',
                    result.error || 'Biometric authentication test failed.',
                    [{ text: 'OK' }]
                );
            }
        } catch (error) {
            console.error('Test biometric failed:', error);
            Alert.alert('Test Failed', 'An error occurred during the test.', [{ text: 'OK' }]);
        }
    };

    const getSecurityLevelDisplay = (): { text: string; color: string; icon: string } => {
        switch (securityLevel) {
            case 'strong':
                return { text: 'Strong', color: '#10B981', icon: '🔒' };
            case 'weak':
                return { text: 'Weak', color: '#F59E0B', icon: '⚠️' };
            default:
                return { text: 'None', color: '#6B7280', icon: '❌' };
        }
    };

    if (isLoading) {
        return (
            <View style={styles.loadingContainer}>
                <ActivityIndicator size="large" color="#6366F1" />
                <Text style={styles.loadingText}>Loading biometric settings...</Text>
            </View>
        );
    }

    const securityDisplay = getSecurityLevelDisplay();

    return (
        <View style={styles.container}>
            <ScrollView contentContainerStyle={styles.scrollContent}>
                <Text style={styles.title}>Biometric Authentication</Text>
                <Text style={styles.subtitle}>
                    Secure your wallet with {supportedTypes[0] || 'biometric'} authentication
                </Text>

                {/* Main Toggle */}
                <Card style={styles.card}>
                    <View style={styles.toggleRow}>
                        <View style={styles.toggleLeft}>
                            <Text style={styles.toggleTitle}>
                                {supportedTypes[0] || 'Biometric Authentication'}
                            </Text>
                            <Text style={styles.toggleDescription}>
                                {biometricEnabled
                                    ? 'Quick and secure access to your wallet'
                                    : 'Enable for quick access'}
                            </Text>
                        </View>
                        <Switch
                            value={biometricEnabled}
                            onValueChange={handleToggleBiometric}
                            disabled={!isAvailable || isToggling}
                            trackColor={{ false: '#D1D5DB', true: '#6366F1' }}
                            thumbColor="#FFFFFF"
                        />
                    </View>
                </Card>

                {/* Status Information */}
                {!isAvailable && (
                    <Card style={styles.warningCard}>
                        <Text style={styles.warningTitle}>⚠️ Not Available</Text>
                        <Text style={styles.warningText}>
                            Biometric authentication is not available on this device.
                        </Text>
                    </Card>
                )}

                {isAvailable && !isEnrolled && (
                    <Card style={styles.warningCard}>
                        <Text style={styles.warningTitle}>⚠️ Not Set Up</Text>
                        <Text style={styles.warningText}>
                            No biometrics are enrolled. Please set up 
                            {supportedTypes[0] || 'biometricauthentication'} in your device settings.
                        </Text>
                    </Card>
                )}

                {/* Device Information */}
                {isAvailable && (
                    <View style={styles.section}>
                        <Text style={styles.sectionTitle}>Device Information</Text>

                        <Card style={styles.infoCard}>
                            <View style={styles.infoRow}>
                                <Text style={styles.infoLabel}>Supported Types</Text>
                                <Text style={styles.infoValue}>
                                    {supportedTypes.join(', ') || 'None'}
                                </Text>
                            </View>

                            <View style={styles.divider} />

                            <View style={styles.infoRow}>
                                <Text style={styles.infoLabel}>Security Level</Text>
                                <View style={styles.securityLevel}>
                                    <Text style={styles.securityIcon}>{securityDisplay.icon}</Text>
                                    <Text
                                        style={[
                                            styles.infoValue,
                                            { color: securityDisplay.color },
                                        ]}
                                    >
                                        {securityDisplay.text}
                                    </Text>
                                </View>
                            </View>

                            <View style={styles.divider} />

                            <View style={styles.infoRow}>
                                <Text style={styles.infoLabel}>Enrollment Status</Text>
                                <Text
                                    style={[
                                        styles.infoValue,
                                        { color: isEnrolled ? '#10B981' : '#EF4444' },
                                    ]}
                                >
                                    {isEnrolled ? 'Enrolled' : 'Not Enrolled'}
                                </Text>
                            </View>

                            <View style={styles.divider} />

                            <View style={styles.infoRow}>
                                <Text style={styles.infoLabel}>Fallback PIN</Text>
                                <Text
                                    style={[
                                        styles.infoValue,
                                        { color: hasFallbackPin ? '#10B981' : '#EF4444' },
                                    ]}
                                >
                                    {hasFallbackPin ? 'Set' : 'Not Set'}
                                </Text>
                            </View>
                        </Card>
                    </View>
                )}

                {/* Actions */}
                {isAvailable && isEnrolled && (
                    <View style={styles.section}>
                        <Text style={styles.sectionTitle}>Actions</Text>

                        <Button
                            onPress={handleTestBiometric}
                            title="Test Biometric Authentication"
                            variant="secondary"
                            style={styles.actionButton}
                        />

                        {!hasFallbackPin && (
                            <Button
                                onPress={() => router.push('/(wallet)/settings/backup')}
                                title="Set Up Fallback PIN"
                                style={styles.actionButton}
                            />
                        )}
                    </View>
                )}

                {/* Security Info */}
                <Card style={styles.infoBoxCard}>
                    <Text style={styles.infoBoxTitle}>🛡️ Security Information</Text>
                    <Text style={styles.infoBoxText}>
                        • Biometric data never leaves your device
                    </Text>
                    <Text style={styles.infoBoxText}>
                        • Your biometric information is stored securely in the device's secure
                        enclave
                    </Text>
                    <Text style={styles.infoBoxText}>
                        • A fallback PIN is required in case biometric authentication fails
                    </Text>
                    <Text style={styles.infoBoxText}>
                        • After 5 failed attempts, you'll be locked out for 5 minutes
                    </Text>
                </Card>

                {/* Best Practices */}
                <Card style={styles.tipsCard}>
                    <Text style={styles.tipsTitle}>💡 Best Practices</Text>
                    <Text style={styles.tipsText}>
                        1. Always set up a strong fallback PIN
                    </Text>
                    <Text style={styles.tipsText}>
                        2. Keep your device's biometric sensors clean
                    </Text>
                    <Text style={styles.tipsText}>
                        3. Re-enroll biometrics if authentication frequently fails
                    </Text>
                    <Text style={styles.tipsText}>
                        4. Never share your device with others
                    </Text>
                </Card>
            </ScrollView>
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
        marginTop: 24,
    },
    sectionTitle: {
        fontSize: 18,
        fontWeight: '600',
        color: '#111827',
        marginBottom: 12,
    },
    card: {
        marginBottom: 16,
    },
    toggleRow: {
        flexDirection: 'row',
        justifyContent: 'space-between',
        alignItems: 'center',
    },
    toggleLeft: {
        flex: 1,
        marginRight: 16,
    },
    toggleTitle: {
        fontSize: 16,
        fontWeight: '600',
        color: '#111827',
        marginBottom: 4,
    },
    toggleDescription: {
        fontSize: 14,
        color: '#6B7280',
        lineHeight: 20,
    },
    warningCard: {
        marginBottom: 16,
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
    infoCard: {
        padding: 16,
    },
    infoRow: {
        flexDirection: 'row',
        justifyContent: 'space-between',
        alignItems: 'center',
        paddingVertical: 12,
    },
    infoLabel: {
        fontSize: 14,
        color: '#6B7280',
    },
    infoValue: {
        fontSize: 14,
        fontWeight: '600',
        color: '#111827',
    },
    securityLevel: {
        flexDirection: 'row',
        alignItems: 'center',
        gap: 6,
    },
    securityIcon: {
        fontSize: 14,
    },
    divider: {
        height: 1,
        backgroundColor: '#E5E7EB',
    },
    actionButton: {
        marginBottom: 12,
    },
    infoBoxCard: {
        marginTop: 24,
        backgroundColor: '#EEF2FF',
        borderColor: '#6366F1',
        borderWidth: 1,
    },
    infoBoxTitle: {
        fontSize: 16,
        fontWeight: '600',
        color: '#3730A3',
        marginBottom: 12,
    },
    infoBoxText: {
        fontSize: 14,
        color: '#4338CA',
        marginBottom: 8,
        lineHeight: 20,
    },
    tipsCard: {
        marginTop: 16,
        marginBottom: 32,
        backgroundColor: '#ECFDF5',
        borderColor: '#10B981',
        borderWidth: 1,
    },
    tipsTitle: {
        fontSize: 16,
        fontWeight: '600',
        color: '#065F46',
        marginBottom: 12,
    },
    tipsText: {
        fontSize: 14,
        color: '#047857',
        marginBottom: 8,
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