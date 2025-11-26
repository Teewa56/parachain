"""
Model evaluation script with detailed metrics
"""

import argparse
import pandas as pd
import numpy as np
import torch
from pathlib import Path
import json
import matplotlib.pyplot as plt
import seaborn as sns
from sklearn.metrics import (
    confusion_matrix,
    classification_report,
    roc_curve,
    auc,
)

import sys
sys.path.append(str(Path(__file__).parent.parent))

from ml.model import BehavioralAuthNN
from ml.preprocessing import FeaturePreprocessor
from ml.utils import calculate_metrics, calculate_far, calculate_frr


def load_test_data(data_path: str):
    """Load test data from CSV"""
    df = pd.read_csv(data_path)
    
    feature_cols = [
        'typing_speed_wpm',
        'avg_key_hold_time_ms',
        'avg_transition_time_ms',
        'error_rate_percent',
        'activity_hour_preference',
    ]
    
    X = df[feature_cols].values
    y = df['is_legitimate'].values
    
    # Add derived features
    speed_accuracy = X[:, 0] / (X[:, 3] + 1)
    rhythm_ratio = X[:, 1] / (X[:, 2] + 1)
    X = np.column_stack([X, speed_accuracy, rhythm_ratio])
    
    return X, y


def evaluate_model(
    model: torch.nn.Module,
    X_test: np.ndarray,
    y_test: np.ndarray,
    device: str = "cpu",
) -> dict:
    """
    Comprehensive model evaluation
    
    Returns:
        Dictionary of evaluation metrics
    """
    model.eval()
    
    # Convert to tensor
    X_tensor = torch.tensor(X_test, dtype=torch.float32, device=device)
    
    # Get predictions
    with torch.no_grad():
        y_pred_proba = model(X_tensor).cpu().numpy().flatten()
    
    # Calculate metrics at different thresholds
    thresholds = [0.3, 0.4, 0.5, 0.6, 0.7]
    threshold_metrics = {}
    
    for threshold in thresholds:
        y_pred = (y_pred_proba >= threshold).astype(int)
        metrics = calculate_metrics(y_test, y_pred_proba, threshold)
        threshold_metrics[threshold] = metrics
    
    # Find optimal threshold (minimize FAR + FRR)
    best_threshold = min(
        threshold_metrics.keys(),
        key=lambda t: threshold_metrics[t]['far'] + threshold_metrics[t]['frr']
    )
    
    # Use best threshold for final predictions
    y_pred_best = (y_pred_proba >= best_threshold).astype(int)
    
    # Confusion matrix
    cm = confusion_matrix(y_test, y_pred_best)
    
    # ROC curve
    fpr, tpr, roc_thresholds = roc_curve(y_test, y_pred_proba)
    roc_auc = auc(fpr, tpr)
    
    # Classification report
    report = classification_report(
        y_test,
        y_pred_best,
        target_names=['Impostor', 'Legitimate'],
        output_dict=True,
    )
    
    return {
        'best_threshold': best_threshold,
        'threshold_metrics': threshold_metrics,
        'confusion_matrix': cm.tolist(),
        'roc_auc': roc_auc,
        'fpr': fpr.tolist(),
        'tpr': tpr.tolist(),
        'classification_report': report,
        'predictions': y_pred_proba.tolist(),
        'labels': y_test.tolist(),
    }


def plot_results(results: dict, output_dir: str):
    """Generate evaluation plots"""
    output_dir = Path(output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)
    
    # 1. ROC Curve
    plt.figure(figsize=(10, 8))
    plt.plot(
        results['fpr'],
        results['tpr'],
        color='darkorange',
        lw=2,
        label=f"ROC curve (AUC = {results['roc_auc']:.3f})"
    )
    plt.plot([0, 1], [0, 1], color='navy', lw=2, linestyle='--', label='Random')
    plt.xlim([0.0, 1.0])
    plt.ylim([0.0, 1.05])
    plt.xlabel('False Positive Rate (FAR)')
    plt.ylabel('True Positive Rate (1 - FRR)')
    plt.title('ROC Curve - Behavioral Biometrics Authentication')
    plt.legend(loc="lower right")
    plt.grid(alpha=0.3)
    plt.savefig(output_dir / 'roc_curve.png', dpi=300, bbox_inches='tight')
    plt.close()
    
    # 2. Confusion Matrix
    cm = np.array(results['confusion_matrix'])
    plt.figure(figsize=(8, 6))
    sns.heatmap(
        cm,
        annot=True,
        fmt='d',
        cmap='Blues',
        xticklabels=['Impostor', 'Legitimate'],
        yticklabels=['Impostor', 'Legitimate']
    )
    plt.title('Confusion Matrix')
    plt.ylabel('True Label')
    plt.xlabel('Predicted Label')
    plt.savefig(output_dir / 'confusion_matrix.png', dpi=300, bbox_inches='tight')
    plt.close()
    
    # 3. Threshold Analysis
    thresholds = list(results['threshold_metrics'].keys())
    fars = [results['threshold_metrics'][t]['far'] for t in thresholds]
    frrs = [results['threshold_metrics'][t]['frr'] for t in thresholds]
    
    plt.figure(figsize=(10, 6))
    plt.plot(thresholds, fars, 'o-', label='FAR', linewidth=2)
    plt.plot(thresholds, frrs, 's-', label='FRR', linewidth=2)
    plt.axvline(
        results['best_threshold'],
        color='red',
        linestyle='--',
        label=f"Optimal Threshold: {results['best_threshold']}"
    )
    plt.xlabel('Threshold')
    plt.ylabel('Error Rate')
    plt.title('FAR vs FRR at Different Thresholds')
    plt.legend()
    plt.grid(alpha=0.3)
    plt.savefig(output_dir / 'threshold_analysis.png', dpi=300, bbox_inches='tight')
    plt.close()
    
    # 4. Score Distribution
    predictions = np.array(results['predictions'])
    labels = np.array(results['labels'])
    
    legitimate_scores = predictions[labels == 1]
    impostor_scores = predictions[labels == 0]
    
    plt.figure(figsize=(10, 6))
    plt.hist(legitimate_scores, bins=50, alpha=0.7, label='Legitimate', color='green')
    plt.hist(impostor_scores, bins=50, alpha=0.7, label='Impostor', color='red')
    plt.axvline(
        results['best_threshold'],
        color='black',
        linestyle='--',
        label=f"Threshold: {results['best_threshold']}"
    )
    plt.xlabel('Confidence Score')
    plt.ylabel('Frequency')
    plt.title('Score Distribution')
    plt.legend()
    plt.grid(alpha=0.3)
    plt.savefig(output_dir / 'score_distribution.png', dpi=300, bbox_inches='tight')
    plt.close()
    
    print(f"\n Plots saved to {output_dir}/")


