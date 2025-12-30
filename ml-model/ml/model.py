"""
Behavioral Biometrics Authentication Model
PyTorch neural network for behavioral pattern verification
"""

import torch
import torch.nn as nn
import torch.nn.functional as F
from typing import Tuple


class BehavioralAuthNN(nn.Module):
    """
    Neural network for behavioral biometric authentication
    
    Architecture:
        Input (6 features) -> Dense(128) -> Dense(64) -> Dense(32) -> Output (1)
    
    Features:
        - Dropout for regularization
        - Batch normalization for stable training
        - ReLU activations
        - Sigmoid output for confidence score
    """
    
    def __init__(
        self,
        input_dim: int = 6,
        hidden_dims: Tuple[int, int, int] = (128, 64, 32),
        dropout_rate: float = 0.3,
    ):
        super(BehavioralAuthNN, self).__init__()
        
        # Layer 1
        self.fc1 = nn.Linear(input_dim, hidden_dims[0])
        self.bn1 = nn.BatchNorm1d(hidden_dims[0])
        self.dropout1 = nn.Dropout(dropout_rate)
        
        # Layer 2
        self.fc2 = nn.Linear(hidden_dims[0], hidden_dims[1])
        self.bn2 = nn.BatchNorm1d(hidden_dims[1])
        self.dropout2 = nn.Dropout(dropout_rate * 0.7)  # Less dropout in deeper layers
        
        # Layer 3
        self.fc3 = nn.Linear(hidden_dims[1], hidden_dims[2])
        self.bn3 = nn.BatchNorm1d(hidden_dims[2])
        
        # Output layer
        self.fc_out = nn.Linear(hidden_dims[2], 1)
        
    def forward(self, x: torch.Tensor) -> torch.Tensor:
        """
        Forward pass
        
        Args:
            x: Input tensor of shape (batch_size, 6)
            
        Returns:
            Confidence score tensor of shape (batch_size, 1)
        """
        # Layer 1
        x = self.fc1(x)
        x = self.bn1(x)
        x = F.relu(x)
        x = self.dropout1(x)
        
        # Layer 2
        x = self.fc2(x)
        x = self.bn2(x)
        x = F.relu(x)
        x = self.dropout2(x)
        
        # Layer 3
        x = self.fc3(x)
        x = self.bn3(x)
        x = F.relu(x)
        
        # Output (sigmoid for 0-1 probability)
        x = self.fc_out(x)
        x = torch.sigmoid(x)
        
        return x
    
    def predict_confidence(self, x: torch.Tensor) -> torch.Tensor:
        """
        Predict confidence score (0-100)
        
        Args:
            x: Input tensor
            
        Returns:
            Confidence scores scaled to 0-100
        """
        with torch.no_grad():
            logits = self.forward(x)
            confidence = logits * 100  # Scale to 0-100
        return confidence


class BehavioralAnomalyDetector(nn.Module):
    """
    Autoencoder for anomaly detection in behavioral patterns
    
    Detects unusual patterns by reconstruction error
    """
    
    def __init__(
        self,
        input_dim: int = 6,
        encoding_dim: int = 3,
    ):
        super(BehavioralAnomalyDetector, self).__init__()
        
        # Encoder
        self.encoder = nn.Sequential(
            nn.Linear(input_dim, 16),
            nn.ReLU(),
            nn.Linear(16, 8),
            nn.ReLU(),
            nn.Linear(8, encoding_dim),
        )
        
        # Decoder
        self.decoder = nn.Sequential(
            nn.Linear(encoding_dim, 8),
            nn.ReLU(),
            nn.Linear(8, 16),
            nn.ReLU(),
            nn.Linear(16, input_dim),
        )
        
    def forward(self, x: torch.Tensor) -> Tuple[torch.Tensor, torch.Tensor]:
        """
        Forward pass with reconstruction
        
        Returns:
            Tuple of (encoded, reconstructed)
        """
        encoded = self.encoder(x)
        reconstructed = self.decoder(encoded)
        return encoded, reconstructed
    
    def anomaly_score(self, x: torch.Tensor) -> torch.Tensor:
        """
        Calculate anomaly score based on reconstruction error
        
        Args:
            x: Input tensor
            
        Returns:
            Anomaly scores (higher = more anomalous)
        """
        with torch.no_grad():
            _, reconstructed = self.forward(x)
            mse = F.mse_loss(reconstructed, x, reduction='none')
            anomaly_scores = mse.mean(dim=1)
        return anomaly_scores


class EnsembleModel(nn.Module):
    """
    Ensemble combining authentication and anomaly detection
    """
    
    def __init__(
        self,
        auth_model: BehavioralAuthNN,
        anomaly_model: BehavioralAnomalyDetector,
        anomaly_weight: float = 0.3,
    ):
        super(EnsembleModel, self).__init__()
        self.auth_model = auth_model
        self.anomaly_model = anomaly_model
        self.anomaly_weight = anomaly_weight
        
    def forward(self, x: torch.Tensor) -> Tuple[torch.Tensor, torch.Tensor]:
        """
        Combined forward pass
        
        Returns:
            Tuple of (confidence_score, anomaly_score)
        """
        # Authentication confidence
        confidence = self.auth_model(x)
        
        # Anomaly detection
        anomaly = self.anomaly_model.anomaly_score(x)
        
        # Adjust confidence based on anomaly score
        # High anomaly -> lower confidence
        adjusted_confidence = confidence * (1 - self.anomaly_weight * torch.sigmoid(anomaly).unsqueeze(1))
        
        return adjusted_confidence, anomaly


def create_model(
    model_type: str = "auth",
    pretrained_path: str = None,
    device: str = "cpu",
) -> nn.Module:
    """
    Factory function to create models
    
    Args:
        model_type: Type of model ('auth', 'anomaly', 'ensemble')
        pretrained_path: Path to pretrained weights
        device: Device to load model on
        
    Returns:
        Initialized model
    """
    if model_type == "auth":
        model = BehavioralAuthNN()
    elif model_type == "anomaly":
        model = BehavioralAnomalyDetector()
    elif model_type == "ensemble":
        auth = BehavioralAuthNN()
        anomaly = BehavioralAnomalyDetector()
        model = EnsembleModel(auth, anomaly)
    else:
        raise ValueError(f"Unknown model type: {model_type}")
    
    if pretrained_path:
        model.load_state_dict(torch.load(pretrained_path, map_location=device))
    
    model = model.to(device)
    model.eval()
    
    return model