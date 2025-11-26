"""
FastAPI application for behavioral biometrics inference
"""

from fastapi import FastAPI, HTTPException, Request
from fastapi.middleware.cors import CORSMiddleware
from fastapi.responses import JSONResponse
from contextlib import asynccontextmanager
import time
import torch
from typing import Dict, Any
import logging

from app.models import PredictRequest, PredictResponse, HealthResponse
from app.config import settings
from ml.inference import BehavioralInferenceEngine

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)

# Global inference engine
inference_engine = None


@asynccontextmanager
async def lifespan(app: FastAPI):
    """
    Startup and shutdown events
    """
    global inference_engine
    
    # Startup: Load model
    logger.info(f"Loading model from {settings.MODEL_PATH}")
    try:
        inference_engine = BehavioralInferenceEngine(
            model_path=settings.MODEL_PATH,
            scaler_path=settings.SCALER_PATH,
            device=settings.DEVICE,
        )
        logger.info(f" Model loaded successfully on {settings.DEVICE}")
    except Exception as e:
        logger.error(f" Failed to load model: {e}")
        raise
    
    yield
    
    # Shutdown
    logger.info("Shutting down ML service...")
    inference_engine = None


# Create FastAPI app
app = FastAPI(
    title="Behavioral Biometrics ML Service",
    description="Real-time behavioral pattern authentication using deep learning",
    version="1.0.0",
    lifespan=lifespan,
    docs_url="/docs",
    redoc_url="/redoc",
)

# CORS middleware
app.add_middleware(
    CORSMiddleware,
    allow_origins=settings.ALLOWED_ORIGINS,
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)


# Request timing middleware
@app.middleware("http")
async def add_process_time_header(request: Request, call_next):
    start_time = time.time()
    response = await call_next(request)
    process_time = (time.time() - start_time) * 1000  # Convert to ms
    response.headers["X-Process-Time-Ms"] = str(round(process_time, 2))
    return response


# Logging middleware
@app.middleware("http")
async def log_requests(request: Request, call_next):
    logger.info(f"📨 {request.method} {request.url.path}")
    response = await call_next(request)
    logger.info(f"📤 {request.method} {request.url.path} - Status: {response.status_code}")
    return response


@app.get("/", tags=["Root"])
async def root():
    """Root endpoint"""
    return {
        "service": "Behavioral Biometrics ML",
        "version": "1.0.0",
        "status": "running",
        "endpoints": {
            "health": "/health",
            "predict": "/predict",
            "batch_predict": "/batch-predict",
            "metrics": "/metrics",
            "docs": "/docs"
        }
    }


@app.get("/health", response_model=HealthResponse, tags=["Health"])
async def health_check():
    """
    Health check endpoint
    
    Returns service status and model information
    """
    model_loaded = inference_engine is not None
    
    # Get uptime
    import psutil
    import os
    process = psutil.Process(os.getpid())
    uptime_seconds = int(time.time() - process.create_time())
    
    return HealthResponse(
        status="healthy" if model_loaded else "unhealthy",
        model_loaded=model_loaded,
        model_version=settings.MODEL_VERSION,
        device=settings.DEVICE,
        uptime_seconds=uptime_seconds,
    )


@app.post("/predict", response_model=PredictResponse, tags=["Inference"])
async def predict(request: PredictRequest):
    """
    Predict confidence score for behavioral pattern
    
    Args:
        request: Prediction request containing DID and features
        
    Returns:
        Confidence score (0-100) and additional metrics
        
    Raises:
        HTTPException: If model not loaded or invalid input
    """
    if inference_engine is None:
        logger.error("Model not loaded")
        raise HTTPException(status_code=503, detail="Model not loaded")
    
    try:
        start_time = time.time()
        
        logger.debug(f"Processing prediction for DID: {request.did}")
        
        # Run inference
        result = inference_engine.predict(
            features=request.features.model_dump(),
            historical_patterns=[p.model_dump() for p in request.historical_patterns]
            if request.historical_patterns
            else None,
        )
        
        inference_time_ms = (time.time() - start_time) * 1000
        
        logger.info(
            f"Prediction for {request.did[:10]}...: "
            f"confidence={result['confidence_score']}, "
            f"time={inference_time_ms:.2f}ms"
        )
        
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


