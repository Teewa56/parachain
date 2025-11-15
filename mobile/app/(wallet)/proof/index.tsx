import { useState, useEffect } from 'react';
import {
    View,
    Text,
    StyleSheet,
    ScrollView,
    TouchableOpacity,
    Alert,
    ActivityIndicator,
} from 'react-native';
import { router } from 'expo-router';
import { useCredentialStore, useUIStore } from '../../../src/store';
import { Card } from '../../../src/components/common/Card';
import { Button } from '../../../src/components/common/Button';
import { FieldSelector } from '../../../src/components/credential/FieldSelector';
import { ProofPreview } from '../../../src/components/credential/ProofPreview';
import type { Credential, ProofType } from '../../../src/types/substrate';
import { formatDid } from '../../../src/services/substrate/utils';

interface SelectedFields {
    [key: number]: boolean;
}

export default function GenerateProofScreen() {
    const [selectedCredential, setSelectedCredential] = useState<Credential | null>(null);
    const [selectedFields, setSelectedFields] = useState<SelectedFields>({});
    const [proofType, setProofType] = useState<ProofType | null>(null);
    const [isGenerating, setIsGenerating] = useState(false);

    const credentials = useCredentialStore(state => state.credentials);
    const getActiveCredentials = useCredentialStore(state => state.getActiveCredentials);
    const showToast = useUIStore(state => state.showToast);
    const setLoading = useUIStore(state => state.setLoading);

    useEffect(() => {
        // Pre-filter only active credentials
        const active = getActiveCredentials();
        if (active.length === 0) {
            showToast('No active credentials available', 'warning');
        }
    }, []);

    const handleCredentialSelect = (credential: Credential) => {
        setSelectedCredential(credential);
        setSelectedFields({});
        
        // Auto-select proof type based on credential type
        const autoProofType = mapCredentialTypeToProofType(credential.credentialType);
        setProofType(autoProofType);
    };

    const mapCredentialTypeToProofType = (credType: string): ProofType => {
        switch (credType) {
            case 'Education':
                return 'StudentStatus' as ProofType;
            case 'Health':
                return 'VaccinationStatus' as ProofType;
            case 'Employment':
                return 'EmploymentStatus' as ProofType;
            case 'Age':
                return 'AgeAbove' as ProofType;
            default:
                return 'Custom' as ProofType;
        }
    };

    const handleFieldToggle = (fieldIndex: number) => {
        setSelectedFields(prev => ({
            ...prev,
            [fieldIndex]: !prev[fieldIndex],
        }));
    };

    const getSelectedFieldIndices = (): number[] => {
        return Object.entries(selectedFields)
            .filter(([_, selected]) => selected)
            .map(([index]) => parseInt(index));
    };

    const validateSelection = (): boolean => {
        if (!selectedCredential) {
            showToast('Please select a credential', 'warning');
            return false;
        }

        const fieldIndices = getSelectedFieldIndices();
        if (fieldIndices.length === 0) {
            showToast('Please select at least one field to reveal', 'warning');
            return false;
        }

        if (fieldIndices.length > 50) {
            showToast('Cannot reveal more than 50 fields', 'error');
            return false;
        }

        return true;
    };

    const handleGenerateProof = () => {
        if (!validateSelection()) {
            return;
        }

        Alert.alert(
            'Generate Proof',
            'This will create a zero-knowledge proof revealing only the selected fields. Continue?',
            [
                {
                    text: 'Cancel',
                    style: 'cancel',
                },
                {
                    text: 'Generate',
                    onPress: proceedToConfirmation,
                },
            ]
        );
    };

    const proceedToConfirmation = () => {
        if (!selectedCredential || !proofType) return;

        const fieldIndices = getSelectedFieldIndices();

        // Navigate to confirmation screen with parameters
        router.push({
            pathname: '/(wallet)/proof/confirm',
            params: {
                credentialId: selectedCredential.subject,
                fieldIndices: JSON.stringify(fieldIndices),
                proofType: proofType,
            },
        });
    };

    const activeCredentials = getActiveCredentials();

    if (activeCredentials.length === 0) {
        return (
            <View style={styles.container}>
                <View style={styles.emptyContainer}>
                    <Text style={styles.emptyIcon}>🔐</Text>
                    <Text style={styles.emptyTitle}>No Active Credentials</Text>
                    <Text style={styles.emptyText}>
                        You need active credentials to generate proofs. Credentials will appear here once issued.
                    </Text>
                    <Button
                        onPress={() => router.push('/(wallet)/credentials')}
                        title="View Credentials"
                        style={styles.emptyButton}
                    />
                </View>
            </View>
        );
    }

    return (
        <View style={styles.container}>
            <ScrollView contentContainerStyle={styles.scrollContent}>
                <Text style={styles.title}>Generate Zero-Knowledge Proof</Text>
                <Text style={styles.subtitle}>
                    Select a credential and choose which fields to reveal
                </Text>

                {/* Step 1: Select Credential */}
                <View style={styles.section}>
                    <Text style={styles.sectionTitle}>Step 1: Select Credential</Text>
                    {activeCredentials.map((credential, index) => (
                        <TouchableOpacity
                            key={`${credential.subject}-${index}`}
                            onPress={() => handleCredentialSelect(credential)}
                            activeOpacity={0.7}
                        >
                            <Card
                                style={[
                                    styles.credentialCard,
                                    selectedCredential?.subject === credential.subject &&
                                        styles.selectedCard,
                                ]}
                            >
                                <View style={styles.credentialHeader}>
                                    <Text style={styles.credentialType}>
                                        {getCredentialIcon(credential.credentialType)}{' '}
                                        {credential.credentialType}
                                    </Text>
                                    {selectedCredential?.subject === credential.subject && (
                                        <Text style={styles.selectedBadge}>✓ Selected</Text>
                                    )}
                                </View>
                                <Text style={styles.credentialIssuer}>
                                    Issuer: {formatDid(credential.issuer)}
                                </Text>
                            </Card>
                        </TouchableOpacity>
                    ))}
                </View>

                {/* Step 2: Select Fields */}
                {selectedCredential && (
                    <View style={styles.section}>
                        <Text style={styles.sectionTitle}>Step 2: Select Fields to Reveal</Text>
                        <Card style={styles.fieldSelectorCard}>
                            <FieldSelector
                                credential={selectedCredential}
                                selectedFields={selectedFields}
                                onFieldToggle={handleFieldToggle}
                            />
                        </Card>

                        <View style={styles.selectionSummary}>
                            <Text style={styles.summaryText}>
                                {getSelectedFieldIndices().length} field(s) selected
                            </Text>
                            {getSelectedFieldIndices().length > 0 && (
                                <TouchableOpacity
                                    onPress={() => setSelectedFields({})}
                                    style={styles.clearButton}
                                >
                                    <Text style={styles.clearButtonText}>Clear All</Text>
                                </TouchableOpacity>
                            )}
                        </View>
                    </View>
                )}

                {/* Step 3: Preview */}
                {selectedCredential && getSelectedFieldIndices().length > 0 && (
                    <View style={styles.section}>
                        <Text style={styles.sectionTitle}>Step 3: Preview Proof</Text>
                        <ProofPreview
                            credential={selectedCredential}
                            selectedFields={getSelectedFieldIndices()}
                            proofType={proofType!}
                        />
                    </View>
                )}

                {/* Privacy Notice */}
                {selectedCredential && (
                    <Card style={styles.privacyCard}>
                        <Text style={styles.privacyTitle}>🔒 Privacy Notice</Text>
                        <Text style={styles.privacyText}>
                            Only the selected fields will be revealed to the verifier. All other
                            information remains private and cryptographically hidden.
                        </Text>
                    </Card>
                )}

                {/* Generate Button */}
                {selectedCredential && getSelectedFieldIndices().length > 0 && (
                    <Button
                        onPress={handleGenerateProof}
                        title="Generate Proof"
                        loading={isGenerating}
                        style={styles.generateButton}
                    />
                )}
            </ScrollView>
        </View>
    );
}

