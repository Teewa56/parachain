import React, { useState } from 'react';
import { View, Text, StyleSheet, Alert } from 'react-native';
import { Card } from '../common/Card';
import { Button } from '../common/Button';
import { keyManagement } from '../../services/crypto/keyManagement';

interface KeyPairManagerProps {
    onExport?: (json: string) => void;
    onDelete?: () => void;
}

export const KeyPairManager: React.FC<KeyPairManagerProps> = ({ onExport, onDelete }) => {
    const [isExporting, setIsExporting] = useState(false);

    const handleExport = async () => {
        Alert.prompt(
            'Export Keypair',
            'Enter a password to encrypt your keypair',
            async (password) => {
                if (!password) return;
                setIsExporting(true);
                try {
                    const json = await keyManagement.exportKeyPairJson(password);
                    onExport?.(json);
                } catch (error) {
                    Alert.alert('Error', 'Failed to export keypair');
                } finally {
                    setIsExporting(false);
                }
            },
            'secure-text'
        );
    };

    const handleDelete = () => {
        Alert.alert(
            'Delete Keypair',
            'Are you sure? This cannot be undone. Make sure you have backed up your seed phrase.',
            [
                { text: 'Cancel', style: 'cancel' },
                { text: 'Delete', style: 'destructive', onPress: onDelete },
            ]
        );
    };

    return (
        <Card>
            <Text style={styles7.title}>Keypair Management</Text>
            <Button
                onPress={handleExport}
                title="Export Encrypted Keypair"
                loading={isExporting}
                variant="secondary"
                style={styles7.button}
            />
            <Button
                onPress={handleDelete}
                title="Delete Keypair"
                variant="danger"
            />
        </Card>
    );
};

const styles7 = StyleSheet.create({
    title: { fontSize: 18, fontWeight: '600', color: '#111827', marginBottom: 16 },
    button: { marginBottom: 12 },
});