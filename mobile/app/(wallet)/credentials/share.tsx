import { useState } from 'react';
import { View, Text, StyleSheet, ScrollView, Alert } from 'react-native';
import { useLocalSearchParams, router } from 'expo-router';
import { useCredentialStore, useUIStore } from '../../../src/store';
import { Card } from '../../../src/components/common/Card';
import { Button } from '../../../src/components/common/Button';
import { FieldSelector } from '../../../src/components/credential/FieldSelector';

export default function ShareCredentialScreen() {
    const params = useLocalSearchParams();
    const credentialId = params.credentialId as string;

    const getCredentialById = useCredentialStore(state => state.getCredentialById);
    const showToast = useUIStore(state => state.showToast);

    const [selectedFields, setSelectedFields] = useState<{ [key: number]: boolean }>({});
    const [shareMethod, setShareMethod] = useState<'qr' | 'link' | null>(null);

    const credential = getCredentialById(credentialId);

    if (!credential) {
        return (
            <View style={styles.container}>
                <View style={styles.errorContainer}>
                    <Text style={styles.errorIcon}>⚠️</Text>
                    <Text style={styles.errorTitle}>Credential Not Found</Text>
                    <Button onPress={() => router.back()} title="Go Back" variant="secondary" />
                </View>
            </View>
        );
    }

    const handleFieldToggle = (fieldIndex: number) => {
        setSelectedFields(prev => ({
            ...prev,
            [fieldIndex]: !prev[fieldIndex],
        }));
    };

    const getSelectedCount = (): number => {
        return Object.values(selectedFields).filter(Boolean).length;
    };

    const handleShare = () => {
        const selectedCount = getSelectedCount();

        if (selectedCount === 0) {
            showToast('Please select at least one field to share', 'warning');
            return;
        }

        if (!shareMethod) {
            showToast('Please select a sharing method', 'warning');
            return;
        }

        if (shareMethod === 'qr') {
            router.push({
                pathname: '/(wallet)/credentials/qr',
                params: { credentialId: credential.subject },
            });
        } else {
            showToast('Link sharing coming soon', 'info');
        }
    };

    return (
        <View style={styles.container}>
            <ScrollView contentContainerStyle={styles.scrollContent}>
                <Text style={styles.title}>Share Credential</Text>
                <Text style={styles.subtitle}>
                    Select which fields to share and how to share them
                </Text>

                {/* Credential Info */}
                <Card style={styles.credentialCard}>
                    <Text style={styles.credentialType}>{credential.credentialType}</Text>
                    <Text style={styles.credentialStatus}>Status: {credential.status}</Text>
                </Card>

                {/* Field Selection */}
                <View style={styles.section}>
                    <Text style={styles.sectionTitle}>Select Fields to Share</Text>
                    <Card style={styles.fieldCard}>
                        <FieldSelector
                            credential={credential}
                            selectedFields={selectedFields}
                            onFieldToggle={handleFieldToggle}
                        />
                    </Card>
                </View>

                {/* Share Method */}
                <View style={styles.section}>
                    <Text style={styles.sectionTitle}>Sharing Method</Text>
                    
                    <Card
                        style={[
                            styles.methodCard,
                            shareMethod === 'qr' && styles.methodCardSelected,
                        ]}
                    >
                        <Button
                            onPress={() => setShareMethod('qr')}
                            title="📱 QR Code"
                            variant={shareMethod === 'qr' ? 'primary' : 'secondary'}
                        />
                        <Text style={styles.methodDescription}>
                            Generate a temporary QR code for in-person verification
                        </Text>
                    </Card>

                    <Card
                        style={[
                            styles.methodCard,
                            shareMethod === 'link' && styles.methodCardSelected,
                        ]}
                    >
                        <Button
                            onPress={() => setShareMethod('link')}
                            title="🔗 Secure Link"
                            variant={shareMethod === 'link' ? 'primary' : 'secondary'}
                            disabled
                        />
                        <Text style={styles.methodDescription}>
                            Create an encrypted link for remote verification (Coming soon)
                        </Text>
                    </Card>
                </View>

                {/* Privacy Notice */}
                <Card style={styles.privacyCard}>
                    <Text style={styles.privacyTitle}>🔒 Privacy Notice</Text>
                    <Text style={styles.privacyText}>
                        • Only {getSelectedCount()} selected field(s) will be visible{'\n'}
                        • All other information remains encrypted{'\n'}
                        • Verifier cannot access unselected fields{'\n'}
                        • You can revoke access at any time
                    </Text>
                </Card>

                <Button
                    onPress={handleShare}
                    title="Share Credential"
                    disabled={getSelectedCount() === 0 || !shareMethod}
                    style={styles.shareButton}
                />
            </ScrollView>
        </View>
    );
}

const styles = StyleSheet.create({
    container: { flex: 1, backgroundColor: '#F9FAFB' },
    scrollContent: { padding: 20 },
    title: { fontSize: 28, fontWeight: '700', color: '#111827', marginBottom: 8 },
    subtitle: { fontSize: 16, color: '#6B7280', marginBottom: 24, lineHeight: 24 },
    credentialCard: { marginBottom: 24 },
    credentialType: { fontSize: 20, fontWeight: '600', color: '#111827', marginBottom: 4 },
    credentialStatus: { fontSize: 14, color: '#6B7280' },
    section: { marginBottom: 24 },
    sectionTitle: { fontSize: 18, fontWeight: '600', color: '#111827', marginBottom: 12 },
    fieldCard: { padding: 16 },
    methodCard: { marginBottom: 12, borderWidth: 2, borderColor: 'transparent' },
    methodCardSelected: { borderColor: '#6366F1' },
    methodDescription: { fontSize: 13, color: '#6B7280', marginTop: 8, textAlign: 'center' },
    privacyCard: { marginBottom: 24, backgroundColor: '#EEF2FF', borderColor: '#6366F1', borderWidth: 1 },
    privacyTitle: { fontSize: 16, fontWeight: '600', color: '#3730A3', marginBottom: 12 },
    privacyText: { fontSize: 14, color: '#4338CA', lineHeight: 20 },
    shareButton: {},
    errorContainer: { flex: 1, alignItems: 'center', justifyContent: 'center', padding: 40 },
    errorIcon: { fontSize: 64, marginBottom: 16 },
    errorTitle: { fontSize: 20, fontWeight: '600', color: '#111827', marginBottom: 24 },
});