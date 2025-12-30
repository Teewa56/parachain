"""
Training script for behavioral biometrics model
"""

import argparse
import pandas as pd
import numpy as np
import torch
import torch.nn as nn
import torch.optim as optim
from torch.utils.data import DataLoader, TensorDataset
from sklearn.model_selection import train_test_split
from sklearn.preprocessing import StandardScaler
import pickle
from pathlib import Path
import json

import sys
sys.path.append(str(Path(__file__).parent.parent))

from ml.model import BehavioralAuthNN
from ml.preprocessing import FeaturePreprocessor, augment_data


def load_data(data_path: str) -> tuple:
    """
    Load training data from CSV
    
    CSV format:
        typing_speed_wpm, avg_key_hold_time_ms, avg_transition_time_ms,
        error_rate_percent, activity_hour_preference, is_legitimate
    """
    df = pd.read_csv(data_path)
    
    # Extract features
    feature_cols = [
        'typing_speed_wpm',
        'avg_key_hold_time_ms',
        'avg_transition_time_ms',
        'error_rate_percent',
        'activity_hour_preference',
    ]
    
    X = df[feature_cols].values
    y = df['is_legitimate'].values
    
    return X, y


def add_derived_features(X: np.ndarray) -> np.ndarray:
    """Add derived features"""
    # Speed-accuracy ratio
    speed_accuracy = X[:, 0] / (X[:, 3] + 1)
    
    # Rhythm ratio
    rhythm_ratio = X[:, 1] / (X[:, 2] + 1)
    
    # Concatenate
    X_extended = np.column_stack([X, speed_accuracy, rhythm_ratio])
    
    return X_extended


def train_model(
    model: nn.Module,
    train_loader: DataLoader,
    val_loader: DataLoader,
    epochs: int,
    learning_rate: float,
    device: torch.device,
    save_path: str,
) -> dict:
    """
    Train the model
    
    Returns:
        Training history dictionary
    """
    criterion = nn.BCELoss()  # Binary cross-entropy
    optimizer = optim.AdamW(
        model.parameters(),
        lr=learning_rate,
        weight_decay=0.01,
    )
    
    # Learning rate scheduler
    scheduler = optim.lr_scheduler.ReduceLROnPlateau(
        optimizer,
        mode='min',
        factor=0.5,
        patience=5,
        verbose=True,
    )
    
    # Training history
    history = {
        'train_loss': [],
        'val_loss': [],
        'val_accuracy': [],
    }
    
    best_val_loss = float('inf')
    patience_counter = 0
    early_stop_patience = 10
    
    for epoch in range(epochs):
        # Training phase
        model.train()
        train_loss = 0.0
        
        for batch_X, batch_y in train_loader:
            batch_X = batch_X.to(device)
            batch_y = batch_y.to(device).unsqueeze(1)
            
            # Forward pass
            optimizer.zero_grad()
            outputs = model(batch_X)
            loss = criterion(outputs, batch_y)
            
            # Backward pass
            loss.backward()
            optimizer.step()
            
            train_loss += loss.item()
        
        avg_train_loss = train_loss / len(train_loader)
        
        # Validation phase
        model.eval()
        val_loss = 0.0
        correct = 0
        total = 0
        
        with torch.no_grad():
            for batch_X, batch_y in val_loader:
                batch_X = batch_X.to(device)
                batch_y = batch_y.to(device).unsqueeze(1)
                
                outputs = model(batch_X)
                loss = criterion(outputs, batch_y)
                val_loss += loss.item()
                
                # Calculate accuracy
                predicted = (outputs > 0.5).float()
                total += batch_y.size(0)
                correct += (predicted == batch_y).sum().item()
        
        avg_val_loss = val_loss / len(val_loader)
        val_accuracy = 100 * correct / total
        
        # Update history
        history['train_loss'].append(avg_train_loss)
        history['val_loss'].append(avg_val_loss)
        history['val_accuracy'].append(val_accuracy)
        
        # Learning rate scheduling
        scheduler.step(avg_val_loss)
        
        # Print progress
        print(f"Epoch [{epoch+1}/{epochs}]")
        print(f"  Train Loss: {avg_train_loss:.4f}")
        print(f"  Val Loss: {avg_val_loss:.4f}")
        print(f"  Val Accuracy: {val_accuracy:.2f}%")
        
        # Save best model
        if avg_val_loss < best_val_loss:
            best_val_loss = avg_val_loss
            torch.save(model.state_dict(), save_path)
            print(f"  ✓ Saved best model")
            patience_counter = 0
        else:
            patience_counter += 1
        
        # Early stopping
        if patience_counter >= early_stop_patience:
            print(f"\nEarly stopping triggered after {epoch+1} epochs")
            break
    
    return history


