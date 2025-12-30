"""
API endpoint tests
"""

import pytest
from fastapi.testclient import TestClient
from app.main import app

client = TestClient(app)


def test_root_endpoint():
    """Test root endpoint"""
    response = client.get("/")
    assert response.status_code == 200
    data = response.json()
    assert data["service"] == "Behavioral Biometrics ML"
    assert "version" in data


def test_health_check():
    """Test health check endpoint"""
    response = client.get("/health")
    assert response.status_code == 200
    data = response.json()
    assert "status" in data
    assert "model_loaded" in data
    assert "model_version" in data


def test_predict_endpoint_valid():
    """Test prediction with valid input"""
    request_data = {
        "did": "0x" + "0" * 64,
        "features": {
            "typing_speed_wpm": 65,
            "avg_key_hold_time_ms": 120,
            "avg_transition_time_ms": 85,
            "error_rate_percent": 3,
            "activity_hour_preference": 14
        }
    }
    
    response = client.post("/predict", json=request_data)
    assert response.status_code == 200
    
    data = response.json()
    assert "confidence_score" in data
    assert 0 <= data["confidence_score"] <= 100
    assert "anomaly_score" in data
    assert "model_version" in data


def test_predict_endpoint_invalid_did():
    """Test prediction with invalid DID"""
    request_data = {
        "did": "invalid_did",
        "features": {
            "typing_speed_wpm": 65,
            "avg_key_hold_time_ms": 120,
            "avg_transition_time_ms": 85,
            "error_rate_percent": 3,
            "activity_hour_preference": 14
        }
    }
    
    response = client.post("/predict", json=request_data)
    assert response.status_code == 422  # Validation error


def test_predict_endpoint_invalid_features():
    """Test prediction with out-of-range features"""
    request_data = {
        "did": "0x" + "0" * 64,
        "features": {
            "typing_speed_wpm": 500,  # Too high
            "avg_key_hold_time_ms": 120,
            "avg_transition_time_ms": 85,
            "error_rate_percent": 3,
            "activity_hour_preference": 14
        }
    }
    
    response = client.post("/predict", json=request_data)
    assert response.status_code == 422


def test_predict_with_historical_patterns():
    """Test prediction with historical patterns"""
    request_data = {
        "did": "0x" + "0" * 64,
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
    
    response = client.post("/predict", json=request_data)
    assert response.status_code == 200


def test_batch_predict():
    """Test batch prediction endpoint"""
    requests = [
        {
            "did": "0x" + "0" * 64,
            "features": {
                "typing_speed_wpm": 65,
                "avg_key_hold_time_ms": 120,
                "avg_transition_time_ms": 85,
                "error_rate_percent": 3,
                "activity_hour_preference": 14
            }
        },
        {
            "did": "0x" + "1" * 64,
            "features": {
                "typing_speed_wpm": 70,
                "avg_key_hold_time_ms": 115,
                "avg_transition_time_ms": 90,
                "error_rate_percent": 5,
                "activity_hour_preference": 16
            }
        }
    ]
    
    response = client.post("/batch-predict", json=requests)
    assert response.status_code == 200
    data = response.json()
    assert len(data) == 2


def test_batch_predict_limit():
    """Test batch prediction size limit"""
    # Create 101 requests (exceeds limit of 100)
    requests = [
        {
            "did": "0x" + "0" * 64,
            "features": {
                "typing_speed_wpm": 65,
                "avg_key_hold_time_ms": 120,
                "avg_transition_time_ms": 85,
                "error_rate_percent": 3,
                "activity_hour_preference": 14
            }
        }
    ] * 101
    
    response = client.post("/batch-predict", json=requests)
    assert response.status_code == 400


def test_metrics_endpoint():
    """Test metrics endpoint"""
    response = client.get("/metrics")
    assert response.status_code == 200


if __name__ == "__main__":
    pytest.main([__file__, "-v"])