import { Stack } from 'expo-router';

export default function ProofLayout() {
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
                    title: 'Generate Proof',
                    headerBackTitle: 'Back',
                }}
            />
            <Stack.Screen
                name="confirm"
                options={{
                    title: 'Confirm Proof',
                    headerBackTitle: 'Back',
                }}
            />
            <Stack.Screen
                name="history"
                options={{
                    title: 'Proof History',
                    headerBackTitle: 'Back',
                }}
            />
        </Stack>
    );
}
