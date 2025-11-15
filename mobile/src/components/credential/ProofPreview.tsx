import React from 'react';
import { View, Text, StyleSheet } from 'react-native';
import type { Credential, ProofType } from '../../types/substrate';
import { formatTimestamp } from '../../services/substrate/utils';

interface ProofPreviewProps {
    credential: Credential;
    selectedFields: number[];
    proofType: ProofType;
}

export const ProofPreview: React.FC<ProofPreviewProps> = ({
    credential,
    selectedFields,
    proofType,
}) => {
    const getProofTypeName = (): string => {
        const typeNames: Record<ProofType, string> = {
            AgeAbove: 'Age Verification',
            StudentStatus: 'Student Status',
            VaccinationStatus: 'Vaccination Status',
            EmploymentStatus: 'Employment Status',
            Custom: 'Custom Proof',
        };
        return typeNames[proofType] || 'Unknown';
    };

    const getProofDescription = (): string => {
        const descriptions: Record<ProofType, string> = {
            AgeAbove: 'Proves age is above a certain threshold without revealing exact birthdate',
            StudentStatus: 'Proves active student enrollment without revealing personal details',
            VaccinationStatus: 'Proves vaccination status without revealing medical history',
            EmploymentStatus: 'Proves employment without revealing salary information',
            Custom: 'Custom zero-knowledge proof for specific use case',
        };
        return descriptions[proofType] || 'Custom verification proof';
    };

    const getFieldPreview = (): string[] => {
        const fieldMappings: Record<string, string[]> = {
            Education: ['Institution', 'Student ID', 'Status', 'GPA', 'Enrollment Date', 'Graduation Date', 'Major', 'Degree Type'],
            Health: ['Patient ID', 'Vaccination Type', 'Vaccination Date', 'Expiry Date', 'Batch Number', 'Doses', 'Healthcare Provider'],
            Employment: ['Employee ID', 'Company', 'Position', 'Start Date', 'End Date', 'Status', 'Department', 'Salary Range'],
            Age: ['Date of Birth', 'Age', 'Age Threshold', 'Verification Date'],
            Address: ['Street Address', 'City', 'State/Province', 'Postal Code', 'Country', 'Residence Type'],
        };

        const fields = fieldMappings[credential.credentialType] || [];
        return selectedFields.map(index => fields[index] || `Field ${index + 1}`);
    };

    const revealedFields = getFieldPreview();
    const totalFieldsForType = {
        Education: 8,
        Health: 7,
        Employment: 8,
        Age: 4,
        Address: 6,
    }[credential.credentialType] || 3;
    const hiddenFieldsCount = totalFieldsForType - selectedFields.length;

    return (
        <View style={styles.container}>
            <View style={styles.section}>
                <Text style={styles.sectionTitle}>Proof Type</Text>
                <View style={styles.proofTypeContainer}>
                    <Text style={styles.proofTypeIcon}>🔐</Text>
                    <View style={styles.proofTypeInfo}>
                        <Text style={styles.proofTypeName}>{getProofTypeName()}</Text>
                        <Text style={styles.proofTypeDescription}>{getProofDescription()}</Text>
                    </View>
                </View>
            </View>

            <View style={styles.divider} />

            <View style={styles.section}>
                <Text style={styles.sectionTitle}>What Will Be Revealed</Text>
                <View style={styles.revealedContainer}>
                    {revealedFields.map((field, index) => (
                        <View key={index} style={styles.revealedField}>
                            <Text style={styles.revealedIcon}>✓</Text>
                            <Text style={styles.revealedText}>{field}</Text>
                        </View>
                    ))}
                </View>
            </View>

            <View style={styles.divider} />

            <View style={styles.section}>
                <Text style={styles.sectionTitle}>What Remains Private</Text>
                <View style={styles.privateContainer}>
                    <Text style={styles.privateIcon}>🔒</Text>
                    <Text style={styles.privateText}>
                        {hiddenFieldsCount} field(s) will remain cryptographically hidden
                    </Text>
                </View>
                <Text style={styles.privateSubtext}>
                    These fields are protected by zero-knowledge cryptography and cannot be revealed
                    without generating a new proof.
                </Text>
            </View>

            <View style={styles.divider} />

            <View style={styles.section}>
                <Text style={styles.sectionTitle}>Proof Details</Text>
                <View style={styles.detailsGrid}>
                    <View style={styles.detailItem}>
                        <Text style={styles.detailLabel}>Privacy Level</Text>
                        <View style={styles.privacyLevel}>
                            <View style={styles.privacyDot} />
                            <Text style={styles.detailValue}>
                                {selectedFields.length <= 2 ? 'High' : selectedFields.length <= 4 ? 'Medium' : 'Standard'}
                            </Text>
                        </View>
                    </View>
                    <View style={styles.detailItem}>
                        <Text style={styles.detailLabel}>Fields Disclosed</Text>
                        <Text style={styles.detailValue}>
                            {selectedFields.length} / {totalFieldsForType}
                        </Text>
                    </View>
                    <View style={styles.detailItem}>
                        <Text style={styles.detailLabel}>Valid Until</Text>
                        <Text style={styles.detailValue}>
                            {formatTimestamp(Math.floor(Date.now() / 1000) + 3600)}
                        </Text>
                    </View>
                    <View style={styles.detailItem}>
                        <Text style={styles.detailLabel}>Verification Method</Text>
                        <Text style={styles.detailValue}>Groth16 ZK-SNARK</Text>
                    </View>
                </View>
            </View>

            <View style={styles.guaranteeBox}>
                <Text style={styles.guaranteeTitle}>🛡️ Privacy Guarantee</Text>
                <Text style={styles.guaranteeText}>
                    Only the fields you've selected will be revealed. All other information is
                    mathematically proven to remain hidden, even from the verifier.
                </Text>
            </View>
        </View>
    );
};

