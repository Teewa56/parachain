import React from 'react';
import { View, Text, StyleSheet, TouchableOpacity, Platform, StatusBar } from 'react-native';
import { router } from 'expo-router';

interface HeaderProps {
    title?: string;
    subtitle?: string;
    showBack?: boolean;
    rightAction?: {
        icon: string;
        onPress: () => void;
    };
    leftAction?: {
        icon: string;
        onPress: () => void;
    };
}

export const Header: React.FC<HeaderProps> = ({
    title,
    subtitle,
    showBack = false,
    rightAction,
    leftAction,
}) => {
    return (
        <View style={styles.container}>
            <View style={styles.content}>
                {/* Left Side */}
                <View style={styles.leftSection}>
                    {showBack ? (
                        <TouchableOpacity
                            onPress={() => router.back()}
                            style={styles.actionButton}
                            hitSlop={{ top: 10, bottom: 10, left: 10, right: 10 }}
                        >
                            <Text style={styles.backIcon}>←</Text>
                        </TouchableOpacity>
                    ) : leftAction ? (
                        <TouchableOpacity
                            onPress={leftAction.onPress}
                            style={styles.actionButton}
                            hitSlop={{ top: 10, bottom: 10, left: 10, right: 10 }}
                        >
                            <Text style={styles.actionIcon}>{leftAction.icon}</Text>
                        </TouchableOpacity>
                    ) : (
                        <View style={styles.placeholder} />
                    )}
                </View>

                {/* Center Title */}
                <View style={styles.centerSection}>
                    {title && <Text style={styles.title} numberOfLines={1}>{title}</Text>}
                    {subtitle && <Text style={styles.subtitle} numberOfLines={1}>{subtitle}</Text>}
                </View>

                {/* Right Action */}
                <View style={styles.rightSection}>
                    {rightAction ? (
                        <TouchableOpacity
                            onPress={rightAction.onPress}
                            style={styles.actionButton}
                            hitSlop={{ top: 10, bottom: 10, left: 10, right: 10 }}
                        >
                            <Text style={styles.actionIcon}>{rightAction.icon}</Text>
                        </TouchableOpacity>
                    ) : (
                        <View style={styles.placeholder} />
                    )}
                </View>
            </View>
        </View>
    );
};

const styles = StyleSheet.create({
    container: {
        backgroundColor: '#FFFFFF',
        borderBottomWidth: 1,
        borderBottomColor: '#E5E7EB',
        paddingTop: Platform.OS === 'android' ? StatusBar.currentHeight : 0,
    },
    content: {
        flexDirection: 'row',
        alignItems: 'center',
        justifyContent: 'space-between',
        paddingHorizontal: 16,
        paddingVertical: 12,
        minHeight: 56,
    },
    leftSection: {
        flex: 1,
        alignItems: 'flex-start',
    },
    centerSection: {
        flex: 3,
        alignItems: 'center',
    },
    rightSection: {
        flex: 1,
        alignItems: 'flex-end',
    },
    actionButton: {
        padding: 8,
        borderRadius: 8,
    },
    backIcon: {
        fontSize: 24,
        color: '#111827',
    },
    actionIcon: {
        fontSize: 20,
    },
    title: {
        fontSize: 18,
        fontWeight: '600',
        color: '#111827',
        textAlign: 'center',
    },
    subtitle: {
        fontSize: 12,
        color: '#6B7280',
        marginTop: 2,
        textAlign: 'center',
    },
    placeholder: {
        width: 40,
    },
});