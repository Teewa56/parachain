# PortableID Machine Learning Model: Technical Documentation

## 1. Introduction

The PortableID Machine Learning Model is a behavioral biometrics authentication system designed to provide "Proof of Personhood" and continuous authentication. By analyzing unique patterns in how users interact with their devices (e.g., typing speed, key rhythm), the system generates a confidence score that can be used on-chain to verify identity without revealing sensitive biometric data.

## 2. Core Concepts: Behavioral Biometrics

Unlike static biometrics (fingerprints, face), behavioral biometrics analyze *how* a user performs an action. This has several advantages:
- **Continuous Auth**: Can verify the user is still the same person throughout a session.
- **Non-Invasive**: Requires no specialized hardware beyond standard input devices.
- **Privacy-Preserving**: The raw data (exact timestamps) can be normalized and obfuscated before processing.

## 3. Technology Stack

- **Framework**: PyTorch
- **Language**: Python 3.9+
- **API Wrapper**: FastAPI
- **Data Science**: NumPy, Scikit-learn, Pandas
- **Deployment**: Docker with Uvicorn

## 4. Model Architecture

The system utilizes an ensemble of two distinct neural network architectures to ensure high precision and robustness.

### 4.1 Behavioral Authentication Network (`BehavioralAuthNN`)
A deep neural network trained to recognize specific user patterns.
- **Input Layer**: 6 features (see section 5).
- **Hidden Layers**: 3 dense layers (128, 64, 32 units) with ReLU activation and Batch Normalization.
- **Output Layer**: Sigmoid activation providing a confidence score between 0 and 1.
- **Regularization**: Dropout (30%) to prevent overfitting to specific capture sessions.

### 4.2 Anomaly Detector (`BehavioralAnomalyDetector`)
An Autoencoder designed to detect "unusual" behavior that might indicate a compromised session or automated bot activity.
- **Encoder**: Compresses 6 features into a latent space of 3.
- **Decoder**: Attempts to reconstruct the original features from the latent space.
- **Anomaly Score**: Calculated based on the Mean Squared Error (MSE) between input and reconstruction. High error = High anomaly.

### 4.3 Ensemble Logic
The final confidence score is a weighted combination:
```python
final_score = auth_confidence * (1 - anomaly_weight * sigmoid(anomaly_score))
```

## 5. Feature Engineering

The model processes a set of normalized behavioral features:

| Feature Name | Description | Units |
|--------------|-------------|-------|
| `typing_speed_wpm` | Words per minute during input | WPM |
| `avg_key_hold_time_ms` | Duration a key remains pressed | ms |
| `avg_transition_time_ms` | Time between releasing a key and pressing the next | ms |
| `error_rate_percent` | Frequency of backspaces/corrections | % |
| `activity_hour_preference` | Normalized hour of day (circadian rhythm) | 0-23 |
| `pattern_consistency` | Variance in the above metrics over a window | scalar |

### 5.1 Preprocessing Pipeline
1. **Outlier Clipping**: Removes extreme values (e.g., 5000ms key hold) that indicate system lag rather than human behavior.
2. **Scaling**: Uses `StandardScaler` to normalize features to zero mean and unit variance.
3. **Temporal Extraction**: Extracts trends over sliding windows to capture the "rhythm" of interaction.

## 6. API Specification

The model is exposed via a RESTful API for inference.

### 6.1 Predict Endpoint
- **URL**: `/api/v1/inference/predict`
- **Method**: `POST`
- **Payload**:
```json
{
  "did": "did:portableid:0x...",
  "features": {
    "typing_speed_wpm": 65,
    "avg_key_hold_time_ms": 82,
    "avg_transition_time_ms": 115,
    "error_rate_percent": 2.5,
    "activity_hour_preference": 14
  }
}
```
- **Response**:
```json
{
  "confidence_score": 94.2,
  "is_authentic": true,
  "anomaly_detected": false,
  "timestamp": "2024-05-20T10:00:00Z"
}
```

## 7. Security and Privacy

- **Data Minimization**: The API does not store raw timestamps; it only receives pre-calculated metrics.
- **Salted DID Hashing**: User identities are referenced by their DID hash to prevent PII leakage in model logs.
- **Oracle Integration**: In production, this service acts as an "Off-chain Oracle" that submits signed scores to the parachain's `pallet-proof-of-personhood`.

## 8. Deployment

The service is containerized for easy scaling and consistent environments.
```bash
# Build and run
docker build -t portableid-ml-oracle .
docker run -p 8000:8000 portableid-ml-oracle
```

---
*Documentation Version: 1.0.0*
