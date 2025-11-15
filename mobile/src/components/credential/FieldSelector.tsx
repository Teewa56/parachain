import React from 'react';
import { View, Text, StyleSheet, TouchableOpacity, ScrollView } from 'react-native';
import type { Credential } from '../../types/substrate';

interface FieldSelectorProps {
    credential: Credential;
    selectedFields: { [key: number]: boolean };
    onFieldToggle: (fieldIndex: number) => void;
}

interface FieldDefinition {
    index: number;
    name: string;
    description: string;
    required: boolean;
}

export const FieldSelector: React.FC<FieldSelectorProps> = ({
    credential,
    selectedFields,
    onFieldToggle,
}) => {
    const getFieldsForCredentialType = (): FieldDefinition[] => {
        const fieldMappings: Record<string, FieldDefinition[]> = {
            Education: [
                { index: 0, name: 'Institution', description: 'Educational institution name', required: true },
                { index: 1, name: 'Student ID', description: 'Unique student identifier', required: false },
                { index: 2, name: 'Status', description: 'Current enrollment status', required: true },
                { index: 3, name: 'GPA', description: 'Grade point average', required: false },
                { index: 4, name: 'Enrollment Date', description: 'Date of initial enrollment', required: false },
                { index: 5, name: 'Graduation Date', description: 'Expected or actual graduation date', required: false },
                { index: 6, name: 'Major', description: 'Primary field of study', required: false },
                { index: 7, name: 'Degree Type', description: 'Type of degree program', required: false },
            ],
            Health: [
                { index: 0, name: 'Patient ID', description: 'Unique patient identifier', required: false },
                { index: 1, name: 'Vaccination Type', description: 'Type of vaccination received', required: true },
                { index: 2, name: 'Vaccination Date', description: 'Date vaccine was administered', required: true },
                { index: 3, name: 'Expiry Date', description: 'Vaccination certificate expiry', required: false },
                { index: 4, name: 'Batch Number', description: 'Vaccine batch/lot number', required: false },
                { index: 5, name: 'Doses', description: 'Number of doses completed', required: true },
                { index: 6, name: 'Healthcare Provider', description: 'Name of administering facility', required: false },
            ],
            Employment: [
                { index: 0, name: 'Employee ID', description: 'Unique employee identifier', required: false },
                { index: 1, name: 'Company', description: 'Employer company name', required: true },
                { index: 2, name: 'Position', description: 'Job title or role', required: true },
                { index: 3, name: 'Start Date', description: 'Employment start date', required: true },
                { index: 4, name: 'End Date', description: 'Employment end date (if applicable)', required: false },
                { index: 5, name: 'Status', description: 'Current employment status', required: true },
                { index: 6, name: 'Department', description: 'Department or division', required: false },
                { index: 7, name: 'Salary Range', description: 'Compensation bracket', required: false },
            ],
            Age: [
                { index: 0, name: 'Date of Birth', description: 'Birth date', required: false },
                { index: 1, name: 'Age', description: 'Current age', required: true },
                { index: 2, name: 'Age Threshold', description: 'Minimum age verification', required: true },
                { index: 3, name: 'Verification Date', description: 'Date of age verification', required: true },
            ],
            Address: [
                { index: 0, name: 'Street Address', description: 'Street and house number', required: false },
                { index: 1, name: 'City', description: 'City name', required: true },
                { index: 2, name: 'State/Province', description: 'State or province', required: true },
                { index: 3, name: 'Postal Code', description: 'ZIP or postal code', required: false },
                { index: 4, name: 'Country', description: 'Country name', required: true },
                { index: 5, name: 'Residence Type', description: 'Type of residence', required: false },
            ],
        };

        return (
            fieldMappings[credential.credentialType] || [
                { index: 0, name: 'Field 1', description: 'Custom field 1', required: false },
                { index: 1, name: 'Field 2', description: 'Custom field 2', required: false },
                { index: 2, name: 'Field 3', description: 'Custom field 3', required: false },
            ]
        );
    };

    const fields = getFieldsForCredentialType();

    const handleSelectAll = () => {
        fields.forEach(field => {
            if (!selectedFields[field.index]) {
                onFieldToggle(field.index);
            }
        });
    };

    const handleDeselectAll = () => {
        fields.forEach(field => {
            if (selectedFields[field.index]) {
                onFieldToggle(field.index);
            }
        });
    };

    const selectedCount = Object.values(selectedFields).filter(Boolean).length;
    const allSelected = selectedCount === fields.length;

    return (
        <View style={styles.container}>
            <View style={styles.header}>
                <Text style={styles.headerTitle}>Select Fields</Text>
                <View style={styles.headerActions}>
                    <TouchableOpacity
                        onPress={allSelected ? handleDeselectAll : handleSelectAll}
                        style={styles.headerButton}
                    >
                        <Text style={styles.headerButtonText}>
                            {allSelected ? 'Deselect All' : 'Select All'}
                        </Text>
                    </TouchableOpacity>
                </View>
            </View>

            <Text style={styles.helperText}>
                Choose which fields to reveal to the verifier. Required fields are marked with *.
            </Text>

            <ScrollView style={styles.fieldsContainer} nestedScrollEnabled>
                {fields.map(field => (
                    <TouchableOpacity
                        key={field.index}
                        onPress={() => onFieldToggle(field.index)}
                        style={[
                            styles.fieldItem,
                            selectedFields[field.index] && styles.fieldItemSelected,
                        ]}
                        activeOpacity={0.7}
                    >
                        <View style={styles.fieldLeft}>
                            <View
                                style={[
                                    styles.checkbox,
                                    selectedFields[field.index] && styles.checkboxSelected,
                                ]}
                            >
                                {selectedFields[field.index] && (
                                    <Text style={styles.checkmark}>✓</Text>
                                )}
                            </View>
                            <View style={styles.fieldInfo}>
                                <View style={styles.fieldNameRow}>
                                    <Text style={styles.fieldName}>
                                        {field.name}
                                        {field.required && (
                                            <Text style={styles.requiredIndicator}> *</Text>
                                        )}
                                    </Text>
                                </View>
                                <Text style={styles.fieldDescription}>{field.description}</Text>
                            </View>
                        </View>
                    </TouchableOpacity>
                ))}
            </ScrollView>

            <View style={styles.footer}>
                <View style={styles.legend}>
                    <Text style={styles.legendItem}>
                        <Text style={styles.requiredIndicator}>*</Text> Required field
                    </Text>
                    <Text style={styles.legendItem}>
                        ✓ {selectedCount} selected
                    </Text>
                </View>
            </View>
        </View>
    );
};

