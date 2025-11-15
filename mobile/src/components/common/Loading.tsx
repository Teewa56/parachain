import React from 'react';
import { View, Text, ActivityIndicator, StyleSheet, Modal } from 'react-native';

interface LoadingProps {
    visible: boolean;
    message?: string;
}

export const Loading: React.FC<LoadingProps> = ({ visible, message }) => {
    return (
        <Modal visible={visible} transparent animationType="fade">
            <View style={styles.overlay}>
                <View style={styles.container}>
                    <ActivityIndicator size="large" color="#6366F1" />
                    {message && <Text style={styles.message}>{message}</Text>}
                </View>
            </View>
        </Modal>
    );
};

const styles = StyleSheet.create({
    overlay: { flex: 1, backgroundColor: 'rgba(0, 0, 0, 0.5)', justifyContent: 'center', alignItems: 'center' },
    container: { backgroundColor: '#FFFFFF', borderRadius: 16, padding: 32, alignItems: 'center', minWidth: 200 },
    message: { marginTop: 16, fontSize: 16, color: '#111827', textAlign: 'center' },
});