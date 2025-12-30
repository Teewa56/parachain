"""
Utility functions for ML pipeline
"""

import torch
import numpy as np
from pathlib import Path
import json
import hashlib
from typing import Dict, Any, List


def set_seed(seed: int = 42):
    """
    Set random seed for reproducibility
    
    Args:
        seed: Random seed value
    """
    torch.manual_seed(seed)
    torch.cuda.manual_seed_all(seed)
    np.random.seed(seed)


def count_parameters(model: torch.nn.Module) -> int:
    """
    Count trainable parameters in model
    
    Args:
        model: PyTorch model
        
    Returns:
        Number of trainable parameters
    """
    return sum(p.numel() for p in model.parameters() if p.requires_grad)


def save_model_checkpoint(
    model: torch.nn.Module,
    optimizer: torch.optim.Optimizer,
    epoch: int,
    metrics: Dict[str, float],
    save_path: str,
):
    """
    Save model checkpoint with metadata
    
    Args:
        model: PyTorch model
        optimizer: Optimizer
        epoch: Current epoch
        metrics: Training metrics
        save_path: Path to save checkpoint
    """
    checkpoint = {
        'epoch': epoch,
        'model_state_dict': model.state_dict(),
        'optimizer_state_dict': optimizer.state_dict(),
        'metrics': metrics,
    }
    
    torch.save(checkpoint, save_path)
    print(f"Checkpoint saved to {save_path}")


def load_model_checkpoint(
    model: torch.nn.Module,
    checkpoint_path: str,
    device: str = "cpu",
) -> Dict[str, Any]:
    """
    Load model checkpoint
    
    Args:
        model: PyTorch model
        checkpoint_path: Path to checkpoint
        device: Device to load on
        
    Returns:
        Checkpoint metadata
    """
    checkpoint = torch.load(checkpoint_path, map_location=device)
    model.load_state_dict(checkpoint['model_state_dict'])
    
    return checkpoint


def calculate_model_hash(model_path: str) -> str:
    """
    Calculate SHA256 hash of model file for integrity verification
    
    Args:
        model_path: Path to model file
        
    Returns:
        Hex string of SHA256 hash
    """
    sha256_hash = hashlib.sha256()
    
    with open(model_path, "rb") as f:
        for byte_block in iter(lambda: f.read(4096), b""):
            sha256_hash.update(byte_block)
    
    return sha256_hash.hexdigest()


def save_training_config(config: Dict[str, Any], output_path: str):
    """
    Save training configuration to JSON
    
    Args:
        config: Configuration dictionary
        output_path: Path to save config
    """
    with open(output_path, 'w') as f:
        json.dump(config, f, indent=2)


def calculate_class_weights(y: np.ndarray) -> torch.Tensor:
    """
    Calculate class weights for imbalanced datasets
    
    Args:
        y: Label array
        
    Returns:
        Tensor of class weights
    """
    unique, counts = np.unique(y, return_counts=True)
    total = len(y)
    
    weights = total / (len(unique) * counts)
    
    return torch.tensor(weights, dtype=torch.float32)


def format_time(seconds: float) -> str:
    """
    Format seconds into human-readable time
    
    Args:
        seconds: Time in seconds
        
    Returns:
        Formatted time string
    """
    if seconds < 60:
        return f"{seconds:.2f}s"
    elif seconds < 3600:
        minutes = seconds / 60
        return f"{minutes:.2f}m"
    else:
        hours = seconds / 3600
        return f"{hours:.2f}h"


def calculate_metrics(
    y_true: np.ndarray,
    y_pred: np.ndarray,
    threshold: float = 0.5,
) -> Dict[str, float]:
    """
    Calculate classification metrics
    
    Args:
        y_true: True labels
        y_pred: Predicted probabilities
        threshold: Classification threshold
        
    Returns:
        Dictionary of metrics
    """
    from sklearn.metrics import (
        accuracy_score,
        precision_score,
        recall_score,
        f1_score,
        roc_auc_score,
    )
    
    # Convert probabilities to binary predictions
    y_pred_binary = (y_pred >= threshold).astype(int)
    
    metrics = {
        'accuracy': accuracy_score(y_true, y_pred_binary),
        'precision': precision_score(y_true, y_pred_binary),
        'recall': recall_score(y_true, y_pred_binary),
        'f1': f1_score(y_true, y_pred_binary),
        'auc_roc': roc_auc_score(y_true, y_pred),
        'far': calculate_far(y_true, y_pred_binary),
        'frr': calculate_frr(y_true, y_pred_binary),
    }
    
    return metrics


def calculate_far(y_true: np.ndarray, y_pred: np.ndarray) -> float:
    """
    Calculate False Acceptance Rate (FAR)
    
    FAR = False Positives / (False Positives + True Negatives)
    
    Args:
        y_true: True labels
        y_pred: Predicted labels
        
    Returns:
        FAR value
    """
    # True label = 0 (impostor), Predicted = 1 (accept)
    false_positives = np.sum((y_true == 0) & (y_pred == 1))
    true_negatives = np.sum((y_true == 0) & (y_pred == 0))
    
    if (false_positives + true_negatives) == 0:
        return 0.0
    
    return false_positives / (false_positives + true_negatives)


def calculate_frr(y_true: np.ndarray, y_pred: np.ndarray) -> float:
    """
    Calculate False Rejection Rate (FRR)
    
    FRR = False Negatives / (False Negatives + True Positives)
    
    Args:
        y_true: True labels
        y_pred: Predicted labels
        
    Returns:
        FRR value
    """
    # True label = 1 (legitimate), Predicted = 0 (reject)
    false_negatives = np.sum((y_true == 1) & (y_pred == 0))
    true_positives = np.sum((y_true == 1) & (y_pred == 1))
    
    if (false_negatives + true_positives) == 0:
        return 0.0
    
    return false_negatives / (false_negatives + true_positives)


def print_model_summary(model: torch.nn.Module):
    """
    Print model architecture summary
    
    Args:
        model: PyTorch model
    """
    print("=" * 80)
    print("MODEL ARCHITECTURE")
    print("=" * 80)
    print(model)
    print("=" * 80)
    print(f"Total Parameters: {count_parameters(model):,}")
    print("=" * 80)