def main():
    parser = argparse.ArgumentParser(description='Train behavioral biometrics model')
    parser.add_argument('--data', type=str, required=True, help='Path to training data CSV')
    parser.add_argument('--epochs', type=int, default=100, help='Number of epochs')
    parser.add_argument('--batch-size', type=int, default=256, help='Batch size')
    parser.add_argument('--learning-rate', type=float, default=0.001, help='Learning rate')
    parser.add_argument('--device', type=str, default='cpu', help='Device (cpu/cuda)')
    parser.add_argument('--output-dir', type=str, default='models/production', help='Output directory')
    parser.add_argument('--augment', action='store_true', help='Apply data augmentation')
    
    args = parser.parse_args()
    
    # Set device
    device = torch.device(args.device)
    print(f"Using device: {device}")
    
    # Load data
    print("Loading data...")
    X, y = load_data(args.data)
    print(f"Loaded {len(X)} samples")
    
    # Add derived features
    X = add_derived_features(X)
    
    # Augment data
    if args.augment:
        print("Applying data augmentation...")
        X, y = augment_data(X, y, noise_level=0.05, augmentation_factor=2)
        print(f"Augmented to {len(X)} samples")
    
    # Train/test split
    X_train, X_test, y_train, y_test = train_test_split(
        X, y, test_size=0.2, random_state=42, stratify=y
    )
    
    # Normalize features
    scaler = StandardScaler()
    X_train = scaler.fit_transform(X_train)
    X_test = scaler.transform(X_test)
    
    # Convert to PyTorch tensors
    X_train_tensor = torch.tensor(X_train, dtype=torch.float32)
    y_train_tensor = torch.tensor(y_train, dtype=torch.float32)
    X_test_tensor = torch.tensor(X_test, dtype=torch.float32)
    y_test_tensor = torch.tensor(y_test, dtype=torch.float32)
    
    # Create data loaders
    train_dataset = TensorDataset(X_train_tensor, y_train_tensor)
    test_dataset = TensorDataset(X_test_tensor, y_test_tensor)
    
    train_loader = DataLoader(
        train_dataset,
        batch_size=args.batch_size,
        shuffle=True,
    )
    test_loader = DataLoader(
        test_dataset,
        batch_size=args.batch_size,
        shuffle=False,
    )
    
    # Create model
    print("Creating model...")
    model = BehavioralAuthNN(input_dim=7)  # 5 base + 2 derived features
    model = model.to(device)
    
    # Count parameters
    total_params = sum(p.numel() for p in model.parameters())
    print(f"Total parameters: {total_params:,}")
    
    # Create output directory
    output_dir = Path(args.output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)
    
    model_path = output_dir / 'model.pth'
    scaler_path = output_dir / 'scaler.pkl'
    history_path = output_dir / 'training_history.json'
    
    # Train model
    print("\nStarting training...")
    history = train_model(
        model=model,
        train_loader=train_loader,
        val_loader=test_loader,
        epochs=args.epochs,
        learning_rate=args.learning_rate,
        device=device,
        save_path=str(model_path),
    )
    
    # Save scaler
    with open(scaler_path, 'wb') as f:
        pickle.dump(scaler, f)
    print(f"\nSaved scaler to {scaler_path}")
    
    # Save training history
    with open(history_path, 'w') as f:
        json.dump(history, f, indent=2)
    print(f"Saved training history to {history_path}")
    
    print("\nTraining complete!")
    print(f"Best model saved to: {model_path}")


if __name__ == '__main__':
    main()