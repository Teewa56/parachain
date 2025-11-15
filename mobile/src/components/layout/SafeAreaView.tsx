import React from 'react';
import { SafeAreaView as RNSafeAreaView, StyleSheet, ViewStyle, Platform } from 'react-native';
import { useSafeAreaInsets } from 'react-native-safe-area-context';

interface SafeAreaViewProps {
    children: React.ReactNode;
    style?: ViewStyle;
    edges?: Array<'top' | 'bottom' | 'left' | 'right'>;
    backgroundColor?: string;
}

export const SafeAreaView: React.FC<SafeAreaViewProps> = ({
    children,
    style,
    edges = ['top', 'bottom', 'left', 'right'],
    backgroundColor = '#FFFFFF',
}) => {
    const insets = useSafeAreaInsets();

    const paddingStyle = {
        paddingTop: edges.includes('top') ? insets.top : 0,
        paddingBottom: edges.includes('bottom') ? insets.bottom : 0,
        paddingLeft: edges.includes('left') ? insets.left : 0,
        paddingRight: edges.includes('right') ? insets.right : 0,
    };

    return (
        <RNSafeAreaView
            style={[
                styles.container,
                { backgroundColor },
                paddingStyle,
                style,
            ]}
        >
            {children}
        </RNSafeAreaView>
    );
};

const styles = StyleSheet.create({
    container: {
        flex: 1,
    },
});