const styles = StyleSheet.create({
    container: {
        backgroundColor: '#FFFFFF',
        borderRadius: 12,
        padding: 16,
        borderWidth: 1,
        borderColor: '#E5E7EB',
    },
    section: {
        marginBottom: 16,
    },
    sectionTitle: {
        fontSize: 14,
        fontWeight: '600',
        color: '#6B7280',
        textTransform: 'uppercase',
        marginBottom: 12,
        letterSpacing: 0.5,
    },
    proofTypeContainer: {
        flexDirection: 'row',
        alignItems: 'flex-start',
        backgroundColor: '#F9FAFB',
        padding: 12,
        borderRadius: 8,
    },
    proofTypeIcon: {
        fontSize: 24,
        marginRight: 12,
    },
    proofTypeInfo: {
        flex: 1,
    },
    proofTypeName: {
        fontSize: 16,
        fontWeight: '600',
        color: '#111827',
        marginBottom: 4,
    },
    proofTypeDescription: {
        fontSize: 13,
        color: '#6B7280',
        lineHeight: 18,
    },
    divider: {
        height: 1,
        backgroundColor: '#E5E7EB',
        marginVertical: 16,
    },
    revealedContainer: {
        gap: 8,
    },
    revealedField: {
        flexDirection: 'row',
        alignItems: 'center',
        paddingVertical: 8,
        paddingHorizontal: 12,
        backgroundColor: '#ECFDF5',
        borderRadius: 6,
        borderLeftWidth: 3,
        borderLeftColor: '#10B981',
    },
    revealedIcon: {
        fontSize: 14,
        color: '#10B981',
        marginRight: 10,
    },
    revealedText: {
        fontSize: 14,
        fontWeight: '500',
        color: '#065F46',
    },
    privateContainer: {
        flexDirection: 'row',
        alignItems: 'center',
        padding: 12,
        backgroundColor: '#FEF3C7',
        borderRadius: 8,
        marginBottom: 8,
    },
    privateIcon: {
        fontSize: 20,
        marginRight: 12,
    },
    privateText: {
        fontSize: 14,
        fontWeight: '600',
        color: '#92400E',
        flex: 1,
    },
    privateSubtext: {
        fontSize: 12,
        color: '#78350F',
        lineHeight: 18,
        paddingHorizontal: 4,
    },
    detailsGrid: {
        flexDirection: 'row',
        flexWrap: 'wrap',
        gap: 12,
    },
    detailItem: {
        width: '48%',
        backgroundColor: '#F9FAFB',
        padding: 12,
        borderRadius: 8,
    },
    detailLabel: {
        fontSize: 12,
        fontWeight: '600',
        color: '#6B7280',
        marginBottom: 6,
    },
    detailValue: {
        fontSize: 14,
        fontWeight: '600',
        color: '#111827',
    },
    privacyLevel: {
        flexDirection: 'row',
        alignItems: 'center',
        gap: 6,
    },
    privacyDot: {
        width: 8,
        height: 8,
        borderRadius: 4,
        backgroundColor: '#10B981',
    },
    guaranteeBox: {
        marginTop: 8,
        padding: 16,
        backgroundColor: '#EEF2FF',
        borderRadius: 8,
        borderWidth: 1,
        borderColor: '#6366F1',
    },
    guaranteeTitle: {
        fontSize: 14,
        fontWeight: '600',
        color: '#3730A3',
        marginBottom: 6,
    },
    guaranteeText: {
        fontSize: 13,
        color: '#4338CA',
        lineHeight: 18,
    },
});
