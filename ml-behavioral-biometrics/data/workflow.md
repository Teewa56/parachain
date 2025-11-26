# Complete ML Workflow Guide

## 📋 Table of Contents

1. [Project Setup](#1-project-setup)
2. [Data Generation/Collection](#2-data-generationcollection)
3. [Model Training](#3-model-training)
4. [Model Evaluation](#4-model-evaluation)
5. [API Deployment](#5-api-deployment)
6. [Substrate Integration](#6-substrate-integration)
7. [Production Checklist](#7-production-checklist)

---

## 1. Project Setup

### Initial Setup

```bash
# Clone/create project directory
mkdir ml-behavioral-biometrics
cd ml-behavioral-biometrics

# Create virtual environment
python -m venv venv
source venv/bin/activate  # On Windows: venv\Scripts\activate

# Install dependencies
pip install -r requirements.txt

# Create directory structure
mkdir -p data/{raw,processed}
mkdir -p models/{production,checkpoints,experiments}
mkdir -p logs
mkdir -p evaluation_results
```

### Environment Configuration

Create `.env` file:

```bash
# .env
MODEL_PATH=models/production/model.pth
SCALER_PATH=models/production/scaler.pkl
MODEL_VERSION=1.0.0
DEVICE=cpu
HOST=0.0.0.0
PORT=8000
WORKERS=4
DEBUG=false
```

---

## 2. Data Generation/Collection

### Option A: Synthetic Data (Development/Testing)

```bash
# Generate 10,000 synthetic samples
python scripts/generate_synthetic.py \
    --samples 10000 \
    --output data/raw/synthetic_data.csv \
    --legitimate-ratio 0.8

# Verify data quality
python -c "
import pandas as pd
df = pd.read_csv('data/raw/synthetic_data.csv')
print(df.describe())
print(f'\nLegitimate: {df.is_legitimate.sum()} ({df.is_legitimate.mean():.1%})')
"
```

### Option B: Real Data Collection

#### Step 1: Set Up Collection Infrastructure

Create a data collection script (`scripts/collect_behavioral_data.py`):

```python
#!/usr/bin/env python3
"""
Behavioral data collection tool
IMPORTANT: Only collects timing features, NOT keystrokes or content
"""

import json
import time
from datetime import datetime
from pathlib import Path
from pynput import keyboard

class BehavioralDataCollector:
    def __init__(self, user_id: str, session_id: str):
        self.user_id = user_id
        self.session_id = session_id
        self.key_press_times = {}
        self.key_events = []
        self.session_start = time.time()
        
    def on_press(self, key):
        """Record key press time"""
        try:
            key_code = key.char if hasattr(key, 'char') else str(key)
            self.key_press_times[key_code] = time.time()
        except Exception:
            pass
    
    def on_release(self, key):
        """Record key release and calculate features"""
        try:
            key_code = key.char if hasattr(key, 'char') else str(key)
            if key_code in self.key_press_times:
                release_time = time.time()
                press_time = self.key_press_times[key_code]
                hold_time = (release_time - press_time) * 1000  # ms
                
                self.key_events.append({
                    'hold_time_ms': hold_time,
                    'timestamp': release_time,
                })
                
                del self.key_press_times[key_code]
        except Exception:
            pass
        
        # Stop on ESC
        if key == keyboard.Key.esc:
            return False
    
    def calculate_features(self):
        """Calculate behavioral features from collected events"""
        if len(self.key_events) < 10:
            return None
        
        # Calculate typing speed (approximate)
        session_duration_minutes = (time.time() - self.session_start) / 60
        estimated_words = len(self.key_events) / 5  # Avg 5 chars per word
        typing_speed = int(estimated_words / session_duration_minutes) if session_duration_minutes > 0 else 0
        
        # Calculate hold times
        hold_times = [e['hold_time_ms'] for e in self.key_events]
        avg_hold_time = int(sum(hold_times) / len(hold_times))
        
        # Calculate transition times
        transition_times = []
        for i in range(1, len(self.key_events)):
            transition = (self.key_events[i]['timestamp'] - 
                         self.key_events[i-1]['timestamp']) * 1000
            if 0 < transition < 1000:  # Filter outliers
                transition_times.append(transition)
        
        avg_transition = int(sum(transition_times) / len(transition_times)) if transition_times else 0
        
        # Error rate (placeholder - would need actual error detection)
        error_rate = 3  # Default
        
        # Activity hour
        activity_hour = datetime.now().hour
        
        return {
            'user_id': self.user_id,
            'session_id': self.session_id,
            'typing_speed_wpm': typing_speed,
            'avg_key_hold_time_ms': avg_hold_time,
            'avg_transition_time_ms': avg_transition,
            'error_rate_percent': error_rate,
            'activity_hour_preference': activity_hour,
            'timestamp': int(time.time()),
        }
    
    def save_features(self, output_path: str):
        """Save features to JSON"""
        features = self.calculate_features()
        if features:
            Path(output_path).parent.mkdir(parents=True, exist_ok=True)
            with open(output_path, 'a') as f:
                f.write(json.dumps(features) + '\n')
            print(f"✅ Saved features: {features}")

def main():
    import argparse
    parser = argparse.ArgumentParser()
    parser.add_argument('--user-id', required=True, help='User ID')
    parser.add_argument('--output', default='data/raw/collected_features.jsonl')
    args = parser.parse_args()
    
    session_id = f"session_{int(time.time())}"
    collector = BehavioralDataCollector(args.user_id, session_id)
    
    print("=" * 60)
    print("BEHAVIORAL DATA COLLECTOR")
    print("=" * 60)
    print(f"User ID: {args.user_id}")
    print(f"Session: {session_id}")
    print("\n⌨️  Start typing... (Press ESC to stop)")
    print("\nNOTE: Only timing features are collected, NOT keystrokes!")
    print("=" * 60)
    
    # Start listener
    with keyboard.Listener(
        on_press=collector.on_press,
        on_release=collector.on_release
    ) as listener:
        listener.join()
    
    # Save features
    collector.save_features(args.output)
    print(f"\n✅ Collection complete. Data saved to {args.output}")

if __name__ == '__main__':
    main()
```

#### Step 2: Collect Data from Multiple Users

```bash
# User 1 (multiple sessions)
python scripts/collect_behavioral_data.py --user-id user_001
python scripts/collect_behavioral_data.py --user-id user_001
python scripts/collect_behavioral_data.py --user-id user_001

# User 2
python scripts/collect_behavioral_data.py --user-id user_002
# ... etc

# Collect from at least 50+ users with 10+ sessions each
```

#### Step 3: Convert JSONL to CSV

```python
# scripts/jsonl_to_csv.py
import json
import pandas as pd

features_list = []
with open('data/raw/collected_features.jsonl') as f:
    for line in f:
        features_list.append(json.loads(line))

df = pd.DataFrame(features_list)

# Add labels (1 = legitimate, 0 = impostor)
# You'd need to label impostor attempts manually or simulate them
df['is_legitimate'] = 1  # Default to legitimate

df.to_csv('data/raw/real_data.csv', index=False)
print(f"Converted {len(df)} samples to CSV")
```

---

## 3. Model Training

### Step 1: Validate Data

```bash
python -c "
import pandas as pd

df = pd.read_csv('data/raw/synthetic_data.csv')

# Check for issues
assert df.isnull().sum().sum() == 0, 'Missing values!'
assert len(df) >= 1000, 'Not enough samples!'
assert 0.5 <= df['is_legitimate'].mean() <= 0.9, 'Imbalanced!'

print('✅ Data validation passed')
"
```

### Step 2: Train Model

```bash
# Basic training
python scripts/train_model.py \
    --data data/raw/synthetic_data.csv \
    --epochs 100 \
    --batch-size 256 \
    --learning-rate 0.001 \
    --output-dir models/production

# With data augmentation
python scripts/train_model.py \
    --data data/raw/synthetic_data.csv \
    --epochs 100 \
    --augment \
    --output-dir models/production

# GPU training (if available)
python scripts/train_model.py \
    --data data/raw/synthetic_data.csv \
    --epochs 100 \
    --device cuda \
    --output-dir models/production
```

### Step 3: Monitor Training

Training will output:

```
Epoch [1/100] (15.32s)
  Train Loss: 0.4523
  Val Loss:   0.3891
  Val Acc:    85.23%
  Val F1:     0.8456
  FAR:        3.21%
  FRR:        5.67%
  LR:         0.001000
  ✓ Saved best model (loss: 0.3891)
```

---

## 4. Model Evaluation

### Comprehensive Evaluation

```bash
python scripts/evaluate_model.py \
    --model models/production/model.pth \
    --test-data data/raw/synthetic_data.csv \
    --scaler models/production/scaler.pkl \
    --output-dir evaluation_results
```

This generates:

- `roc_curve.png` - ROC curve with AUC score
- `confusion_matrix.png` - Confusion matrix heatmap
- `threshold_analysis.png` - FAR/FRR at different thresholds
- `score_distribution.png` - Score histograms
- `evaluation_results.json` - Detailed metrics

### Interpret Results

Target metrics for production:
- **Accuracy**: >90%
- **FAR (False Accept)**: <3%
- **FRR (False Reject)**: <5%
- **AUC-ROC**: >0.95

---

## 5. API Deployment

### Local Development

```bash
# Start development server
uvicorn app.main:app --reload --port 8000

# Test endpoint
curl -X POST http://localhost:8000/predict \
  -H "Content-Type: application/json" \
  -d '{
    "did": "0x0000000000000000000000000000000000000000000000000000000000000000",
    "features": {
      "typing_speed_wpm": 65,
      "avg_key_hold_time_ms": 120,
      "avg_transition_time_ms": 85,
      "error_rate_percent": 3,
      "activity_hour_preference": 14
    }
  }'
```

### Docker Deployment

```bash
# Build image
docker build -f docker/Dockerfile -t behavioral-ml:latest .

# Run container
docker run -d \
    --name behavioral-ml \
    -p 8000:8000 \
    -v $(pwd)/models:/app/models:ro \
    behavioral-ml:latest

# Check health
curl http://localhost:8000/health
```

### Production Deployment with Docker Compose

```bash
# Start all services (ML API + monitoring)
docker-compose -f docker/docker-compose.yml up -d

# View logs
docker-compose logs -f ml-service

# Scale workers
docker-compose up -d --scale ml-service=4
```

---

## 6. Substrate Integration

### Step 1: Configure Runtime

In your Substrate runtime's `lib.rs`:

```rust
impl pallet_proof_of_personhood::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type TimeProvider = Timestamp;
    type RegistrationDeposit = ConstU128<{ 10 * UNIT }>;
    type RecoveryDeposit = ConstU128<{ 5 * UNIT }>;
    type ZkCredentials = ZkCredentials;
    type WeightInfo = ();
    type AuthorityId = pallet_proof_of_personhood::crypto::TestAuthId;
}
```

### Step 2: Set ML Service URL

```bash
# Via governance or sudo
polkadot-js-api \
    --ws ws://localhost:9944 \
    tx.proofOfPersonhood.setMlServiceUrl \
    "http://ml-service:8000/predict"
```

### Step 3: Test Off-Chain Worker

```rust
// The off-chain worker will automatically:
// 1. Check for pending patterns every 10 blocks
// 2. Call ML service HTTP endpoint
// 3. Submit signed transactions with scores

// Queue a pattern for ML scoring
let features = BehavioralFeatures {
    typing_speed_wpm: 65,
    avg_key_hold_time_ms: 120,
    avg_transition_time_ms: 85,
    error_rate_percent: 3,
    activity_hour_preference: 14,
};

ProofOfPersonhood::queue_for_ml_scoring(
    Origin::signed(account),
    features.encode(),
)?;
```

---

## 7. Production Checklist

### Pre-Launch

- [ ] Collect real user data (500+ users minimum)
- [ ] Train model on real data (>95% accuracy)
- [ ] Complete evaluation on hold-out test set
- [ ] Load test API (1000+ req/s)
- [ ] Set up monitoring (Prometheus + Grafana)
- [ ] Configure alerts for:
  - High error rates (>5%)
  - High latency (>100ms)
  - Model drift
  - Off-chain worker failures
- [ ] Security audit of API endpoints
- [ ] Privacy compliance review (GDPR/CCPA)
- [ ] Documentation for users

### Post-Launch Monitoring

- [ ] Track FAR/FRR in production
- [ ] Monitor model drift (retrain quarterly)
- [ ] Collect feedback from users
- [ ] A/B test threshold adjustments
- [ ] Regular security audits

### Model Retraining

```bash
# Quarterly retraining workflow
# 1. Export production data
# 2. Retrain model
python scripts/train_model.py \
    --data data/raw/production_q1_2024.csv \
    --epochs 100 \
    --output-dir models/experiments/v2.0.0

# 3. Evaluate new model
python scripts/evaluate_model.py \
    --model models/experiments/v2.0.0/model.pth \
    --test-data data/raw/holdout_test.csv

# 4. A/B test (50/50 traffic split)
# 5. Gradual rollout if metrics improve
# 6. Promote to production
```

---

## Troubleshooting

### Model Not Loading

```bash
# Check model file
ls -lh models/production/model.pth

# Verify integrity
python -c "import torch; torch.load('models/production/model.pth')"
```

### Low Accuracy

- Collect more training data (aim for 100k+ samples)
- Balance classes (80% legitimate / 20% impostor)
- Tune hyperparameters
- Try ensemble methods

### High Latency

- Enable GPU inference
- Reduce batch processing
- Use model quantization
- Scale horizontally (more workers)

---

## Next Steps

1. **Start with synthetic data** for development
2. **Collect real data** for production model
3. **Train and evaluate** iteratively
4. **Deploy API** with monitoring
5. **Integrate with Substrate** via off-chain worker
6. **Monitor and retrain** regularly

For more details, see individual component READMEs in each directory.