# Behavioral Biometrics Training Data

This directory contains datasets for training and evaluating the behavioral authentication model.

## Directory Structure

```
data/
├── raw/                    # Raw training data
│   ├── synthetic_data.csv  # Generated synthetic samples
│   └── real_data.csv       # Real user data (if available)
├── processed/              # Preprocessed features
│   ├── train.npy          # Training features
│   ├── test.npy           # Test features
│   └── scaler.pkl         # Fitted StandardScaler
└── README.md              # This file
```

## Data Format

### CSV Format (raw data)

All raw data should be in CSV format with the following columns:

| Column | Type | Range | Description |
|--------|------|-------|-------------|
| `user_id` | int | 0-∞ | Unique user identifier |
| `typing_speed_wpm` | int | 10-200 | Words per minute typing speed |
| `avg_key_hold_time_ms` | int | 40-300 | Average key hold time in milliseconds |
| `avg_transition_time_ms` | int | 30-250 | Average transition time between keys |
| `error_rate_percent` | int | 0-30 | Error rate percentage |
| `activity_hour_preference` | int | 0-23 | Preferred hour of day for activity |
| `is_legitimate` | int | 0 or 1 | 1 = legitimate user, 0 = impostor |

**Example CSV:**

```csv
user_id,typing_speed_wpm,avg_key_hold_time_ms,avg_transition_time_ms,error_rate_percent,activity_hour_preference,is_legitimate
0,65,120,85,3,14,1
0,63,118,87,4,15,1
0,67,122,83,2,14,1
1,72,110,90,5,10,0
```

## Generating Synthetic Data

For initial development and testing, use the synthetic data generator:

```bash
python scripts/generate_synthetic.py \
    --samples 10000 \
    --output data/raw/synthetic_data.csv \
    --legitimate-ratio 0.8
```

### Synthetic Data Characteristics

- **Legitimate users**: Consistent patterns with small natural variations
  - Typing speed: ±5 WPM variation
  - Key hold time: ±10ms variation
  - Transition time: ±8ms variation
  - Error rate: ±1% variation
  - Activity hour: ±2 hour variation

- **Impostors**: Different base characteristics
  - Higher variance in all metrics
  - Different timing patterns
  - No correlation with legitimate user patterns

## Collecting Real Data

### Data Collection Requirements

For production-ready models, use real user data:

1. **Minimum Samples per User**
   - Legitimate: 50-100 samples per user
   - Impostor: 20-40 samples per user

2. **Data Diversity**
   - Multiple sessions per user
   - Different times of day
   - Various text types (emails, documents, code)
   - Different devices/keyboards

3. **Privacy Considerations**
   - ⚠️ **NEVER store raw keystroke data or actual text content**
   - Only store derived timing features
   - Anonymize user IDs
   - Obtain informed consent
   - Comply with GDPR/CCPA

### Collection Script Template

```python
import time
from pynput import keyboard

class KeystrokeCollector:
    def __init__(self):
        self.key_press_times = {}
        self.hold_times = []
        self.transition_times = []
        self.last_release_time = None
    
    def on_press(self, key):
        self.key_press_times[key] = time.time()
    
    def on_release(self, key):
        if key in self.key_press_times:
            # Calculate hold time
            hold_time = time.time() - self.key_press_times[key]
            self.hold_times.append(hold_time * 1000)  # Convert to ms
            
            # Calculate transition time
            if self.last_release_time:
                transition = self.key_press_times[key] - self.last_release_time
                self.transition_times.append(transition * 1000)
            
            self.last_release_time = time.time()
            del self.key_press_times[key]
    
    def get_features(self):
        if not self.hold_times:
            return None
        
        return {
            'avg_key_hold_time_ms': int(sum(self.hold_times) / len(self.hold_times)),
            'avg_transition_time_ms': int(sum(self.transition_times) / len(self.transition_times)) if self.transition_times else 0,
        }

# Note: This is a simplified example. Production code should:
# - Handle errors gracefully
# - Add session management
# - Implement proper data storage
# - Add privacy protections
```

