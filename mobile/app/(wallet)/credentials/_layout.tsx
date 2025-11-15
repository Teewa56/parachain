import { Stack } from 'expo-router';

export default function AuthLayout() {
  return (
    <Stack
      screenOptions={{
        headerShown: false,
        contentStyle: { backgroundColor: '#FFFFFF' },
      }}
    >
      <Stack.Screen name="[id]]" />
      <Stack.Screen name="qr" />
      <Stack.Screen name="share" />
    </Stack>
  );
}