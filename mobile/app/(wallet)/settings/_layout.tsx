import { Stack } from 'expo-router';

export default function SettingsLayout() {
    return (
        <Stack
            screenOptions={{
                headerStyle: {
                    backgroundColor: '#FFFFFF',
                },
                headerTintColor: '#111827',
                headerTitleStyle: {
                    fontWeight: '600',
                },
                headerShadowVisible: true,
            }}
        >
            <Stack.Screen
                name="index"
                options={{
                    title: 'Settings',
                    headerBackTitle: 'Back',
                }}
            />
            <Stack.Screen
                name="biometric"
                options={{
                    title: 'Biometric Authentication',
                    headerBackTitle: 'Settings',
                }}
            />
            <Stack.Screen
                name="backup"
                options={{
                    title: 'Backup & Security',
                    headerBackTitle: 'Settings',
                }}
            />
        </Stack>
    );
}