def print_evaluation_summary(results: dict):
    """Print evaluation summary"""
    print("\n" + "=" * 80)
    print("MODEL EVALUATION SUMMARY")
    print("=" * 80)
    
    best_metrics = results['threshold_metrics'][results['best_threshold']]
    
    print(f"\nOptimal Threshold: {results['best_threshold']}")
    print(f"\nPerformance Metrics:")
    print(f"  Accuracy:  {best_metrics['accuracy']:.2%}")
    print(f"  Precision: {best_metrics['precision']:.2%}")
    print(f"  Recall:    {best_metrics['recall']:.2%}")
    print(f"  F1 Score:  {best_metrics['f1']:.4f}")
    print(f"  AUC-ROC:   {results['roc_auc']:.4f}")
    
    print(f"\nBiometric-Specific Metrics:")
    print(f"  FAR (False Accept Rate):  {best_metrics['far']:.2%}")
    print(f"  FRR (False Reject Rate):  {best_metrics['frr']:.2%}")
    print(f"  EER (Equal Error Rate):   {(best_metrics['far'] + best_metrics['frr']) / 2:.2%}")
    
    print(f"\nConfusion Matrix:")
    cm = np.array(results['confusion_matrix'])
    print(f"  True Negatives:  {cm[0, 0]}")
    print(f"  False Positives: {cm[0, 1]} (Impostors accepted)")
    print(f"  False Negatives: {cm[1, 0]} (Legitimate rejected)")
    print(f"  True Positives:  {cm[1, 1]}")
    
    print("\n" + "=" * 80)


def main():
    parser = argparse.ArgumentParser(description='Evaluate behavioral biometrics model')
    parser.add_argument('--model', type=str, required=True, help='Path to trained model')
    parser.add_argument('--test-data', type=str, required=True, help='Path to test data CSV')
    parser.add_argument('--scaler', type=str, default=None, help='Path to scaler')
    parser.add_argument('--device', type=str, default='cpu', help='Device (cpu/cuda)')
    parser.add_argument('--output-dir', type=str, default='evaluation_results', help='Output directory')
    
    args = parser.parse_args()
    
    # Set device
    device = torch.device(args.device)
    print(f"Using device: {device}")
    
    # Load model
    print(f"\nLoading model from {args.model}...")
    model = BehavioralAuthNN(input_dim=7)
    model.load_state_dict(torch.load(args.model, map_location=device))
    model = model.to(device)
    model.eval()
    
    # Load test data
    print(f"Loading test data from {args.test_data}...")
    X_test, y_test = load_test_data(args.test_data)
    
    # Load and apply scaler if provided
    if args.scaler:
        import pickle
        with open(args.scaler, 'rb') as f:
            scaler = pickle.load(f)
        X_test = scaler.transform(X_test)
    
    print(f"Test samples: {len(X_test)}")
    print(f"  Legitimate: {y_test.sum()} ({y_test.mean():.1%})")
    print(f"  Impostor: {len(y_test) - y_test.sum()} ({1 - y_test.mean():.1%})")
    
    # Evaluate
    print("\n Evaluating model...")
    results = evaluate_model(model, X_test, y_test, device)
    
    # Print summary
    print_evaluation_summary(results)
    
    # Generate plots
    print("\n Generating evaluation plots...")
    plot_results(results, args.output_dir)
    
    # Save results to JSON
    output_path = Path(args.output_dir) / 'evaluation_results.json'
    with open(output_path, 'w') as f:
        json.dump(results, f, indent=2)
    print(f"\n Results saved to {output_path}")


if __name__ == '__main__':
    main()