const styles = StyleSheet.create({
    container: {
        width: '100%',
    },
    header: {
        flexDirection: 'row',
        justifyContent: 'space-between',
        alignItems: 'center',
        marginBottom: 12,
    },
    headerTitle: {
        fontSize: 16,
        fontWeight: '600',
        color: '#111827',
    },
    headerActions: {
        flexDirection: 'row',
        gap: 8,
    },
    headerButton: {
        paddingVertical: 4,
        paddingHorizontal: 12,
    },
    headerButtonText: {
        fontSize: 14,
        fontWeight: '600',
        color: '#6366F1',
    },
    helperText: {
        fontSize: 12,
        color: '#6B7280',
        marginBottom: 16,
        lineHeight: 18,
    },
    fieldsContainer: {
        maxHeight: 400,
    },
    fieldItem: {
        flexDirection: 'row',
        alignItems: 'center',
        justifyContent: 'space-between',
        paddingVertical: 12,
        paddingHorizontal: 12,
        backgroundColor: '#F9FAFB',
        borderRadius: 8,
        marginBottom: 8,
        borderWidth: 2,
        borderColor: 'transparent',
    },
    fieldItemSelected: {
        backgroundColor: '#EEF2FF',
        borderColor: '#6366F1',
    },
    fieldLeft: {
        flexDirection: 'row',
        alignItems: 'center',
        flex: 1,
    },
    checkbox: {
        width: 24,
        height: 24,
        borderRadius: 6,
        borderWidth: 2,
        borderColor: '#D1D5DB',
        backgroundColor: '#FFFFFF',
        alignItems: 'center',
        justifyContent: 'center',
        marginRight: 12,
    },
    checkboxSelected: {
        borderColor: '#6366F1',
        backgroundColor: '#6366F1',
    },
    checkmark: {
        color: '#FFFFFF',
        fontSize: 14,
        fontWeight: '700',
    },
    fieldInfo: {
        flex: 1,
    },
    fieldNameRow: {
        flexDirection: 'row',
        alignItems: 'center',
        marginBottom: 4,
    },
    fieldName: {
        fontSize: 14,
        fontWeight: '600',
        color: '#111827',
    },
    requiredIndicator: {
        color: '#EF4444',
        fontWeight: '700',
    },
    fieldDescription: {
        fontSize: 12,
        color: '#6B7280',
        lineHeight: 16,
    },
    footer: {
        marginTop: 16,
        paddingTop: 16,
        borderTopWidth: 1,
        borderTopColor: '#E5E7EB',
    },
    legend: {
        flexDirection: 'row',
        justifyContent: 'space-between',
        alignItems: 'center',
    },
    legendItem: {
        fontSize: 12,
        color: '#6B7280',
    },
});
