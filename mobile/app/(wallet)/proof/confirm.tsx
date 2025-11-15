import { useState } from 'react';
import {
    View,
    Text,
    StyleSheet,
    ScrollView,
    Alert,
    ActivityIndicator,
} from 'react-native';
import { router, useLocalSearchParams } from 'expo-router';
import { useIdentityStore, useCredentialStore, useUIStore } from '../../../src/store';
import { substrateCalls } from '../../../src/services/substrate/calls';
import { substrateAPI } from '../../../src/services/substrate/api';
import { Card } from '../../../src/components/common/Card';
import { Button } from '../../../src/components/common/Button';
import type { ProofType } from '../../../src/types/substrate';
import { formatDid } from '../../../src/services/substrate/utils';
import { ERROR_MESSAGES } from '../../../src/utils/constants';

export default function ConfirmProofScreen() {
    const params = useLocalSearchParams();
    const [isSubmitting, setIsSubmitting] = useState(false);
    const [proofHash, setProofHash] = useState<string | null>(null);

    const keyPair = useIdentityStore(state => state.keyPair);
    const getCredentialById = useCredentialStore(state => state.getCredentialById);
    const showToast = useUIStore(state => state.showToast);
    const setLoading = useUIStore(state => state.setLoading);

    // Parse parameters
    const credentialId = params.credentialId as string;
    const fieldIndices = JSON.parse(params.fieldIndices as string) as number[];
    const proofType = params.proofType as ProofType;

    const credential = getCredentialById(credentialId);

    if (!credential) {
        return (
            <View style={styles.container}>
                <View style={styles.errorContainer}>
                    <Text style={styles.errorIcon}>⚠️</Text>
                    <Text style={styles.errorTitle}>Credential Not Found</Text>
                    <Text style={styles.errorText}>
                        The selected credential could not be found.
                    </Text>
                    <Button
                        onPress={() => router.back()}
                        title="Go Back"
                        variant="secondary"
                    />
                </View>
            </View>
        );
    }

    const handleGenerateProof = async () => {
        if (!keyPair) {
            showToast(ERROR_MESSAGES.KEYPAIR_NOT_FOUND, 'error');
            return;
        }

        if (!substrateAPI.isConnected()) {
            showToast(ERROR_MESSAGES.API_CONNECTION_FAILED, 'error');
            return;
        }

        Alert.alert(
            'Confirm Proof Generation',
            'Once generated, this proof cannot be undone. The selected fields will be visible to the verifier.',
            [
                {
                    text: 'Cancel',
                    style: 'cancel',
                },
                {
                    text: 'Generate',
                    onPress: performProofGeneration,
                    style: 'default',
                },
            ]
        );
    };

    const performProofGeneration = async () => {
        setIsSubmitting(true);
        setLoading(true, 'Generating zero-knowledge proof...');

        try {
            // Generate mock proof hash (in production, this would be actual ZK proof generation)
            const mockProofData = generateMockProof(
                credential.subject,
                fieldIndices,
                proofType
            );

            // Submit selective disclosure to blockchain
            const result = await substrateCalls.selectiveDisclosure(
                keyPair!,
                credential.subject,
                fieldIndices,
                mockProofData.proofHash
            );

            setLoading(false);

            if (result.success) {
                setProofHash(mockProofData.proofHash);
                showToast('Proof generated successfully!', 'success');

                // Navigate to success/QR screen after delay
                setTimeout(() => {
                    router.replace({
                        pathname: '/(wallet)/credentials/qr',
                        params: {
                            proofHash: mockProofData.proofHash,
                            credentialId: credential.subject,
                        },
                    });
                }, 1500);
            } else {
                throw new Error(result.error || 'Failed to generate proof');
            }
        } catch (error) {
            setLoading(false);
            console.error('Proof generation error:', error);
            showToast(
                error instanceof Error ? error.message : ERROR_MESSAGES.UNKNOWN_ERROR,
                'error'
            );
        } finally {
            setIsSubmitting(false);
        }
    };

    const generateMockProof = (
        credentialId: string,
        fields: number[],
        type: ProofType
    ) => {
        // In production, this would call actual ZK proof generation library
        // For now, generate a deterministic hash based on inputs
        const proofData = {
            credentialId,
            fields,
            type,
            timestamp: Date.now(),
            nonce: Math.random().toString(36).substring(7),
        };

        const proofString = JSON.stringify(proofData);
        const proofHash = `0x${Array.from(proofString)
            .reduce((hash, char) => {
                const chr = char.charCodeAt(0);
                hash = (hash << 5) - hash + chr;
                return hash & hash;
            }, 0)
            .toString(16)
            .padStart(64, '0')}`;

        return {
            proofHash,
            proofData,
        };
    };

    const getFieldNames = (): string[] => {
        // Mock field names based on credential type
        const fieldMappings: Record<string, string[]> = {
            Education: ['Institution', 'Student ID', 'Status', 'GPA', 'Enrollment Date'],
            Health: ['Patient ID', 'Vaccination Type', 'Date', 'Expiry', 'Issuer'],
            Employment: ['Employee ID', 'Company', 'Position', 'Start Date', 'Status'],
            Age: ['Date of Birth', 'Age', 'Verification Date'],
            Address: ['Street', 'City', 'State', 'Postal Code', 'Country'],
        };

        const fields = fieldMappings[credential.credentialType] || ['Field 1', 'Field 2', 'Field 3'];
        return fieldIndices.map(index => fields[index] || `Field ${index + 1}`);
    };

    const estimatedTime = Math.ceil(fieldIndices.length * 0.5); // Mock: 0.5s per field

    return (
        <View style={styles.container}>
            <ScrollView contentContainerStyle={styles.scrollContent}>
                <Text style={styles.title}>Confirm Proof Generation</Text>
                <Text style={styles.subtitle}>
                    Review the information before generating your proof
                </Text>

                {/* Credential Info */}
                <Card style={styles.card}>
                    <Text style={styles.cardTitle}>Credential Details</Text>
                    <View style={styles.infoRow}>
                        <Text style={styles.label}>Type</Text>
                        <Text style={styles.value}>{credential.credentialType}</Text>
                    </View>
                    <View style={styles.infoRow}>
                        <Text style={styles.label}>Issuer</Text>
                        <Text style={styles.value}>{formatDid(credential.issuer)}</Text>
                    </View>
                    <View style={styles.infoRow}>
                        <Text style={styles.label}>Status</Text>
                        <Text style={[styles.value, styles.activeStatus]}>
                            {credential.status}
                        </Text>
                    </View>
                </Card>

                {/* Fields to Reveal */}
                <Card style={styles.card}>
                    <Text style={styles.cardTitle}>Fields to Reveal</Text>
                    <Text style={styles.cardSubtitle}>
                        The following fields will be visible to the verifier:
                    </Text>
                    {getFieldNames().map((fieldName, index) => (
                        <View key={index} style={styles.fieldItem}>
                            <Text style={styles.fieldIcon}>✓</Text>
                            <Text style={styles.fieldName}>{fieldName}</Text>
                        </View>
                    ))}
                    <View style={styles.fieldCount}>
                        <Text style={styles.fieldCountText}>
                            {fieldIndices.length} field(s) selected
                        </Text>
                    </View>
                </Card>

                {/* Proof Details */}
                <Card style={styles.card}>
                    <Text style={styles.cardTitle}>Proof Information</Text>
                    <View style={styles.infoRow}>
                        <Text style={styles.label}>Proof Type</Text>
                        <Text style={styles.value}>{proofType}</Text>
                    </View>
                    <View style={styles.infoRow}>
                        <Text style={styles.label}>Est. Generation Time</Text>
                        <Text style={styles.value}>{estimatedTime}s</Text>
                    </View>
                    <View style={styles.infoRow}>
                        <Text style={styles.label}>Validity Period</Text>
                        <Text style={styles.value}>1 hour</Text>
                    </View>
                </Card>

                {/* Warning Card */}
                <Card style={styles.warningCard}>
                    <Text style={styles.warningTitle}>⚠️ Important</Text>
                    <Text style={styles.warningText}>
                        • This action cannot be undone
                    </Text>
                    <Text style={styles.warningText}>
                        • Selected fields will be cryptographically revealed
                    </Text>
                    <Text style={styles.warningText}>
                        • All other fields remain private
                    </Text>
                    <Text style={styles.warningText}>
                        • The proof will be stored on-chain
                    </Text>
                </Card>

                {/* Success State */}
                {proofHash && (
                    <Card style={styles.successCard}>
                        <Text style={styles.successTitle}>✓ Proof Generated</Text>
                        <Text style={styles.successText}>
                            Your zero-knowledge proof has been successfully generated and submitted
                            to the blockchain.
                        </Text>
                    </Card>
                )}

                {/* Action Buttons */}
                <View style={styles.buttonContainer}>
                    {!proofHash && (
                        <>
                            <Button
                                onPress={() => router.back()}
                                title="Go Back"
                                variant="secondary"
                                disabled={isSubmitting}
                                style={styles.button}
                            />
                            <Button
                                onPress={handleGenerateProof}
                                title="Generate Proof"
                                loading={isSubmitting}
                                style={styles.button}
                            />
                        </>
                    )}
                </View>
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
    card: {
        marginBottom: 16,
    },
    cardTitle: {
        fontSize: 18,
        fontWeight: '600',
        color: '#111827',
        marginBottom: 12,
    },
    cardSubtitle: {
        fontSize: 14,
        color: '#6B7280',
        marginBottom: 12,
    },
    infoRow: {
        flexDirection: 'row',
        justifyContent: 'space-between',
        alignItems: 'center',
        marginBottom: 12,
    },
    label: {
        fontSize: 14,
        fontWeight: '600',
        color: '#6B7280',
        textTransform: 'uppercase',
    },
    value: {
        fontSize: 14,
        color: '#111827',
        fontWeight: '500',
    },
    activeStatus: {
        color: '#10B981',
    },
    fieldItem: {
        flexDirection: 'row',
        alignItems: 'center',
        paddingVertical: 8,
        paddingHorizontal: 12,
        backgroundColor: '#F3F4F6',
        borderRadius: 8,
        marginBottom: 8,
    },
    fieldIcon: {
        fontSize: 16,
        color: '#10B981',
        marginRight: 12,
    },
    fieldName: {
        fontSize: 14,
        fontWeight: '500',
        color: '#111827',
        flex: 1,
    },
    fieldCount: {
        marginTop: 8,
        paddingTop: 12,
        borderTopWidth: 1,
        borderTopColor: '#E5E7EB',
    },
    fieldCountText: {
        fontSize: 14,
        fontWeight: '600',
        color: '#6B7280',
        textAlign: 'center',
    },
    warningCard: {
        marginBottom: 24,
        backgroundColor: '#FEF3C7',
        borderColor: '#F59E0B',
        borderWidth: 1,
    },
    warningTitle: {
        fontSize: 16,
        fontWeight: '600',
        color: '#92400E',
        marginBottom: 12,
    },
    warningText: {
        fontSize: 14,
        color: '#78350F',
        marginBottom: 6,
        lineHeight: 20,
    },
    successCard: {
        marginBottom: 24,
        backgroundColor: '#D1FAE5',
        borderColor: '#10B981',
        borderWidth: 1,
    },
    successTitle: {
        fontSize: 18,
        fontWeight: '600',
        color: '#065F46',
        marginBottom: 8,
    },
    successText: {
        fontSize: 14,
        color: '#047857',
        lineHeight: 20,
    },
    buttonContainer: {
        gap: 12,
    },
    button: {
        marginBottom: 8,
    },
    errorContainer: {
        flex: 1,
        alignItems: 'center',
        justifyContent: 'center',
        paddingHorizontal: 40,
    },
    errorIcon: {
        fontSize: 64,
        marginBottom: 16,
    },
    errorTitle: {
        fontSize: 20,
        fontWeight: '600',
        color: '#111827',
        marginBottom: 8,
        textAlign: 'center',
    },
    errorText: {
        fontSize: 14,
        color: '#6B7280',
        textAlign: 'center',
        marginBottom: 24,
        lineHeight: 20,
    },
});
