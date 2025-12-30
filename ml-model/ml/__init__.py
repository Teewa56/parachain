"""
Behavioral Biometrics ML Module

This module contains:
- PyTorch model architectures
- Training pipelines
- Inference engine
- Feature preprocessing
"""

from .model import BehavioralAuthNN, BehavioralAnomalyDetector, EnsembleModel
from .inference import BehavioralInferenceEngine
from .preprocessing import FeaturePreprocessor, TemporalFeatureExtractor

__version__ = "1.0.0"

__all__ = [
    'BehavioralAuthNN',
    'BehavioralAnomalyDetector',
    'EnsembleModel',
    'BehavioralInferenceEngine',
    'FeaturePreprocessor',
    'TemporalFeatureExtractor',
]