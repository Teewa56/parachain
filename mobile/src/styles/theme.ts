import { colors } from './colors';
import { spacing } from './spacing';
import { typography } from './typography';

export interface Theme {
    colors: typeof colors;
    spacing: typeof spacing;
    typography: typeof typography;
    shadows: {
        small: object;
        medium: object;
        large: object;
    };
    animation: {
        duration: {
            fast: number;
            normal: number;
            slow: number;
        };
    };
}

export const lightTheme: Theme = {
    colors,
    spacing,
    typography,

    shadows: {
        small: {
            shadowColor: '#000',
            shadowOffset: { width: 0, height: 1 },
            shadowOpacity: 0.05,
            shadowRadius: 2,
            elevation: 1,
        },
        medium: {
            shadowColor: '#000',
            shadowOffset: { width: 0, height: 2 },
            shadowOpacity: 0.1,
            shadowRadius: 4,
            elevation: 2,
        },
        large: {
            shadowColor: '#000',
            shadowOffset: { width: 0, height: 4 },
            shadowOpacity: 0.15,
            shadowRadius: 8,
            elevation: 4,
        },
    },

    animation: {
        duration: {
            fast: 150,
            normal: 300,
            slow: 500,
        },
    },
};

// Dark theme 
export const darkTheme: Theme = {
    ...lightTheme,
    colors: {
        ...colors,
        background: '#111827',
        surface: '#1F2937',
        text: {
            primary: '#F9FAFB',
            secondary: '#D1D5DB',
            tertiary: '#9CA3AF',
            disabled: '#91a6cfff',
            inverse: '#111827',
        },
    },
};

// Helper to get responsive values
export const responsive = {
    isSmallScreen: (width: number) => width < 375,
    isMediumScreen: (width: number) => width >= 375 && width < 768,
    isLargeScreen: (width: number) => width >= 768,
};