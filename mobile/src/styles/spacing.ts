export const spacing = {
    // Base spacing unit (4px)
    unit: 4,

    // Predefined spacing values
    none: 0,
    xxs: 2,    // 2px
    xs: 4,     // 4px
    sm: 8,     // 8px
    md: 16,    // 16px
    lg: 24,    // 24px
    xl: 32,    // 32px
    xxl: 48,   // 48px
    xxxl: 64,  // 64px

    // Specific use cases
    screenPadding: 20,
    cardPadding: 20,
    buttonPadding: 16,
    inputPadding: 16,
    iconSize: {
        small: 16,
        medium: 24,
        large: 32,
        xlarge: 48,
    },

    // Border radius
    radius: {
        none: 0,
        sm: 4,
        md: 8,
        lg: 12,
        xl: 16,
        xxl: 24,
        full: 9999,
    },

    // Component specific spacing
    card: {
        padding: 20,
        margin: 16,
        gap: 12,
    },

    button: {
        paddingVertical: 16,
        paddingHorizontal: 24,
        gap: 8,
    },

    input: {
        paddingVertical: 16,
        paddingHorizontal: 16,
        gap: 8,
    },

    header: {
        height: 56,
        paddingHorizontal: 16,
        paddingVertical: 12,
    },

    tabBar: {
        height: 60,
        heightIOS: 80,
        paddingVertical: 8,
    },

    // Layout spacing
    section: {
        marginBottom: 24,
        gap: 16,
    },

    list: {
        gap: 12,
        padding: 16,
    },
};

// Helper function to multiply spacing
export const space = (multiplier: number): number => {
    return spacing.unit * multiplier;
};