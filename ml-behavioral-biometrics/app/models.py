"""
Pydantic models for API request/response validation
"""

from pydantic import BaseModel, Field, field_validator
from typing import Optional, Dict, List


class BehavioralFeatures(BaseModel):
    """Behavioral biometric features"""
    
    typing_speed_wpm: int = Field(
        ..., 
        ge=0, 
        le=300,
        description="Words per minute typing speed",
        examples=[65]
    )
    avg_key_hold_time_ms: int = Field(
        ...,
        ge=0,
        le=5000,
        description="Average key hold time in milliseconds",
        examples=[120]
    )
    avg_transition_time_ms: int = Field(
        ...,
        ge=0,
        le=5000,
        description="Average transition time between keys in milliseconds",
        examples=[85]
    )
    error_rate_percent: int = Field(
        ...,
        ge=0,
        le=100,
        description="Error rate percentage (0-100)",
        examples=[3]
    )
    activity_hour_preference: int = Field(
        ...,
        ge=0,
        lt=24,
        description="Preferred hour of day for activity (0-23)",
        examples=[14]
    )
    
    @field_validator('typing_speed_wpm')
    @classmethod
    def validate_typing_speed(cls, v):
        if v == 0:
            raise ValueError("Typing speed must be greater than 0")
        return v
    
    class Config:
        json_schema_extra = {
            "example": {
                "typing_speed_wpm": 65,
                "avg_key_hold_time_ms": 120,
                "avg_transition_time_ms": 85,
                "error_rate_percent": 3,
                "activity_hour_preference": 14
            }
        }


class HistoricalPattern(BaseModel):
    """Historical behavioral pattern for comparison"""
    
    typing_speed_wpm: int = Field(
        ...,
        ge=0,
        le=300,
        description="Historical typing speed"
    )
    avg_key_hold_time_ms: int = Field(
        ...,
        ge=0,
        le=5000,
        description="Historical key hold time"
    )
    avg_transition_time_ms: int = Field(
        ...,
        ge=0,
        le=5000,
        description="Historical transition time"
    )
    error_rate_percent: int = Field(
        ...,
        ge=0,
        le=100,
        description="Historical error rate"
    )
    timestamp: int = Field(
        ...,
        description="Unix timestamp when pattern was recorded",
        examples=[1704067200]
    )
    
    class Config:
        json_schema_extra = {
            "example": {
                "typing_speed_wpm": 63,
                "avg_key_hold_time_ms": 118,
                "avg_transition_time_ms": 87,
                "error_rate_percent": 4,
                "timestamp": 1704067200
            }
        }


class PredictRequest(BaseModel):
    """Request model for prediction endpoint"""
    
    did: str = Field(
        ...,
        min_length=66,
        max_length=66,
        description="Decentralized Identifier (0x...)",
        examples=["0x0000000000000000000000000000000000000000000000000000000000000000"]
    )
    features: BehavioralFeatures = Field(
        ...,
        description="Current behavioral features"
    )
    historical_patterns: Optional[List[HistoricalPattern]] = Field(
        default=None,
        max_length=10,
        description="Up to 10 historical patterns for comparison"
    )
    
    @field_validator('did')
    @classmethod
    def validate_did(cls, v):
        if not v.startswith('0x'):
            raise ValueError("DID must start with '0x'")
        # Check if valid hex after 0x
        try:
            int(v, 16)
        except ValueError:
            raise ValueError("DID must be a valid hexadecimal string")
        return v.lower()  # Normalize to lowercase
    
    class Config:
        json_schema_extra = {
            "example": {
                "did": "0x0000000000000000000000000000000000000000000000000000000000000000",
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
        }


class PredictResponse(BaseModel):
    """Response model for prediction endpoint"""
    
    confidence_score: int = Field(
        ...,
        ge=0,
        le=100,
        description="Confidence score (0-100)",
        examples=[87]
    )
    anomaly_score: float = Field(
        ...,
        ge=0.0,
        le=1.0,
        description="Anomaly score (0.0-1.0, higher = more anomalous)",
        examples=[0.12]
    )
    feature_importance: Dict[str, float] = Field(
        default_factory=dict,
        description="Feature importance scores"
    )
    model_version: str = Field(
        ...,
        description="Model version used for inference",
        examples=["1.0.0"]
    )
    inference_time_ms: float = Field(
        ...,
        description="Inference time in milliseconds",
        examples=[12.45]
    )
    
    class Config:
        json_schema_extra = {
            "example": {
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
        }


class HealthResponse(BaseModel):
    """Response model for health check endpoint"""
    
    status: str = Field(
        ...,
        description="Service status",
        examples=["healthy"]
    )
    model_loaded: bool = Field(
        ...,
        description="Whether model is loaded",
        examples=[True]
    )
    model_version: str = Field(
        ...,
        description="Model version",
        examples=["1.0.0"]
    )
    device: str = Field(
        default="cpu",
        description="Device (cpu/cuda)",
        examples=["cpu"]
    )
    uptime_seconds: Optional[int] = Field(
        default=None,
        description="Service uptime in seconds",
        examples=[3600]
    )
    
    class Config:
        json_schema_extra = {
            "example": {
                "status": "healthy",
                "model_loaded": True,
                "model_version": "1.0.0",
                "device": "cpu",
                "uptime_seconds": 3600
            }
        }


class ErrorResponse(BaseModel):
    """Error response model"""
    
    detail: str = Field(
        ...,
        description="Error message",
        examples=["Invalid input data"]
    )
    error_code: Optional[str] = Field(
        default=None,
        description="Error code",
        examples=["INVALID_INPUT"]
    )
    path: Optional[str] = Field(
        default=None,
        description="Request path",
        examples=["/predict"]
    )
    
    class Config:
        json_schema_extra = {
            "example": {
                "detail": "Invalid input data",
                "error_code": "INVALID_INPUT",
                "path": "/predict"
            }
        }


class BatchPredictResponse(BaseModel):
    """Response model for batch prediction"""
    
    predictions: List[PredictResponse] = Field(
        ...,
        description="List of prediction results"
    )
    total_count: int = Field(
        ...,
        description="Total number of predictions"
    )
    successful_count: int = Field(
        ...,
        description="Number of successful predictions"
    )
    failed_count: int = Field(
        ...,
        description="Number of failed predictions"
    )