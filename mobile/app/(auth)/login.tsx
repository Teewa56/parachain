import { useState } from 'react';
import { View, Text, StyleSheet, TextInput, Alert } from 'react-native';
import { router } from 'expo-router';
import { useAuthStore, useIdentityStore, useUIStore } from '../../src/store';
import { biometricAuth } from '../../src/services/storage/biometric';
import { Button } from '../../src/components/common/Button';
import { Card } from '../../src/components/common/Card';

export default function LoginScreen() {
    const [pin, setPin] = useState('');
    const [loading, setLoading] = useState(false);

    const { login, biometricEnabled } = useAuthStore();
    const loadIdentity = useIdentityStore(state => state.loadIdentity);
    const showToast = useUIStore(state => state.showToast);

    const handleBiometricLogin = async () => {
        setLoading(true);
        try {
        const result = await biometricAuth.authenticate('Authenticate to access your wallet');
        
        if (result.success) {
            await login();
            await loadIdentity();
            showToast('Login successful!', 'success');
            router.replace('/(wallet)');
        } else {
            showToast(result.error || 'Authentication failed', 'error');
        }
        } catch (error) {
        showToast('Authentication failed', 'error');
        } finally {
        setLoading(false);
        }
    };

    const handlePinLogin = async () => {
        if (!pin || pin.length < 6) {
        showToast('Please enter your PIN', 'warning');
        return;
        }

        setLoading(true);
        try {
        const isValid = await biometricAuth.verifyFallbackPin(pin);
        
        if (isValid) {
            await login();
            await loadIdentity();
            showToast('Login successful!', 'success');
            router.replace('/(wallet)');
        } else {
            showToast('Incorrect PIN', 'error');
            setPin('');
        }
        } catch (error) {
        showToast('Login failed', 'error');
        } finally {
        setLoading(false);
        }
    };

    return (
        <View style={styles.container}>
        <View style={styles.header}>
            <Text style={styles.icon}>🔐</Text>
            <Text style={styles.title}>Welcome Back</Text>
            <Text style={styles.subtitle}>Access your decentralized identity</Text>
        </View>

        {biometricEnabled && (
            <Button
            onPress={handleBiometricLogin}
            title="Unlock with Biometrics"
            loading={loading}
            style={styles.button}
            />
        )}

        <Card style={styles.pinCard}>
            <Text style={styles.pinLabel}>Enter PIN</Text>
            <TextInput
            style={styles.pinInput}
            value={pin}
            onChangeText={setPin}
            keyboardType="number-pad"
            maxLength={8}
            secureTextEntry
            placeholder="Enter your PIN"
            placeholderTextColor="#9CA3AF"
            />
            <Button
            onPress={handlePinLogin}
            title="Login with PIN"
            loading={loading}
            disabled={pin.length < 6}
            style={styles.button}
            />
        </Card>

        <Button
            onPress={() => router.push('/(auth)/recovery')}
            title="Forgot PIN?"
            variant="secondary"
            style={styles.forgotButton}
        />

        <Button
            onPress={() => router.push('/(auth)/register')}
            title="Create New Identity"
            variant="outline"
            style={styles.registerButton}
        />
        </View>
    );
}

const styles = StyleSheet.create({
    container: { flex: 1, padding: 20, justifyContent: 'center', backgroundColor: '#FFFFFF' },
    header: { alignItems: 'center', marginBottom: 40 },
    icon: { fontSize: 64, marginBottom: 16 },
    title: { fontSize: 28, fontWeight: '700', color: '#111827', marginBottom: 8 },
    subtitle: { fontSize: 16, color: '#6B7280', textAlign: 'center' },
    pinCard: { marginBottom: 20 },
    pinLabel: { fontSize: 14, fontWeight: '600', color: '#111827', marginBottom: 8 },
    pinInput: { backgroundColor: '#F9FAFB', borderWidth: 1, borderColor: '#D1D5DB', borderRadius: 8, padding: 16, fontSize: 16, marginBottom: 16 },
    button: { marginBottom: 12 },
    forgotButton: { marginBottom: 12 },
    registerButton: { marginTop: 8 },
});