## Data Preprocessing

Before training, preprocess raw data:

```python
from sklearn.preprocessing import StandardScaler
import numpy as np
import pandas as pd

# Load data
df = pd.read_csv('data/raw/synthetic_data.csv')

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

# Add derived features
speed_accuracy = X[:, 0] / (X[:, 3] + 1)
rhythm_ratio = X[:, 1] / (X[:, 2] + 1)
X = np.column_stack([X, speed_accuracy, rhythm_ratio])

# Normalize
scaler = StandardScaler()
X_normalized = scaler.fit_transform(X)

# Save
np.save('data/processed/train.npy', X_normalized)
np.save('data/processed/labels.npy', y)

import pickle
with open('data/processed/scaler.pkl', 'wb') as f:
    pickle.dump(scaler, f)
```

## Data Quality Checks

Before training, verify data quality:

```python
def validate_dataset(df):
    """Validate dataset meets quality requirements"""
    
    # Check for missing values
    assert df.isnull().sum().sum() == 0, "Missing values found"
    
    # Check value ranges
    assert df['typing_speed_wpm'].between(10, 200).all(), "Invalid typing speed"
    assert df['avg_key_hold_time_ms'].between(20, 500).all(), "Invalid hold time"
    assert df['avg_transition_time_ms'].between(20, 400).all(), "Invalid transition time"
    assert df['error_rate_percent'].between(0, 50).all(), "Invalid error rate"
    assert df['activity_hour_preference'].between(0, 23).all(), "Invalid hour"
    assert df['is_legitimate'].isin([0, 1]).all(), "Invalid labels"
    
    # Check class balance
    legitimate_ratio = df['is_legitimate'].mean()
    assert 0.5 <= legitimate_ratio <= 0.9, f"Imbalanced classes: {legitimate_ratio:.1%}"
    
    # Check sufficient samples
    assert len(df) >= 1000, f"Insufficient samples: {len(df)}"
    
    print("✅ Dataset validation passed")
```

## Dataset Statistics

After generating or collecting data, compute statistics:

```python
def print_dataset_stats(df):
    """Print dataset statistics"""
    
    print("=" * 60)
    print("DATASET STATISTICS")
    print("=" * 60)
    
    print(f"\nTotal samples: {len(df):,}")
    print(f"Legitimate: {df['is_legitimate'].sum():,} ({df['is_legitimate'].mean():.1%})")
    print(f"Impostor: {(~df['is_legitimate'].astype(bool)).sum():,} ({1 - df['is_legitimate'].mean():.1%})")
    
    print("\nFeature Statistics:")
    print(df.describe())
    
    print("\nFeature Correlations:")
    feature_cols = ['typing_speed_wpm', 'avg_key_hold_time_ms', 
                    'avg_transition_time_ms', 'error_rate_percent']
    print(df[feature_cols].corr())
```

## Augmentation Strategies

To increase training data:

1. **Temporal Jitter**: Add small random noise to timing features (±5%)
2. **Feature Perturbation**: Slight modifications within natural variation
3. **Session Simulation**: Combine multiple samples to simulate sessions

```python
def augment_sample(sample, noise_level=0.05):
    """Add realistic noise to sample"""
    noisy_sample = sample.copy()
    
    # Add Gaussian noise
    for feature in ['typing_speed_wpm', 'avg_key_hold_time_ms', 'avg_transition_time_ms']:
        noise = np.random.normal(0, noise_level * sample[feature])
        noisy_sample[feature] = max(0, sample[feature] + noise)
    
    return noisy_sample
```


## References

- [Keystroke Dynamics Datasets](https://github.com/topics/keystroke-dynamics)
- [Privacy-Preserving Behavioral Biometrics](https://arxiv.org/abs/1903.09515)