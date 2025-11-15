import { useState, useEffect } from 'react';
import {
    View,
    Text,
    StyleSheet,
    ScrollView,
    TouchableOpacity,
    Alert,
    Switch,
} from 'react-native';
import { router } from 'expo-router';
import { useAuthStore, useIdentityStore, useUIStore } from '../../../src/store';
import { biometricAuth } from '../../../src/services/storage/biometric';
import { substrateAPI } from '../../../src/substrate/api';
import { Card } from '../../../src/components/common/Card';
import { APP_VERSION, config } from '../../../src/config/env';
import AsyncStorage from '@react-native-async-storage/async-storage';

export default function SettingsScreen() {
    const [biometricEnabled, setBiometricEnabled] = useState(false);
    const [networkStatus, setNetworkStatus] = useState<'connected' | 'disconnected' | 'connecting'>('disconnected');
    const [biometricType, setBiometricType] = useState('Biometric');

    const logout = useAuthStore(state => state.logout);
    const clearIdentity = useIdentityStore(state => state.clearIdentity);
    const address = useIdentityStore(state => state.address);
    const did = useIdentityStore(state => state.did);
    const showToast = useUIStore(state => state.showToast);

    useEffect(() => {
        loadSettings();
        checkNetworkStatus();
    }, []);

    const loadSettings = async () => {
        try {
            const enabled = await biometricAuth.isBiometricEnabled();
            setBiometricEnabled(enabled);

            const typeName = await biometricAuth.getBiometricTypeName();
            setBiometricType(typeName);
        } catch (error) {
            console.error('Load settings failed:', error);
        }
    };

    const checkNetworkStatus = () => {
        const status = substrateAPI.getConnectionStatus();
        setNetworkStatus(status as any);
    };

    const handleLogout = () => {
        Alert.alert(
            'Logout',
            'Are you sure you want to logout? You will need your seed phrase to restore your wallet.',
            [
                { text: 'Cancel', style: 'cancel' },
                {
                    text: 'Logout',
                    style: 'destructive',
                    onPress: performLogout,
                },
            ]
        );
    };

    const performLogout = async () => {
        try {
            await logout();
            await clearIdentity();
            showToast('Logged out successfully', 'success');
            router.replace('/(auth)/login');
        } catch (error) {
            console.error('Logout failed:', error);
            showToast('Failed to logout', 'error');
        }
    };

    const handleClearCache = () => {
        Alert.alert(
            'Clear Cache',
            'This will clear all cached data. Your credentials and identity will not be affected.',
            [
                { text: 'Cancel', style: 'cancel' },
                {
                    text: 'Clear',
                    onPress: performClearCache,
                },
            ]
        );
    };

    const performClearCache = async () => {
        try {
            // Clear cached data but preserve essential data
            const keysToPreserve = [
                '@identity_wallet/is_authenticated',
                '@identity_wallet/biometric_enabled',
                '@identity_wallet/current_did',
                '@identity_wallet/did_hash',
            ];

            const allKeys = await AsyncStorage.getAllKeys();
            const keysToRemove = allKeys.filter(key => !keysToPreserve.includes(key));
            
            await AsyncStorage.multiRemove(keysToRemove);
            showToast('Cache cleared successfully', 'success');
        } catch (error) {
            console.error('Clear cache failed:', error);
            showToast('Failed to clear cache', 'error');
        }
    };

    const getNetworkStatusColor = () => {
        switch (networkStatus) {
            case 'connected':
                return '#10B981';
            case 'connecting':
                return '#F59E0B';
            default:
                return '#EF4444';
        }
    };

    const getNetworkStatusText = () => {
        switch (networkStatus) {
            case 'connected':
                return 'Connected';
            case 'connecting':
                return 'Connecting...';
            default:
                return 'Disconnected';
        }
    };

    return (
        <View style={styles.container}>
            <ScrollView contentContainerStyle={styles.scrollContent}>
                <Text style={styles.title}>Settings</Text>
                <Text style={styles.subtitle}>Manage your wallet preferences</Text>

                {/* Account Info */}
                <View style={styles.section}>
                    <Text style={styles.sectionTitle}>Account</Text>
                    
                    <Card style={styles.infoCard}>
                        <View style={styles.infoRow}>
                            <Text style={styles.infoLabel}>Address</Text>
                            <Text style={styles.infoValue} numberOfLines={1}>
                                {address ? `${address.slice(0, 6)}...${address.slice(-4)}` : 'Not set'}
                            </Text>
                        </View>
                        <View style={styles.divider} />
                        <View style={styles.infoRow}>
                            <Text style={styles.infoLabel}>DID</Text>
                            <Text style={styles.infoValue} numberOfLines={1}>
                                {did ? `${did.slice(0, 20)}...` : 'Not set'}
                            </Text>
                        </View>
                        <View style={styles.divider} />
                        <View style={styles.infoRow}>
                            <Text style={styles.infoLabel}>Network</Text>
                            <View style={styles.networkStatus}>
                                <View
                                    style={[
                                        styles.statusDot,
                                        { backgroundColor: getNetworkStatusColor() },
                                    ]}
                                />
                                <Text style={styles.infoValue}>{getNetworkStatusText()}</Text>
                            </View>
                        </View>
                    </Card>
                </View>

                {/* Security Settings */}
                <View style={styles.section}>
                    <Text style={styles.sectionTitle}>Security</Text>

                    <TouchableOpacity
                        style={styles.settingItem}
                        onPress={() => router.push('/(wallet)/settings/biometric')}
                    >
                        <View style={styles.settingLeft}>
                            <Text style={styles.settingIcon}>👆</Text>
                            <View style={styles.settingText}>
                                <Text style={styles.settingTitle}>{biometricType}</Text>
                                <Text style={styles.settingDescription}>
                                    {biometricEnabled ? 'Enabled' : 'Disabled'}
                                </Text>
                            </View>
                        </View>
                        <Text style={styles.settingArrow}>›</Text>
                    </TouchableOpacity>

                    <TouchableOpacity
                        style={styles.settingItem}
                        onPress={() => router.push('/(wallet)/settings/backup')}
                    >
                        <View style={styles.settingLeft}>
                            <Text style={styles.settingIcon}>🔑</Text>
                            <View style={styles.settingText}>
                                <Text style={styles.settingTitle}>Backup & Recovery</Text>
                                <Text style={styles.settingDescription}>
                                    Seed phrase and PIN management
                                </Text>
                            </View>
                        </View>
                        <Text style={styles.settingArrow}>›</Text>
                    </TouchableOpacity>
                </View>

                {/* Network Settings */}
                <View style={styles.section}>
                    <Text style={styles.sectionTitle}>Network</Text>

                    <Card style={styles.infoCard}>
                        <View style={styles.infoRow}>
                            <Text style={styles.infoLabel}>Network Name</Text>
                            <Text style={styles.infoValue}>{config.networkName}</Text>
                        </View>
                        <View style={styles.divider} />
                        <View style={styles.infoRow}>
                            <Text style={styles.infoLabel}>Endpoint</Text>
                            <Text style={styles.infoValue} numberOfLines={1}>
                                {config.wsEndpoint}
                            </Text>
                        </View>
                        <View style={styles.divider} />
                        <View style={styles.infoRow}>
                            <Text style={styles.infoLabel}>Para ID</Text>
                            <Text style={styles.infoValue}>{config.paraId}</Text>
                        </View>
                    </Card>
                </View>

                {/* App Info */}
                <View style={styles.section}>
                    <Text style={styles.sectionTitle}>About</Text>

                    <Card style={styles.infoCard}>
                        <View style={styles.infoRow}>
                            <Text style={styles.infoLabel}>Version</Text>
                            <Text style={styles.infoValue}>{APP_VERSION}</Text>
                        </View>
                        <View style={styles.divider} />
                        <View style={styles.infoRow}>
                            <Text style={styles.infoLabel}>Environment</Text>
                            <Text style={styles.infoValue}>{config.env}</Text>
                        </View>
                    </Card>
                </View>

                {/* Actions */}
                <View style={styles.section}>
                    <Text style={styles.sectionTitle}>Actions</Text>

                    <TouchableOpacity
                        style={styles.actionItem}
                        onPress={handleClearCache}
                    >
                        <Text style={styles.actionIcon}>🗑️</Text>
                        <Text style={styles.actionText}>Clear Cache</Text>
                    </TouchableOpacity>

                    <TouchableOpacity
                        style={[styles.actionItem, styles.dangerAction]}
                        onPress={handleLogout}
                    >
                        <Text style={styles.actionIcon}>🚪</Text>
                        <Text style={[styles.actionText, styles.dangerText]}>Logout</Text>
                    </TouchableOpacity>
                </View>

                {/* Footer */}
                <View style={styles.footer}>
                    <Text style={styles.footerText}>
                        Identity Wallet v{APP_VERSION}
                    </Text>
                    <Text style={styles.footerText}>
                        Powered by Polkadot
                    </Text>
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
    section: {
        marginBottom: 24,
    },
    sectionTitle: {
        fontSize: 18,
        fontWeight: '600',
        color: '#111827',
        marginBottom: 12,
    },
    infoCard: {
        padding: 16,
    },
    infoRow: {
        flexDirection: 'row',
        justifyContent: 'space-between',
        alignItems: 'center',
        paddingVertical: 12,
    },
    infoLabel: {
        fontSize: 14,
        color: '#6B7280',
    },
    infoValue: {
        fontSize: 14,
        fontWeight: '600',
        color: '#111827',
        flex: 1,
        textAlign: 'right',
        marginLeft: 16,
    },
    networkStatus: {
        flexDirection: 'row',
        alignItems: 'center',
        gap: 8,
    },
    statusDot: {
        width: 8,
        height: 8,
        borderRadius: 4,
    },
    divider: {
        height: 1,
        backgroundColor: '#E5E7EB',
    },
    settingItem: {
        flexDirection: 'row',
        alignItems: 'center',
        justifyContent: 'space-between',
        backgroundColor: '#FFFFFF',
        padding: 16,
        borderRadius: 12,
        marginBottom: 8,
        shadowColor: '#000',
        shadowOffset: { width: 0, height: 1 },
        shadowOpacity: 0.05,
        shadowRadius: 2,
        elevation: 1,
    },
    settingLeft: {
        flexDirection: 'row',
        alignItems: 'center',
        flex: 1,
    },
    settingIcon: {
        fontSize: 24,
        marginRight: 12,
    },
    settingText: {
        flex: 1,
    },
    settingTitle: {
        fontSize: 16,
        fontWeight: '600',
        color: '#111827',
        marginBottom: 2,
    },
    settingDescription: {
        fontSize: 13,
        color: '#6B7280',
    },
    settingArrow: {
        fontSize: 24,
        color: '#D1D5DB',
        marginLeft: 8,
    },
    actionItem: {
        flexDirection: 'row',
        alignItems: 'center',
        backgroundColor: '#FFFFFF',
        padding: 16,
        borderRadius: 12,
        marginBottom: 8,
        shadowColor: '#000',
        shadowOffset: { width: 0, height: 1 },
        shadowOpacity: 0.05,
        shadowRadius: 2,
        elevation: 1,
    },
    dangerAction: {
        borderWidth: 1,
        borderColor: '#FEE2E2',
        backgroundColor: '#FEF2F2',
    },
    actionIcon: {
        fontSize: 20,
        marginRight: 12,
    },
    actionText: {
        fontSize: 16,
        fontWeight: '600',
        color: '#111827',
    },
    dangerText: {
        color: '#DC2626',
    },
    footer: {
        alignItems: 'center',
        paddingVertical: 24,
    },
    footerText: {
        fontSize: 13,
        color: '#9CA3AF',
        marginBottom: 4,
    },
});