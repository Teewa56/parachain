import { Stack } from 'expo-router';

export default function IdentityLayout() {
  return (
    <Stack
      screenOptions={{
        headerStyle: { backgroundColor: '#FFFFFF' },
        headerTintColor: '#111827',
        headerTitleStyle: { fontWeight: '600' },
        headerShadowVisible: true,
      }}
  >
    <Stack.Screen name="index" options={{ title: 'My Identity' }} />
    <Stack.Screen name="[id]" options={{ title: 'Identity Details' }} />
    <Stack.Screen name="create" options={{ title: 'Create Identity' }} />
    <Stack.Screen name="manage" options={{ title: 'Manage Identity' }} />
    </Stack>
  );
}