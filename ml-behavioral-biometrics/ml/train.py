"""
Training pipeline for behavioral biometrics model
"""

import torch
import torch.nn as nn
import torch.optim as optim
from torch.utils.data import DataLoader
from typing import Dict, Tuple
import time

from ml.utils import (
    save_model_checkpoint,
    calculate_metrics,
    format_time,
)


class Trainer:
    """
    Training pipeline for behavioral authentication models
    """
    
    def __init__(
        self,
        model: nn.Module,
        train_loader: DataLoader,
        val_loader: DataLoader,
        learning_rate: float = 0.001,
        device: str = "cpu",
        checkpoint_dir: str = "models/checkpoints",
    ):
        """
        Initialize trainer
        
        Args:
            model: PyTorch model
            train_loader: Training data loader
            val_loader: Validation data loader
            learning_rate: Initial learning rate
            device: Device to train on
            checkpoint_dir: Directory to save checkpoints
        """
        self.model = model.to(device)
        self.train_loader = train_loader
        self.val_loader = val_loader
        self.device = device
        self.checkpoint_dir = checkpoint_dir
        
        # Loss function with label smoothing
        self.criterion = nn.BCELoss()
        
        # Optimizer with weight decay
        self.optimizer = optim.AdamW(
            model.parameters(),
            lr=learning_rate,
            weight_decay=0.01,
        )
        
        # Learning rate scheduler
        self.scheduler = optim.lr_scheduler.ReduceLROnPlateau(
            self.optimizer,
            mode='min',
            factor=0.5,
            patience=5,
            verbose=True,
        )
        
        # Training state
        self.history = {
            'train_loss': [],
            'val_loss': [],
            'val_accuracy': [],
            'val_f1': [],
            'learning_rate': [],
        }
        
        self.best_val_loss = float('inf')
        self.patience_counter = 0
        self.early_stop_patience = 10
    
    def train_epoch(self) -> float:
        """
        Train for one epoch
        
        Returns:
            Average training loss
        """
        self.model.train()
        total_loss = 0.0
        num_batches = 0
        
        for batch_X, batch_y in self.train_loader:
            batch_X = batch_X.to(self.device)
            batch_y = batch_y.to(self.device).unsqueeze(1)
            
            # Forward pass
            self.optimizer.zero_grad()
            outputs = self.model(batch_X)
            loss = self.criterion(outputs, batch_y)
            
            # Backward pass
            loss.backward()
            
            # Gradient clipping
            torch.nn.utils.clip_grad_norm_(self.model.parameters(), max_norm=1.0)
            
            self.optimizer.step()
            
            total_loss += loss.item()
            num_batches += 1
        
        return total_loss / num_batches
    
    def validate(self) -> Tuple[float, Dict[str, float]]:
        """
        Validate model
        
        Returns:
            Tuple of (validation loss, metrics dict)
        """
        self.model.eval()
        total_loss = 0.0
        num_batches = 0
        
        all_predictions = []
        all_labels = []
        
        with torch.no_grad():
            for batch_X, batch_y in self.val_loader:
                batch_X = batch_X.to(self.device)
                batch_y = batch_y.to(self.device).unsqueeze(1)
                
                outputs = self.model(batch_X)
                loss = self.criterion(outputs, batch_y)
                
                total_loss += loss.item()
                num_batches += 1
                
                # Collect predictions
                all_predictions.extend(outputs.cpu().numpy())
                all_labels.extend(batch_y.cpu().numpy())
        
        avg_loss = total_loss / num_batches
        
        # Calculate metrics
        import numpy as np
        y_true = np.array(all_labels).flatten()
        y_pred = np.array(all_predictions).flatten()
        
        metrics = calculate_metrics(y_true, y_pred)
        
        return avg_loss, metrics
    
    def train(self, epochs: int, save_path: str) -> Dict:
        """
        Train model for specified epochs
        
        Args:
            epochs: Number of epochs
            save_path: Path to save best model
            
        Returns:
            Training history
        """
        print("=" * 80)
        print("STARTING TRAINING")
        print("=" * 80)
        
        start_time = time.time()
        
        for epoch in range(epochs):
            epoch_start = time.time()
            
            # Training
            train_loss = self.train_epoch()
            
            # Validation
            val_loss, val_metrics = self.validate()
            
            # Update history
            self.history['train_loss'].append(train_loss)
            self.history['val_loss'].append(val_loss)
            self.history['val_accuracy'].append(val_metrics['accuracy'])
            self.history['val_f1'].append(val_metrics['f1'])
            self.history['learning_rate'].append(
                self.optimizer.param_groups[0]['lr']
            )
            
            # Learning rate scheduling
            self.scheduler.step(val_loss)
            
            # Print progress
            epoch_time = time.time() - epoch_start
            print(f"\nEpoch [{epoch+1}/{epochs}] ({format_time(epoch_time)})")
            print(f"  Train Loss: {train_loss:.4f}")
            print(f"  Val Loss:   {val_loss:.4f}")
            print(f"  Val Acc:    {val_metrics['accuracy']:.2%}")
            print(f"  Val F1:     {val_metrics['f1']:.4f}")
            print(f"  FAR:        {val_metrics['far']:.2%}")
            print(f"  FRR:        {val_metrics['frr']:.2%}")
            print(f"  LR:         {self.optimizer.param_groups[0]['lr']:.6f}")
            
            # Save best model
            if val_loss < self.best_val_loss:
                self.best_val_loss = val_loss
                torch.save(self.model.state_dict(), save_path)
                print(f"  ✓ Saved best model (loss: {val_loss:.4f})")
                self.patience_counter = 0
            else:
                self.patience_counter += 1
            
            # Early stopping
            if self.patience_counter >= self.early_stop_patience:
                print(f"\n Early stopping triggered after {epoch+1} epochs")
                break
        
        total_time = time.time() - start_time
        print("\n" + "=" * 80)
        print(f"TRAINING COMPLETE ({format_time(total_time)})")
        print(f"Best validation loss: {self.best_val_loss:.4f}")
        print("=" * 80)
        
        return self.history