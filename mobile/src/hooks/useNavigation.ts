import { router } from 'expo-router';
import type { ScreenName } from '../types';

export const useNavigation = () => {
    const navigate = (screen: ScreenName, params?: any) => {
        router.push({ pathname: screen as any, params });
    };

    const goBack = () => {
        router.back();
    };

    const replace = (screen: ScreenName, params?: any) => {
        router.replace({ pathname: screen as any, params });
    };

    return { navigate, goBack, replace };
};