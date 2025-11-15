import { useEffect, useState } from 'react';
import { View, Text, StyleSheet, Alert } from 'react-native';
import { useLocalSearchParams, router } from 'expo-router';
import { useCredentialStore, useUIStore } from '../../../src/store';
import { Card } from '../../../src/components/common/Card';
import { Button } from '../../../src/components/common/Button';
import QRCode from 'react-native-qrcode-svg';
import * as Brightness from 'expo-brightness';

export default function CredentialQRScreen() {
    const params = useLocalSearchParams();
    const credentialId = params.credentialId as string;
    const proofHash = params.proofHash as string;

    const getCredentialById = useCredentialStore(state => state.getCredentialById);
    const showToast = useUIStore(state => state.showToast);

    const [timeLeft, setTimeLeft] = useState(120); // 2 minutes
    const [originalBrightness, setOriginalBrightness] = useState<number | null>(null);

    useEffect(() => {
        increaseBrightness();
        startTimer();

        return () => {
            restoreBrightness();
        };
    }, []);

    const increaseBrightness = async () => {
        try {
            const current = await Brightness.getBrightnessAsync();
            setOriginalBrightness(current);
            await Brightness.setBrightnessAsync(1.0);
        } catch (error) {
            console.log('Failed to adjust brightness:', error);
        }
    };

    const restoreBrightness = async () => {
        if (originalBrightness !== null) {
            try {
                await Brightness.setBrightnessAsync(originalBrightness);
            } catch (error) {
                console.log('Failed to restore brightness:', error);
            }
        }
    };

    const startTimer = () => {
        const interval = setInterval(() => {
            setTimeLeft(prev => {
                if (prev <= 1) {
                    clearInterval(interval);
                    handleExpired();
                    return 0;
                }
                return prev - 1;
            });
        }, 1000);
    };

    const handleExpired = () => {
        Alert.alert(
            'QR Code Expired',
            'This QR code has expired for security reasons. Please generate a new one.',
            [
                {
                    text: 'OK',
                    onPress: () => router.back(),
                },
            ]
        );
    };

    const formatTime = (seconds: number): string => {
        const mins = Math.floor(seconds / 60);
        const secs = seconds % 60;
        return `${mins}:${secs.toString().padStart(2, '0')}`;
    };

    const credential = credentialId ? getCredentialById(credentialId) : null;

    const qrData = JSON.stringify({
        type: proofHash ? 'proof' : 'credential',
        credentialId: credentialId || '',
        proofHash: proofHash || '',
        timestamp: Date.now(),
    });

    return (
        <View style={styles.container}>
            <View style={styles.content}>
                <Card style={styles.card}>
                    <Text style={styles.title}>Scan QR Code</Text>
                    <Text style={styles.subtitle}>
                        {proofHash ? 'Zero-Knowledge Proof' : 'Credential Verification'}
                    </Text>

                    <View style={styles.qrContainer}>
                        <QRCode
                            value={qrData}
                            size={280}
                            backgroundColor="white"
                            color="black"
                        />
                    </View>

                    <View style={styles.timerContainer}>
                        <Text style={styles.timerLabel}>Expires in</Text>
                        <Text style={styles.timerValue}>{formatTime(timeLeft)}</Text>
                    </View>

                    {credential && (
                        <View style={styles.infoContainer}>
                            <Text style={styles.infoLabel}>Credential Type</Text>
                            <Text style={styles.infoValue}>{credential.credentialType}</Text>
                        </View>
                    )}
                </Card>

                <Card style={styles.warningCard}>
                    <Text style={styles.warningTitle}>🔒 Security Notice</Text>
                    <Text style={styles.warningText}>
                        • This QR code is temporary and will expire in 2 minutes{'\n'}
                        • Only share with trusted verifiers{'\n'}
                        • The verifier will see only the revealed fields{'\n'}
                        • All other information remains private
                    </Text>
                </Card>

                <Button
                    onPress={() => router.back()}
                    title="Done"
                    variant="secondary"
                    style={styles.button}
                />
            </View>
        </View>
    );
}

const styles = StyleSheet.create({
    container: { flex: 1, backgroundColor: '#F9FAFB' },
    content: { flex: 1, padding: 20, justifyContent: 'center' },
    card: { alignItems: 'center', marginBottom: 20 },
    title: { fontSize: 24, fontWeight: '700', color: '#111827', marginBottom: 8 },
    subtitle: { fontSize: 16, color: '#6B7280', marginBottom: 32, textAlign: 'center' },
    qrContainer: { padding: 20, backgroundColor: '#FFFFFF', borderRadius: 16, marginBottom: 24 },
    timerContainer: { alignItems: 'center', marginBottom: 16 },
    timerLabel: { fontSize: 14, color: '#6B7280', marginBottom: 4 },
    timerValue: { fontSize: 32, fontWeight: '700', color: '#6366F1' },
    infoContainer: { alignItems: 'center', paddingTop: 16, borderTopWidth: 1, borderTopColor: '#E5E7EB', width: '100%' },
    infoLabel: { fontSize: 12, color: '#6B7280', marginBottom: 4 },
    infoValue: { fontSize: 16, fontWeight: '600', color: '#111827' },
    warningCard: { backgroundColor: '#EEF2FF', borderColor: '#6366F1', borderWidth: 1, marginBottom: 20 },
    warningTitle: { fontSize: 16, fontWeight: '600', color: '#3730A3', marginBottom: 12 },
    warningText: { fontSize: 14, color: '#4338CA', lineHeight: 20 },
    button: {},
});