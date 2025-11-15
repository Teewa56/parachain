import React from 'react';
import { View, Text, StyleSheet, TouchableOpacity } from 'react-native';
import QRCode from 'react-native-qrcode-svg';

interface DIDDisplayProps {
    did: string;
    onCopy?: () => void;
}

export const DIDDisplay: React.FC<DIDDisplayProps> = ({ did, onCopy }) => {
    return (
        <View style={styles5.container}>
            <View style={styles5.qrContainer}>
                <QRCode value={did} size={200} />
            </View>
            <Text style={styles5.label}>Your DID</Text>
            <Text style={styles5.did} numberOfLines={1}>{did}</Text>
            {onCopy && (
                <TouchableOpacity onPress={onCopy} style={styles5.copyButton}>
                    <Text style={styles5.copyText}>📋 Copy DID</Text>
                </TouchableOpacity>
            )}
        </View>
    );
};

const styles5 = StyleSheet.create({
    container: { alignItems: 'center' },
    qrContainer: { padding: 20, backgroundColor: '#FFFFFF', borderRadius: 16, marginBottom: 16 },
    label: { fontSize: 14, fontWeight: '600', color: '#6B7280', marginBottom: 8 },
    did: { fontSize: 12, color: '#111827', fontFamily: 'monospace', marginBottom: 16 },
    copyButton: { paddingVertical: 8, paddingHorizontal: 16 },
    copyText: { fontSize: 14, fontWeight: '600', color: '#6366F1' },
});
