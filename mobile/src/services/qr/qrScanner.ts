import { CameraView, useCameraPermissions } from 'expo-camera';
import { useState } from 'react';

export const useQRScanner = () => {
    const [permission, requestPermission] = useCameraPermissions();
    const [scanned, setScanned] = useState(false);

    const handleScan = (data: string, onScan: (data: string) => void) => {
        if (scanned) return;
        setScanned(true);
        onScan(data);
        setTimeout(() => setScanned(false), 2000);
    };

    return {
        hasPermission: permission?.granted,
        requestPermission,
        handleScan,
        resetScanned: () => setScanned(false),
    };
};