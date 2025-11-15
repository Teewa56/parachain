import { Platform, TextStyle } from 'react-native';

export const typography = {
    // Font families
    fontFamily: {
        regular: Platform.select({
            ios: 'System',
            android: 'Roboto',
            default: 'System',
        }),
        medium: Platform.select({
            ios: 'System',
            android: 'Roboto-Medium',
            default: 'System',
        }),
        semibold: Platform.select({
            ios: 'System',
            android: 'Roboto-Medium',
            default: 'System',
        }),
        bold: Platform.select({
            ios: 'System',
            android: 'Roboto-Bold',
            default: 'System',
        }),
        monospace: Platform.select({
            ios: 'Courier',
            android: 'monospace',
            default: 'monospace',
        }),
    },

    // Font sizes
    fontSize: {
        xs: 11,
        sm: 12,
        base: 14,
        md: 16,
        lg: 18,
        xl: 20,
        '2xl': 24,
        '3xl': 28,
        '4xl': 32,
        '5xl': 36,
        '6xl': 48,
    },

    // Line heights
    lineHeight: {
        tight: 1.2,
        normal: 1.5,
        relaxed: 1.75,
        loose: 2,
    },

    // Font weights
    fontWeight: {
        normal: '400' as TextStyle['fontWeight'],
        medium: '500' as TextStyle['fontWeight'],
        semibold: '600' as TextStyle['fontWeight'],
        bold: '700' as TextStyle['fontWeight'],
    },

    // Letter spacing
    letterSpacing: {
        tight: -0.5,
        normal: 0,
        wide: 0.5,
        wider: 1,
    },

    // Predefined text styles
    h1: {
        fontSize: 32,
        fontWeight: '700' as TextStyle['fontWeight'],
        lineHeight: 40,
        letterSpacing: -0.5,
    },

    h2: {
        fontSize: 28,
        fontWeight: '700' as TextStyle['fontWeight'],
        lineHeight: 36,
        letterSpacing: -0.5,
    },

    h3: {
        fontSize: 24,
        fontWeight: '600' as TextStyle['fontWeight'],
        lineHeight: 32,
    },

    h4: {
        fontSize: 20,
        fontWeight: '600' as TextStyle['fontWeight'],
        lineHeight: 28,
    },

    h5: {
        fontSize: 18,
        fontWeight: '600' as TextStyle['fontWeight'],
        lineHeight: 24,
    },

    h6: {
        fontSize: 16,
        fontWeight: '600' as TextStyle['fontWeight'],
        lineHeight: 22,
    },

    body: {
        fontSize: 16,
        fontWeight: '400' as TextStyle['fontWeight'],
        lineHeight: 24,
    },

    bodyLarge: {
        fontSize: 18,
        fontWeight: '400' as TextStyle['fontWeight'],
        lineHeight: 28,
    },

    bodySmall: {
        fontSize: 14,
        fontWeight: '400' as TextStyle['fontWeight'],
        lineHeight: 20,
    },

    caption: {
        fontSize: 12,
        fontWeight: '400' as TextStyle['fontWeight'],
        lineHeight: 16,
    },

    captionSmall: {
        fontSize: 11,
        fontWeight: '400' as TextStyle['fontWeight'],
        lineHeight: 14,
    },

    button: {
        fontSize: 16,
        fontWeight: '600' as TextStyle['fontWeight'],
        lineHeight: 24,
    },

    buttonSmall: {
        fontSize: 14,
        fontWeight: '600' as TextStyle['fontWeight'],
        lineHeight: 20,
    },

    label: {
        fontSize: 14,
        fontWeight: '600' as TextStyle['fontWeight'],
        lineHeight: 20,
    },

    labelSmall: {
        fontSize: 12,
        fontWeight: '600' as TextStyle['fontWeight'],
        lineHeight: 16,
    },

    code: {
        fontSize: 14,
        fontWeight: '400' as TextStyle['fontWeight'],
        lineHeight: 20,
        fontFamily: Platform.select({
            ios: 'Courier',
            android: 'monospace',
            default: 'monospace',
        }),
    },
};