@app.post("/batch-predict", tags=["Inference"])
async def batch_predict(requests: list[PredictRequest]):
    """
    Batch prediction endpoint for multiple patterns
    
    Args:
        requests: List of prediction requests
        
    Returns:
        List of prediction responses
        
    Raises:
        HTTPException: If batch too large or model not loaded
    """
    if inference_engine is None:
        raise HTTPException(status_code=503, detail="Model not loaded")
    
    if len(requests) > settings.MAX_BATCH_SIZE:
        raise HTTPException(
            status_code=400,
            detail=f"Maximum {settings.MAX_BATCH_SIZE} requests per batch"
        )
    
    logger.info(f"Processing batch of {len(requests)} requests")
    
    results = []
    for idx, req in enumerate(requests):
        try:
            result = inference_engine.predict(
                features=req.features.model_dump(),
                historical_patterns=[p.model_dump() for p in req.historical_patterns]
                if req.historical_patterns
                else None,
            )
            result["model_version"] = settings.MODEL_VERSION
            results.append(result)
        except Exception as e:
            logger.error(f"Error processing request {idx}: {e}")
            results.append({
                "error": str(e),
                "confidence_score": 0,
                "anomaly_score": 1.0,
            })
    
    return results


@app.get("/metrics", tags=["Monitoring"])
async def metrics():
    """
    Prometheus-style metrics endpoint
    
    Returns performance and usage metrics
    """
    if inference_engine is None:
        return JSONResponse(
            status_code=503,
            content={"error": "Model not loaded"}
        )
    
    stats = inference_engine.get_stats()
    
    # Format as Prometheus metrics
    metrics_text = f"""# HELP prediction_count Total number of predictions
    # TYPE prediction_count counter
    prediction_count {stats['total_predictions']}

    # HELP average_inference_time_ms Average inference time in milliseconds
    # TYPE average_inference_time_ms gauge
    average_inference_time_ms {stats['avg_inference_time_ms']}

    # HELP average_confidence Average confidence score
    # TYPE average_confidence gauge
    average_confidence {stats['avg_confidence']}

    # HELP model_version Current model version
    # TYPE model_version gauge
    model_version{{version="{settings.MODEL_VERSION}"}} 1
    """
    
    return JSONResponse(content={"metrics": metrics_text})


@app.get("/stats", tags=["Monitoring"])
async def stats():
    """
    Get detailed statistics in JSON format
    """
    if inference_engine is None:
        return JSONResponse(
            status_code=503,
            content={"error": "Model not loaded"}
        )
    
    stats = inference_engine.get_stats()
    
    return {
        "total_predictions": stats['total_predictions'],
        "avg_inference_time_ms": stats['avg_inference_time_ms'],
        "avg_confidence": stats['avg_confidence'],
        "model_version": settings.MODEL_VERSION,
        "device": settings.DEVICE,
    }


# Error handlers
@app.exception_handler(404)
async def not_found_handler(request: Request, exc):
    return JSONResponse(
        status_code=404,
        content={
            "detail": "Endpoint not found",
            "path": str(request.url.path)
        },
    )


@app.exception_handler(500)
async def internal_error_handler(request: Request, exc):
    logger.error(f"Internal server error: {exc}")
    return JSONResponse(
        status_code=500,
        content={"detail": "Internal server error"},
    )


@app.exception_handler(Exception)
async def general_exception_handler(request: Request, exc: Exception):
    logger.error(f"Unhandled exception: {exc}", exc_info=True)
    return JSONResponse(
        status_code=500,
        content={"detail": "An unexpected error occurred"},
    )


if __name__ == "__main__":
    import uvicorn
    
    uvicorn.run(
        "app.main:app",
        host=settings.HOST,
        port=settings.PORT,
        reload=settings.DEBUG,
        workers=1 if settings.DEBUG else settings.WORKERS,
        log_level="info",
    )