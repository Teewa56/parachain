"""
Inference route handlers
"""

from fastapi import APIRouter, HTTPException, Depends
from typing import List
import time
import logging

from app.models import PredictRequest, PredictResponse, BatchPredictResponse
from app.config import settings

logger = logging.getLogger(__name__)

router = APIRouter(prefix="/api/v1", tags=["Inference"])


# Dependency to get inference engine
def get_inference_engine():
    """Dependency to access global inference engine"""
    from app.main import inference_engine
    if inference_engine is None:
        raise HTTPException(status_code=503, detail="Model not loaded")
    return inference_engine


@router.post("/predict", response_model=PredictResponse)
async def predict_single(
    request: PredictRequest,
    engine=Depends(get_inference_engine)
):
    """
    Predict confidence score for a single behavioral pattern
    
    Args:
        request: Prediction request with DID and features
        engine: Inference engine (injected)
        
    Returns:
        Prediction response with confidence score
    """
    try:
        start_time = time.time()
        
        logger.info(f"Processing prediction for DID: {request.did[:10]}...")
        
        # Run inference
        result = engine.predict(
            features=request.features.model_dump(),
            historical_patterns=[p.model_dump() for p in request.historical_patterns]
            if request.historical_patterns
            else None,
        )
        
        inference_time_ms = (time.time() - start_time) * 1000
        
        return PredictResponse(
            confidence_score=result["confidence_score"],
            anomaly_score=result.get("anomaly_score", 0.0),
            feature_importance=result.get("feature_importance", {}),
            model_version=settings.MODEL_VERSION,
            inference_time_ms=round(inference_time_ms, 2),
        )
        
    except ValueError as e:
        logger.warning(f"Validation error: {e}")
        raise HTTPException(status_code=400, detail=str(e))
    except Exception as e:
        logger.error(f"Inference error: {e}")
        raise HTTPException(status_code=500, detail=f"Inference error: {str(e)}")


@router.post("/batch-predict", response_model=BatchPredictResponse)
async def predict_batch(
    requests: List[PredictRequest],
    engine=Depends(get_inference_engine)
):
    """
    Batch prediction for multiple patterns
    
    Args:
        requests: List of prediction requests
        engine: Inference engine (injected)
        
    Returns:
        Batch prediction response
    """
    if len(requests) > settings.MAX_BATCH_SIZE:
        raise HTTPException(
            status_code=400,
            detail=f"Maximum {settings.MAX_BATCH_SIZE} requests per batch"
        )
    
    logger.info(f"Processing batch of {len(requests)} requests")
    
    predictions = []
    successful = 0
    failed = 0
    
    for idx, req in enumerate(requests):
        try:
            start_time = time.time()
            
            result = engine.predict(
                features=req.features.model_dump(),
                historical_patterns=[p.model_dump() for p in req.historical_patterns]
                if req.historical_patterns
                else None,
            )
            
            inference_time_ms = (time.time() - start_time) * 1000
            
            predictions.append(PredictResponse(
                confidence_score=result["confidence_score"],
                anomaly_score=result.get("anomaly_score", 0.0),
                feature_importance=result.get("feature_importance", {}),
                model_version=settings.MODEL_VERSION,
                inference_time_ms=round(inference_time_ms, 2),
            ))
            successful += 1
            
        except Exception as e:
            logger.error(f"Error processing request {idx}: {e}")
            # Add error response
            predictions.append(PredictResponse(
                confidence_score=0,
                anomaly_score=1.0,
                feature_importance={},
                model_version=settings.MODEL_VERSION,
                inference_time_ms=0.0,
            ))
            failed += 1
    
    return BatchPredictResponse(
        predictions=predictions,
        total_count=len(requests),
        successful_count=successful,
        failed_count=failed,
    )


@router.post("/compare", response_model=dict)
async def compare_patterns(
    pattern1: PredictRequest,
    pattern2: PredictRequest,
    engine=Depends(get_inference_engine)
):
    """
    Compare two behavioral patterns
    
    Args:
        pattern1: First pattern
        pattern2: Second pattern
        engine: Inference engine
        
    Returns:
        Comparison result with similarity score
    """
    try:
        # Get predictions for both patterns
        result1 = engine.predict(
            features=pattern1.features.model_dump(),
            historical_patterns=None,
        )
        
        result2 = engine.predict(
            features=pattern2.features.model_dump(),
            historical_patterns=None,
        )
        
        # Calculate similarity (inverse of distance)
        import numpy as np
        
        features1 = np.array([
            pattern1.features.typing_speed_wpm,
            pattern1.features.avg_key_hold_time_ms,
            pattern1.features.avg_transition_time_ms,
            pattern1.features.error_rate_percent,
            pattern1.features.activity_hour_preference,
        ])
        
        features2 = np.array([
            pattern2.features.typing_speed_wpm,
            pattern2.features.avg_key_hold_time_ms,
            pattern2.features.avg_transition_time_ms,
            pattern2.features.error_rate_percent,
            pattern2.features.activity_hour_preference,
        ])
        
        # Euclidean distance
        distance = np.linalg.norm(features1 - features2)
        
        # Convert to similarity (0-100)
        max_distance = 300  # Approximate maximum
        similarity = max(0, 100 - (distance / max_distance * 100))
        
        return {
            "pattern1_confidence": result1["confidence_score"],
            "pattern2_confidence": result2["confidence_score"],
            "similarity_score": round(similarity, 2),
            "distance": round(float(distance), 2),
            "likely_same_user": similarity > 70,
        }
        
    except Exception as e:
        logger.error(f"Comparison error: {e}")
        raise HTTPException(status_code=500, detail=str(e))