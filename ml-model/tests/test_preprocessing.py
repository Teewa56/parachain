"""
Feature Preprocessing Tests
"""

import pytest
import numpy as np
from pathlib import Path
import sys

sys.path.append(str(Path(__file__).parent.parent))

from ml.preprocessing import (
    FeaturePreprocessor,
    TemporalFeatureExtractor,
    augment_data,
)


class TestFeaturePreprocessor:
    """Test cases for FeaturePreprocessor"""
    
    def test_initialization(self):
        """Test preprocessor initialization"""
        preprocessor = FeaturePreprocessor()
        assert preprocessor is not None
        assert hasattr(preprocessor, 'feature_mins')
        assert hasattr(preprocessor, 'feature_maxs')
    
    def test_process_features(self):
        """Test feature processing"""
        preprocessor = FeaturePreprocessor()
        
        features = {
            'typing_speed_wpm': 65,
            'avg_key_hold_time_ms': 120,
            'avg_transition_time_ms': 85,
            'error_rate_percent': 3,
            'activity_hour_preference': 14,
        }
        
        processed = preprocessor.process_features(features)
        
        # Check output is list
        assert isinstance(processed, list)
        assert len(processed) == 5
        
        # Check values are floats
        assert all(isinstance(x, float) for x in processed)
    
    def test_feature_clipping(self):
        """Test features are clipped to valid ranges"""
        preprocessor = FeaturePreprocessor()
        
        # Features with out-of-range values
        features = {
            'typing_speed_wpm': 500,  # Too high
            'avg_key_hold_time_ms': -50,  # Negative
            'avg_transition_time_ms': 85,
            'error_rate_percent': 150,  # Too high
            'activity_hour_preference': 14,
        }
        
        processed = preprocessor.process_features(features)
        
        # Check values are clipped
        assert processed[0] <= 200  # typing_speed max
        assert processed[1] >= 20   # key_hold_time min
        assert processed[3] <= 50   # error_rate max
    
    def test_batch_processing(self):
        """Test batch feature processing"""
        preprocessor = FeaturePreprocessor()
        
        features_list = [
            {
                'typing_speed_wpm': 65,
                'avg_key_hold_time_ms': 120,
                'avg_transition_time_ms': 85,
                'error_rate_percent': 3,
                'activity_hour_preference': 14,
            },
            {
                'typing_speed_wpm': 70,
                'avg_key_hold_time_ms': 115,
                'avg_transition_time_ms': 90,
                'error_rate_percent': 5,
                'activity_hour_preference': 16,
            }
        ]
        
        batch = preprocessor.batch_process(features_list)
        
        # Check output shape
        assert batch.shape == (2, 5)
        assert isinstance(batch, np.ndarray)
    
    def test_scaler_creation(self):
        """Test StandardScaler creation"""
        preprocessor = FeaturePreprocessor()
        
        # Create sample data
        X = np.random.randn(100, 5)
        
        scaler = preprocessor.create_scaler(X)
        
        # Check scaler is fitted
        assert hasattr(scaler, 'mean_')
        assert hasattr(scaler, 'scale_')
        assert scaler.mean_.shape == (5,)


class TestTemporalFeatureExtractor:
    """Test cases for TemporalFeatureExtractor"""
    
    def test_initialization(self):
        """Test temporal extractor initialization"""
        extractor = TemporalFeatureExtractor(window_size=10)
        assert extractor.window_size == 10
    
    def test_extract_temporal_features(self):
        """Test temporal feature extraction"""
        extractor = TemporalFeatureExtractor()
        
        # Create time series data
        time_series = [
            {'typing_speed_wpm': 65, 'timestamp': 1000},
            {'typing_speed_wpm': 63, 'timestamp': 2000},
            {'typing_speed_wpm': 67, 'timestamp': 3000},
            {'typing_speed_wpm': 64, 'timestamp': 4000},
        ]
        
        features = extractor.extract_temporal_features(time_series)
        
        # Check expected features exist
        assert 'speed_mean' in features
        assert 'speed_std' in features
        assert 'speed_trend' in features
        assert 'speed_cv' in features
    
    def test_insufficient_data(self):
        """Test behavior with insufficient data"""
        extractor = TemporalFeatureExtractor()
        
        # Only one sample
        time_series = [{'typing_speed_wpm': 65, 'timestamp': 1000}]
        
        features = extractor.extract_temporal_features(time_series)
        
        # Should return empty dict
        assert features == {}
    
    def test_trend_calculation(self):
        """Test trend calculation"""
        extractor = TemporalFeatureExtractor()
        
        # Increasing trend
        values = [10, 20, 30, 40, 50]
        trend = extractor._calculate_trend(values)
        
        # Trend should be positive
        assert trend > 0
        
        # Decreasing trend
        values = [50, 40, 30, 20, 10]
        trend = extractor._calculate_trend(values)
        
        # Trend should be negative
        assert trend < 0


class TestDataAugmentation:
    """Test cases for data augmentation"""
    
    def test_augment_data(self):
        """Test data augmentation"""
        X = np.random.randn(100, 5)
        y = np.random.randint(0, 2, 100)
        
        X_aug, y_aug = augment_data(X, y, noise_level=0.1, augmentation_factor=2)
        
        # Check augmented data size
        expected_size = 100 * (1 + 2)  # Original + 2 augmented copies
        assert X_aug.shape[0] == expected_size
        assert y_aug.shape[0] == expected_size
        
        # Check features are preserved
        assert X_aug.shape[1] == 5
    
    def test_augmentation_noise(self):
        """Test augmentation adds noise"""
        X = np.ones((10, 5)) * 100  # Constant values
        y = np.ones(10)
        
        X_aug, y_aug = augment_data(X, y, noise_level=0.1, augmentation_factor=1)
        
        # Augmented data should differ from original
        # (first 10 are original, rest are augmented)
        original = X_aug[:10]
        augmented = X_aug[10:]
        
        # Check they're not identical
        assert not np.allclose(original, augmented)
    
    def test_labels_preserved(self):
        """Test labels are correctly duplicated"""
        X = np.random.randn(50, 5)
        y = np.array([0] * 25 + [1] * 25)
        
        X_aug, y_aug = augment_data(X, y, augmentation_factor=2)
        
        # Check label distribution is preserved
        original_ratio = y.mean()
        augmented_ratio = y_aug.mean()
        
        assert np.isclose(original_ratio, augmented_ratio, atol=0.01)


if __name__ == "__main__":
    # Run tests
    pytest.main([__file__, "-v", "--tb=short"])