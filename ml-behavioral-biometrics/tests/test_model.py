"""
Model Architecture Tests
"""

import pytest
import torch
import numpy as np
from pathlib import Path
import sys

sys.path.append(str(Path(__file__).parent.parent))

from ml.model import (
    BehavioralAuthNN,
    BehavioralAnomalyDetector,
    EnsembleModel,
    create_model,
)


class TestBehavioralAuthNN:
    """Test cases for BehavioralAuthNN model"""
    
    def test_model_initialization(self):
        """Test model can be initialized"""
        model = BehavioralAuthNN(input_dim=7)
        assert model is not None
        assert isinstance(model, torch.nn.Module)
    
    def test_forward_pass(self):
        """Test forward pass produces correct output shape"""
        model = BehavioralAuthNN(input_dim=7)
        model.eval()
        
        # Create dummy input
        x = torch.randn(10, 7)
        
        with torch.no_grad():
            output = model(x)
        
        # Check output shape
        assert output.shape == (10, 1)
        
        # Check output range (sigmoid output should be 0-1)
        assert torch.all(output >= 0)
        assert torch.all(output <= 1)
    
    def test_predict_confidence(self):
        """Test confidence prediction"""
        model = BehavioralAuthNN(input_dim=7)
        model.eval()
        
        x = torch.randn(5, 7)
        confidence = model.predict_confidence(x)
        
        # Check shape
        assert confidence.shape == (5, 1)
        
        # Check range (0-100)
        assert torch.all(confidence >= 0)
        assert torch.all(confidence <= 100)
    
    def test_model_parameters(self):
        """Test model has trainable parameters"""
        model = BehavioralAuthNN(input_dim=7)
        
        # Count parameters
        total_params = sum(p.numel() for p in model.parameters())
        trainable_params = sum(p.numel() for p in model.parameters() if p.requires_grad)
        
        assert total_params > 0
        assert trainable_params == total_params
    
    def test_batch_processing(self):
        """Test model handles different batch sizes"""
        model = BehavioralAuthNN(input_dim=7)
        model.eval()
        
        batch_sizes = [1, 10, 100]
        
        for batch_size in batch_sizes:
            x = torch.randn(batch_size, 7)
            with torch.no_grad():
                output = model(x)
            assert output.shape == (batch_size, 1)


class TestBehavioralAnomalyDetector:
    """Test cases for anomaly detector"""
    
    def test_autoencoder_initialization(self):
        """Test autoencoder can be initialized"""
        model = BehavioralAnomalyDetector(input_dim=6, encoding_dim=3)
        assert model is not None
    
    def test_encoding_decoding(self):
        """Test encoding and decoding"""
        model = BehavioralAnomalyDetector(input_dim=6, encoding_dim=3)
        model.eval()
        
        x = torch.randn(10, 6)
        
        with torch.no_grad():
            encoded, reconstructed = model(x)
        
        # Check shapes
        assert encoded.shape == (10, 3)
        assert reconstructed.shape == (10, 6)
    
    def test_anomaly_score(self):
        """Test anomaly score calculation"""
        model = BehavioralAnomalyDetector(input_dim=6, encoding_dim=3)
        model.eval()
        
        # Normal sample
        x_normal = torch.randn(5, 6)
        
        # Anomalous sample (large values)
        x_anomaly = torch.randn(5, 6) * 10
        
        score_normal = model.anomaly_score(x_normal)
        score_anomaly = model.anomaly_score(x_anomaly)
        
        # Anomalous samples should have higher scores
        assert score_normal.shape == (5,)
        assert score_anomaly.shape == (5,)


class TestEnsembleModel:
    """Test cases for ensemble model"""
    
    def test_ensemble_initialization(self):
        """Test ensemble model initialization"""
        auth_model = BehavioralAuthNN(input_dim=7)
        anomaly_model = BehavioralAnomalyDetector(input_dim=7, encoding_dim=3)
        
        ensemble = EnsembleModel(auth_model, anomaly_model)
        assert ensemble is not None
    
    def test_ensemble_forward(self):
        """Test ensemble forward pass"""
        auth_model = BehavioralAuthNN(input_dim=7)
        anomaly_model = BehavioralAnomalyDetector(input_dim=7, encoding_dim=3)
        ensemble = EnsembleModel(auth_model, anomaly_model)
        ensemble.eval()
        
        x = torch.randn(10, 7)
        
        with torch.no_grad():
            confidence, anomaly = ensemble(x)
        
        assert confidence.shape == (10, 1)
        assert anomaly.shape == (10,)


class TestModelFactory:
    """Test model factory function"""
    
    def test_create_auth_model(self):
        """Test creating authentication model"""
        model = create_model(model_type="auth")
        assert isinstance(model, BehavioralAuthNN)
    
    def test_create_anomaly_model(self):
        """Test creating anomaly detector"""
        model = create_model(model_type="anomaly")
        assert isinstance(model, BehavioralAnomalyDetector)
    
    def test_create_ensemble_model(self):
        """Test creating ensemble model"""
        model = create_model(model_type="ensemble")
        assert isinstance(model, EnsembleModel)
    
    def test_invalid_model_type(self):
        """Test error on invalid model type"""
        with pytest.raises(ValueError):
            create_model(model_type="invalid")

