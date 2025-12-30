"""
Inference engine for behavioral biometric authentication
"""

import torch
import numpy as np
import pickle
from pathlib import Path
from typing import Dict, List, Optional, Any
import time

from ml.model import BehavioralAuthNN, BehavioralAnomalyDetector
from ml.preprocessing import FeaturePreprocessor


class BehavioralInferenceEngine:
    """
    Production inference engine for behavioral authentication
    
    Features:
        - Feature preprocessing and normalization
        - Model inference with confidence scoring
        - Anomaly detection
        - Feature importance calculation
        - Performance monitoring
    """
    
    def __init__(
        self,
        model_path: str,
        scaler_path: Optional[str] = None,
        device: str = "cpu",
    ):
        """
        Initialize inference engine
        
        Args:
            model_path: Path to trained PyTorch model
            scaler_path: Path to fitted StandardScaler
            device: Device to run inference on ('cpu' or 'cuda')
        """
        self.device = torch.device(device)
        
        # Load model
        self.model = BehavioralAuthNN()
        self.model.load_state_dict(
            torch.load(model_path, map_location=self.device)
        )
        self.model.to(self.device)
        self.model.eval()
        
        # Load scaler
        if scaler_path and Path(scaler_path).exists():
            with open(scaler_path, 'rb') as f:
                self.scaler = pickle.load(f)
        else:
            self.scaler = None
        
        # Preprocessor
        self.preprocessor = FeaturePreprocessor()
        
        # Anomaly detector (optional)
        self.anomaly_detector = None
        
        # Performance tracking
        self.stats = {
            'total_predictions': 0,
            'total_inference_time': 0.0,
            'confidence_scores': [],
        }
    
    def predict(
        self,
        features: Dict[str, Any],
        historical_patterns: Optional[List[Dict[str, Any]]] = None,
    ) -> Dict[str, Any]:
        """
        Predict confidence score for behavioral pattern
        
        Args:
            features: Dictionary containing behavioral features
            historical_patterns: Optional historical patterns for comparison
            
        Returns:
            Dictionary with prediction results
        """
        start_time = time.time()
        
        # Preprocess features
        processed_features = self.preprocessor.process_features(features)
        
        # Add derived features
        processed_features = self._add_derived_features(processed_features)
        
        # Convert to tensor
        feature_tensor = torch.tensor(
            processed_features,
            dtype=torch.float32,
            device=self.device
        ).unsqueeze(0)  # Add batch dimension
        
        # Normalize if scaler available
        if self.scaler is not None:
            feature_array = feature_tensor.cpu().numpy()
            feature_array = self.scaler.transform(feature_array)
            feature_tensor = torch.tensor(
                feature_array,
                dtype=torch.float32,
                device=self.device
            )
        
        # Model inference
        with torch.no_grad():
            confidence_logit = self.model(feature_tensor)
            confidence_score = int((confidence_logit * 100).item())
        
        # Anomaly detection
        anomaly_score = 0.0
        if self.anomaly_detector is not None:
            anomaly_score = self._calculate_anomaly_score(feature_tensor)
        
        # Calculate feature importance
        feature_importance = self._calculate_feature_importance(feature_tensor)
        
        # Historical comparison
        if historical_patterns:
            historical_score = self._compare_with_historical(
                processed_features,
                historical_patterns
            )
            # Adjust confidence based on historical consistency
            confidence_score = int(confidence_score * 0.7 + historical_score * 0.3)
        
        # Update stats
        inference_time = time.time() - start_time
        self._update_stats(confidence_score, inference_time)
        
        return {
            'confidence_score': confidence_score,
            'anomaly_score': anomaly_score,
            'feature_importance': feature_importance,
        }
    
    def _add_derived_features(self, features: List[float]) -> List[float]:
        """
        Add derived features for better discrimination
        
        Features:
            - Speed-accuracy ratio
            - Rhythm ratio
        """
        typing_speed = features[0]
        key_hold_time = features[1]
        transition_time = features[2]
        error_rate = features[3]
        
        # Speed-accuracy ratio: typing_speed / (error_rate + 1)
        speed_accuracy = typing_speed / (error_rate + 1)
        
        # Rhythm ratio: key_hold_time / transition_time
        rhythm_ratio = key_hold_time / (transition_time + 1)
        
        # Return extended features
        return features + [speed_accuracy, rhythm_ratio]
    
    def _calculate_anomaly_score(self, feature_tensor: torch.Tensor) -> float:
        """Calculate anomaly score using autoencoder"""
        if self.anomaly_detector is None:
            return 0.0
        
        with torch.no_grad():
            anomaly = self.anomaly_detector.anomaly_score(feature_tensor)
            return float(anomaly.item())
    
    def _calculate_feature_importance(
        self,
        feature_tensor: torch.Tensor
    ) -> Dict[str, float]:
        """
        Calculate feature importance using gradients
        
        Returns:
            Dictionary mapping feature names to importance scores
        """
        feature_names = [
            'typing_speed',
            'key_hold_time',
            'transition_time',
            'error_rate',
            'activity_hour',
            'speed_accuracy_ratio',
            'rhythm_ratio',
        ]
        
        # Enable gradient tracking
        feature_tensor.requires_grad_(True)
        
        # Forward pass
        output = self.model(feature_tensor)
        
        # Backward pass
        output.backward()
        
        # Get gradients (importance)
        gradients = feature_tensor.grad.abs().squeeze().cpu().numpy()
        
        # Normalize to sum to 1
        total = gradients.sum()
        if total > 0:
            gradients = gradients / total
        
        # Create importance dict
        importance = {
            name: float(grad)
            for name, grad in zip(feature_names, gradients)
        }
        
        return importance
    
    def _compare_with_historical(
        self,
        current_features: List[float],
        historical_patterns: List[Dict[str, Any]]
    ) -> float:
        """
        Compare current pattern with historical patterns
        
        Returns:
            Historical consistency score (0-100)
        """
        if not historical_patterns:
            return 50  # Neutral score
        
        # Process historical features
        historical_vectors = []
        for pattern in historical_patterns:
            processed = self.preprocessor.process_features(pattern)
            processed = self._add_derived_features(processed)
            historical_vectors.append(np.array(processed))
        
        # Calculate mean historical pattern
        mean_historical = np.mean(historical_vectors, axis=0)
        
        # Calculate Euclidean distance
        current_array = np.array(current_features)
        distance = np.linalg.norm(current_array - mean_historical)
        
        # Convert distance to similarity score (0-100)
        # Lower distance = higher similarity
        max_distance = 100  # Assume max distance
        similarity = max(0, 100 - (distance / max_distance) * 100)
        
        return similarity
    
    def _update_stats(self, confidence: int, inference_time: float):
        """Update performance statistics"""
        self.stats['total_predictions'] += 1
        self.stats['total_inference_time'] += inference_time
        self.stats['confidence_scores'].append(confidence)
        
        # Keep only last 1000 scores
        if len(self.stats['confidence_scores']) > 1000:
            self.stats['confidence_scores'].pop(0)
    
    def get_stats(self) -> Dict[str, Any]:
        """Get performance statistics"""
        total = self.stats['total_predictions']
        if total == 0:
            return {
                'total_predictions': 0,
                'avg_inference_time_ms': 0.0,
                'avg_confidence': 0.0,
            }
        
        avg_time = (self.stats['total_inference_time'] / total) * 1000  # Convert to ms
        avg_confidence = np.mean(self.stats['confidence_scores'])
        
        return {
            'total_predictions': total,
            'avg_inference_time_ms': round(avg_time, 2),
            'avg_confidence': round(avg_confidence, 2),
        }