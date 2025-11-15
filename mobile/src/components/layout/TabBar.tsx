import React from 'react';
import { View, Text, StyleSheet, TouchableOpacity, Platform } from 'react-native';
import { router, usePathname } from 'expo-router';
import { useSafeAreaInsets } from 'react-native-safe-area-context';

interface TabItem {
    name: string;
    icon: string;
    label: string;
    path: string;
}

const tabs: TabItem[] = [
    { name: 'home', icon: '🏠', label: 'Home', path: '/(wallet)' },
    { name: 'credentials', icon: '📄', label: 'Credentials', path: '/(wallet)/credentials' },
    { name: 'identity', icon: '👤', label: 'Identity', path: '/(wallet)/identity' },
    { name: 'proof', icon: '🔐', label: 'Proofs', path: '/(wallet)/proof' },
    { name: 'settings', icon: '⚙️', label: 'Settings', path: '/(wallet)/settings' },
];

export const TabBar: React.FC = () => {
    const pathname = usePathname();
    const insets = useSafeAreaInsets();

    const isActive = (tabPath: string): boolean => {
        if (tabPath === '/(wallet)') {
            return pathname === '/(wallet)' || pathname === '/';
        }
        return pathname.startsWith(tabPath);
    };

    const handleTabPress = (path: string) => {
        router.push(path as any);
    };

    return (
        <View
            style={[
                styles.container,
                {
                    paddingBottom: Platform.OS === 'ios' ? insets.bottom : 8,
                    height: Platform.OS === 'ios' ? 80 + insets.bottom : 60,
                },
            ]}
        >
            {tabs.map((tab) => {
                const active = isActive(tab.path);
                return (
                    <TouchableOpacity
                        key={tab.name}
                        onPress={() => handleTabPress(tab.path)}
                        style={styles.tab}
                        activeOpacity={0.7}
                    >
                        <Text style={[styles.icon, active && styles.iconActive]}>
                            {tab.icon}
                        </Text>
                        <Text style={[styles.label, active && styles.labelActive]}>
                            {tab.label}
                        </Text>
                        {active && <View style={styles.activeIndicator} />}
                    </TouchableOpacity>
                );
            })}
        </View>
    );
};

const styles = StyleSheet.create({
    container: {
        flexDirection: 'row',
        backgroundColor: '#FFFFFF',
        borderTopWidth: 1,
        borderTopColor: '#E5E7EB',
        paddingTop: 8,
    },
    tab: {
        flex: 1,
        alignItems: 'center',
        justifyContent: 'center',
        paddingVertical: 8,
        position: 'relative',
    },
    icon: {
        fontSize: 24,
        marginBottom: 4,
        opacity: 0.5,
    },
    iconActive: {
        opacity: 1,
    },
    label: {
        fontSize: 11,
        color: '#6B7280',
        fontWeight: '500',
    },
    labelActive: {
        color: '#6366F1',
        fontWeight: '600',
    },
    activeIndicator: {
        position: 'absolute',
        top: 0,
        width: 32,
        height: 3,
        backgroundColor: '#6366F1',
        borderRadius: 2,
    },
});