const getCredentialIcon = (type: string): string => {
    switch (type) {
        case 'Education':
            return '🎓';
        case 'Health':
            return '🏥';
        case 'Employment':
            return '💼';
        case 'Age':
            return '🎂';
        case 'Address':
            return '🏠';
        default:
            return '📄';
    }
};

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
    credentialCard: {
        marginBottom: 12,
        borderWidth: 2,
        borderColor: 'transparent',
    },
    selectedCard: {
        borderColor: '#6366F1',
        backgroundColor: '#EEF2FF',
    },
    credentialHeader: {
        flexDirection: 'row',
        justifyContent: 'space-between',
        alignItems: 'center',
        marginBottom: 8,
    },
    credentialType: {
        fontSize: 18,
        fontWeight: '600',
        color: '#111827',
    },
    selectedBadge: {
        fontSize: 14,
        fontWeight: '600',
        color: '#6366F1',
    },
    credentialIssuer: {
        fontSize: 14,
        color: '#6B7280',
    },
    fieldSelectorCard: {
        padding: 16,
    },
    selectionSummary: {
        flexDirection: 'row',
        justifyContent: 'space-between',
        alignItems: 'center',
        marginTop: 12,
        paddingHorizontal: 4,
    },
    summaryText: {
        fontSize: 14,
        fontWeight: '600',
        color: '#6B7280',
    },
    clearButton: {
        paddingVertical: 4,
        paddingHorizontal: 12,
    },
    clearButtonText: {
        fontSize: 14,
        fontWeight: '600',
        color: '#EF4444',
    },
    privacyCard: {
        marginBottom: 20,
        backgroundColor: '#ECFDF5',
        borderColor: '#10B981',
        borderWidth: 1,
    },
    privacyTitle: {
        fontSize: 16,
        fontWeight: '600',
        color: '#065F46',
        marginBottom: 8,
    },
    privacyText: {
        fontSize: 14,
        color: '#047857',
        lineHeight: 20,
    },
    generateButton: {
        marginBottom: 32,
    },
    emptyContainer: {
        flex: 1,
        alignItems: 'center',
        justifyContent: 'center',
        paddingHorizontal: 40,
        paddingVertical: 60,
    },
    emptyIcon: {
        fontSize: 64,
        marginBottom: 16,
    },
    emptyTitle: {
        fontSize: 20,
        fontWeight: '600',
        color: '#111827',
        marginBottom: 8,
        textAlign: 'center',
    },
    emptyText: {
        fontSize: 14,
        color: '#6B7280',
        textAlign: 'center',
        lineHeight: 20,
        marginBottom: 24,
    },
    emptyButton: {
        minWidth: 200,
    },
});
