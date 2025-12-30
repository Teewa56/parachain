"""
Feature preprocessing and engineering for behavioral biometrics
"""

import numpy as np
from typing import Dict, List, Any
from sklearn.preprocessing import StandardScaler


class FeaturePreprocessor:
    """
    Preprocess behavioral biometric features
    
    Steps:
        1. Extract raw features
        2. Validate and clip outliers
        3. Add derived features
        4. Normalize (if scaler provided)
    """
    
    def __init__(self):
        self.feature_mins = {
            'typing_speed_wpm': 10,
            'avg_key_hold_time_ms': 20,
            'avg_transition_time_ms': 20,
            'error_rate_percent': 0,
            'activity_hour_preference': 0,
        }
        
        self.feature_maxs = {
            'typing_speed_wpm': 200,
            'avg_key_hold_time_ms': 500,
            'avg_transition_time_ms': 400,
            'error_rate_percent': 50,
            'activity_hour_preference': 23,
        }
    
    def process_features(self, features: Dict[str, Any]) -> List[float]:
        """
        Process features from dictionary to list
        
        Args:
            features: Dictionary of feature name -> value
            
        Returns:
            List of processed feature values
        """
        # Extract and validate features
        processed = []
        
        for feature_name in [
            'typing_speed_wpm',
            'avg_key_hold_time_ms',
            'avg_transition_time_ms',
            'error_rate_percent',
            'activity_hour_preference',
        ]:
            value = float(features.get(feature_name, 0))
            
            # Clip to reasonable bounds
            value = np.clip(
                value,
                self.feature_mins[feature_name],
                self.feature_maxs[feature_name]
            )
            
            processed.append(value)
        
        return processed
    
    def create_scaler(self, X: np.ndarray) -> StandardScaler:
        """
        Create and fit StandardScaler on training data
        
        Args:
            X: Training data array (n_samples, n_features)
            
        Returns:
            Fitted StandardScaler
        """
        scaler = StandardScaler()
        scaler.fit(X)
        return scaler
    
    def batch_process(
        self,
        features_list: List[Dict[str, Any]]
    ) -> np.ndarray:
        """
        Process batch of features
        
        Args:
            features_list: List of feature dictionaries
            
        Returns:
            NumPy array of shape (n_samples, n_features)
        """
        processed_batch = []
        
        for features in features_list:
            processed = self.process_features(features)
            processed_batch.append(processed)
        
        return np.array(processed_batch)


class TemporalFeatureExtractor:
    """
    Extract temporal features from time-series behavioral data
    
    Useful for analyzing typing patterns over time
    """
    
    def __init__(self, window_size: int = 10):
        """
        Args:
            window_size: Number of samples in sliding window
        """
        self.window_size = window_size
    
    def extract_temporal_features(
        self,
        time_series: List[Dict[str, Any]]
    ) -> Dict[str, float]:
        """
        Extract statistical features from time series
        
        Features:
            - Mean, std, min, max of each feature
            - Trend (linear regression slope)
            - Variability (coefficient of variation)
        
        Args:
            time_series: List of feature dictionaries with timestamps
            
        Returns:
            Dictionary of temporal features
        """
        if len(time_series) < 2:
            return {}
        
        # Extract typing speeds
        speeds = [t['typing_speed_wpm'] for t in time_series]
        
        # Calculate statistics
        temporal_features = {
            'speed_mean': float(np.mean(speeds)),
            'speed_std': float(np.std(speeds)),
            'speed_min': float(np.min(speeds)),
            'speed_max': float(np.max(speeds)),
            'speed_trend': self._calculate_trend(speeds),
            'speed_cv': self._coefficient_of_variation(speeds),
        }
        
        return temporal_features
    
    def _calculate_trend(self, values: List[float]) -> float:
        """
        Calculate linear trend (slope)
        
        Returns:
            Slope of best-fit line
        """
        if len(values) < 2:
            return 0.0
        
        x = np.arange(len(values))
        y = np.array(values)
        
        # Linear regression
        coeffs = np.polyfit(x, y, 1)
        slope = coeffs[0]
        
        return float(slope)
    
    def _coefficient_of_variation(self, values: List[float]) -> float:
        """
        Calculate coefficient of variation (std / mean)
        
        Returns:
            CV value
        """
        mean = np.mean(values)
        std = np.std(values)
        
        if mean == 0:
            return 0.0
        
        return float(std / mean)


def augment_data(
    X: np.ndarray,
    y: np.ndarray,
    noise_level: float = 0.05,
    augmentation_factor: int = 2
) -> tuple[np.ndarray, np.ndarray]:
    """
    Augment training data with noise
    
    Args:
        X: Feature array (n_samples, n_features)
        y: Label array (n_samples,)
        noise_level: Std dev of Gaussian noise (as fraction of feature std)
        augmentation_factor: How many augmented copies to create
        
    Returns:
        Augmented (X, y) tuple
    """
    X_augmented = [X]
    y_augmented = [y]
    
    for _ in range(augmentation_factor):
        # Add Gaussian noise
        noise = np.random.normal(0, noise_level, X.shape) * X.std(axis=0)
        X_noisy = X + noise
        
        # Clip to valid ranges
        X_noisy = np.clip(X_noisy, 0, None)
        
        X_augmented.append(X_noisy)
        y_augmented.append(y)
    
    X_final = np.vstack(X_augmented)
    y_final = np.concatenate(y_augmented)
    
    return X_final, y_final