API Documentation
Behavioral Biometrics ML API Documentation
Complete API reference for the behavioral biometrics machine learning service.
Base URL
http://localhost:8000
Authentication
Currently, no authentication is required. For production, implement:

API key authentication
Rate limiting per client
IP whitelisting for internal services


Endpoints
1. Health Check
GET /health
Check service health and model status.
Response:
json{
  "status": "healthy",
  "model_loaded": true,
  "model_version": "1.0.0",
  "device": "cpu",
  "uptime_seconds": 3600
}
Status Codes:

200 - Service is healthy
503 - Service is unhealthy (model not loaded)


2. Predict Confidence Score
POST /predict
Predict authentication confidence for a behavioral pattern.
Request Body:
json{
  "did": "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb000000000000000000000000",
  "features": {
    "typing_speed_wpm": 65,
    "avg_key_hold_time_ms": 120,
    "avg_transition_time_ms": 85,
    "error_rate_percent": 3,
    "activity_hour_preference": 14
  },
  "historical_patterns": [
    {
      "typing_speed_wpm": 63,
      "avg_key_hold_time_ms": 118,
      "avg_transition_time_ms": 87,
      "error_rate_percent": 4,
      "timestamp": 1704067200
    }
  ]
}
Response:
json{
  "confidence_score": 87,
  "anomaly_score": 0.12,
  "feature_importance": {
    "typing_speed": 0.18,
    "key_hold_time": 0.24,
    "transition_time": 0.35,
    "error_rate": 0.10,
    "speed_accuracy_ratio": 0.08,
    "rhythm_ratio": 0.05
  },
  "model_version": "1.0.0",
  "inference_time_ms": 12.45
}
Status Codes:

200 - Successful prediction
400 - Invalid input
503 - Model not loaded


3. Batch Prediction
POST /batch-predict
Process multiple predictions in one request.
Request Body:
json[
  {
    "did": "0x742d35...",
    "features": { /* ... */ }
  },
  {
    "did": "0x853e46...",
    "features": { /* ... */ }
  }
]
Limits:

Maximum 100 requests per batch

Response:
json[
  {
    "confidence_score": 87,
    "anomaly_score": 0.12,
    "model_version": "1.0.0",
    "inference_time_ms": 12.45
  },
  {
    "confidence_score": 92,
    "anomaly_score": 0.08,
    "model_version": "1.0.0",
    "inference_time_ms": 11.23
  }
]

4. Metrics
GET /metrics
Get Prometheus-style metrics.
Response:
# HELP prediction_count Total number of predictions
# TYPE prediction_count counter
prediction_count 1543

# HELP average_inference_time_ms Average inference time
# TYPE average_inference_time_ms gauge
average_inference_time_ms 12.34

5. Statistics
GET /stats
Get detailed statistics in JSON format.
Response:
json{
  "model": {
    "version": "1.0.0",
    "device": "cpu",
    "loaded": true
  },
  "performance": {
    "total_predictions": 1543,
    "avg_inference_time_ms": 12.34,
    "avg_confidence": 85.67
  },
  "system": {
    "cpu_percent": 23.5,
    "memory_used_mb": 512.34,
    "memory_available_mb": 7680.12,
    "memory_percent": 6.2
  }
}

Error Responses
All errors follow this format:
json{
  "detail": "Error message",
  "error_code": "ERROR_CODE",
  "path": "/predict"
}
Common Error Codes:

INVALID_INPUT - Validation error
MODEL_NOT_LOADED - Service not ready
INFERENCE_ERROR - Processing error


Rate Limiting
Current limits (per IP):

1000 requests per minute
100 requests per second


Examples
Python
pythonimport requests

url = "http://localhost:8000/predict"
payload = {
    "did": "0x742d35...",
    "features": {
        "typing_speed_wpm": 65,
        "avg_key_hold_time_ms": 120,
        "avg_transition_time_ms": 85,
        "error_rate_percent": 3,
        "activity_hour_preference": 14
    }
}

response = requests.post(url, json=payload)
result = response.json()
print(f"Confidence: {result['confidence_score']}%")
cURL
bashcurl -X POST http://localhost:8000/predict \
  -H "Content-Type: application/json" \
  -d '{
    "did": "0x742d35...",
    "features": {
      "typing_speed_wpm": 65,
      "avg_key_hold_time_ms": 120,
      "avg_transition_time_ms": 85,
      "error_rate_percent": 3,
      "activity_hour_preference": 14
    }
  }'
JavaScript
javascriptconst response = await fetch('http://localhost:8000/predict', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    did: '0x742d35...',
    features: {
      typing_speed_wpm: 65,
      avg_key_hold_time_ms: 120,
      avg_transition_time_ms: 85,
      error_rate_percent: 3,
      activity_hour_preference: 14
    }
  })
});

const result = await response.json();
console.log(`Confidence: ${result.confidence_score}%`);

