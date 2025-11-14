import React from 'react';
import { 
    TouchableOpacity, 
    Text, 
    StyleSheet, 
    ActivityIndicator, 
    ViewStyle, 
    TextStyle 
} from 'react-native';

interface ButtonProps {
    onPress: () => void;
    title: string;
    variant?: 'primary' | 'secondary' | 'danger' | 'outline';
    disabled?: boolean;
    loading?: boolean;
    style?: ViewStyle;
    textStyle?: TextStyle;
}

export const Button: React.FC<ButtonProps> = ({
    onPress,
    title,
    variant = 'primary',
    disabled = false,
    loading = false,
    style,
    textStyle,
}) => {
    const buttonStyles = [
        styles.button,
        styles[variant],
        disabled && styles.disabled,
        style,
    ];

    const textStyles = [
        styles.text,
        styles[`${variant}Text`],
        disabled && styles.disabledText,
        textStyle,
    ];

    return (
        <TouchableOpacity
            style={buttonStyles}
            onPress={onPress}
            disabled={disabled || loading}
            activeOpacity={0.7}
        >
            {loading ? (
                <ActivityIndicator 
                    color={variant === 'primary' ? '#FFFFFF' : '#6366F1'} 
                />
            ) : (
                <Text style={textStyles}>{title}</Text>
            )}
        </TouchableOpacity>
    );
};

const styles = StyleSheet.create({
    button: {
        paddingVertical: 16,
        paddingHorizontal: 24,
        borderRadius: 12,
        alignItems: 'center',
        justifyContent: 'center',
        minHeight: 56,
    },
    primary: {
        backgroundColor: '#6366F1',
    },
    secondary: {
        backgroundColor: '#F3F4F6',
    },
    danger: {
        backgroundColor: '#EF4444',
    },
    outline: {
        backgroundColor: 'transparent',
        borderWidth: 2,
        borderColor: '#6366F1',
    },
    disabled: {
        opacity: 0.5,
    },
    text: {
        fontSize: 16,
        fontWeight: '600',
        textAlign: 'center',
    },
    primaryText: {
        color: '#FFFFFF',
    },
    secondaryText: {
        color: '#111827',
    },
    dangerText: {
        color: '#FFFFFF',
    },
    outlineText: {
        color: '#6366F1',
    },
    disabledText: {
        opacity: 0.5,
    },
});