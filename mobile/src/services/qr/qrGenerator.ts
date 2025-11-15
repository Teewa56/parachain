export interface QRData {
    type: 'credential' | 'proof' | 'did';
    data: string;
    timestamp: number;
    expiresAt?: number;
}

export const generateQRData = (type: QRData['type'], data: string, expiresIn?: number): string => {
    const qrData: QRData = {
        type,
        data,
        timestamp: Date.now(),
        expiresAt: expiresIn ? Date.now() + expiresIn : undefined,
    };
    return JSON.stringify(qrData);
};

export const parseQRData = (qrString: string): QRData | null => {
    try {
        const data = JSON.parse(qrString);
        if (!data.type || !data.data || !data.timestamp) return null;
        if (data.expiresAt && Date.now() > data.expiresAt) return null;
        return data;
    } catch {
        return null;
    }
};