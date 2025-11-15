import React from 'react';
import { Modal as RNModal, View, Text, StyleSheet, TouchableOpacity } from 'react-native';
import { Button } from './Button';

interface ModalProps {
    visible: boolean;
    onClose: () => void;
    title?: string;
    children: React.ReactNode;
    actions?: Array<{ label: string; onPress: () => void; variant?: 'primary' | 'secondary' | 'danger' }>;
}

export const Modal: React.FC<ModalProps> = ({ visible, onClose, title, children, actions }) => {
    return (
        <RNModal visible={visible} transparent animationType="slide" onRequestClose={onClose}>
            <View style={styles3.overlay}>
                <View style={styles3.container}>
                    {title && (
                        <View style={styles3.header}>
                            <Text style={styles3.title}>{title}</Text>
                            <TouchableOpacity onPress={onClose} style={styles3.closeButton}>
                                <Text style={styles3.closeText}>✕</Text>
                            </TouchableOpacity>
                        </View>
                    )}
                    <View style={styles3.content}>{children}</View>
                    {actions && (
                        <View style={styles3.actions}>
                            {actions.map((action, index) => (
                                <Button
                                    key={index}
                                    onPress={action.onPress}
                                    title={action.label}
                                    variant={action.variant}
                                    style={styles3.actionButton}
                                />
                            ))}
                        </View>
                    )}
                </View>
            </View>
        </RNModal>
    );
};

const styles3 = StyleSheet.create({
    overlay: { flex: 1, backgroundColor: 'rgba(0, 0, 0, 0.5)', justifyContent: 'flex-end' },
    container: { backgroundColor: '#FFFFFF', borderTopLeftRadius: 24, borderTopRightRadius: 24, maxHeight: '90%' },
    header: { flexDirection: 'row', justifyContent: 'space-between', alignItems: 'center', padding: 20, borderBottomWidth: 1, borderBottomColor: '#E5E7EB' },
    title: { fontSize: 20, fontWeight: '700', color: '#111827' },
    closeButton: { padding: 4 },
    closeText: { fontSize: 24, color: '#6B7280' },
    content: { padding: 20 },
    actions: { padding: 20, borderTopWidth: 1, borderTopColor: '#E5E7EB', gap: 12 },
    actionButton: {},
});