"""
Health check and monitoring route handlers
"""

from fastapi import APIRouter
from fastapi.responses import JSONResponse
import time
import logging

from app.models import HealthResponse
from app.config import settings

logger = logging.getLogger(__name__)

router = APIRouter(tags=["Health"])


@router.get("/health", response_model=HealthResponse)
async def health_check():
    """
    Health check endpoint
    
    Returns service status and model information
    """
    from app.main import inference_engine
    
    model_loaded = inference_engine is not None
    
    # Get uptime
    try:
        import psutil
        import os
        process = psutil.Process(os.getpid())
        uptime_seconds = int(time.time() - process.create_time())
    except Exception:
        uptime_seconds = None
    
    return HealthResponse(
        status="healthy" if model_loaded else "unhealthy",
        model_loaded=model_loaded,
        model_version=settings.MODEL_VERSION,
        device=settings.DEVICE,
        uptime_seconds=uptime_seconds,
    )


@router.get("/ready")
async def readiness_probe():
    """
    Kubernetes readiness probe
    
    Returns 200 if service is ready to accept traffic
    """
    from app.main import inference_engine
    
    if inference_engine is None:
        return JSONResponse(
            status_code=503,
            content={"ready": False, "reason": "Model not loaded"}
        )
    
    return {"ready": True}


@router.get("/live")
async def liveness_probe():
    """
    Kubernetes liveness probe
    
    Returns 200 if service is alive
    """
    return {"alive": True}


@router.get("/metrics")
async def metrics():
    """
    Prometheus-style metrics endpoint
    
    Returns performance and usage metrics
    """
    from app.main import inference_engine
    
    if inference_engine is None:
        return JSONResponse(
            status_code=503,
            content={"error": "Model not loaded"}
        )
    
    try:
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

# HELP model_loaded Model loading status
# TYPE model_loaded gauge
model_loaded 1
"""
        
        return JSONResponse(
            content={"metrics": metrics_text},
            media_type="text/plain"
        )
        
    except Exception as e:
        logger.error(f"Error getting metrics: {e}")
        return JSONResponse(
            status_code=500,
            content={"error": str(e)}
        )


@router.get("/stats")
async def detailed_stats():
    """
    Get detailed statistics in JSON format
    """
    from app.main import inference_engine
    
    if inference_engine is None:
        return JSONResponse(
            status_code=503,
            content={"error": "Model not loaded"}
        )
    
    try:
        stats = inference_engine.get_stats()
        
        # Add system stats
        try:
            import psutil
            cpu_percent = psutil.cpu_percent(interval=1)
            memory = psutil.virtual_memory()
            
            system_stats = {
                "cpu_percent": cpu_percent,
                "memory_used_mb": memory.used / (1024 * 1024),
                "memory_available_mb": memory.available / (1024 * 1024),
                "memory_percent": memory.percent,
            }
        except Exception:
            system_stats = {}
        
        return {
            "model": {
                "version": settings.MODEL_VERSION,
                "device": settings.DEVICE,
                "loaded": True,
            },
            "performance": {
                "total_predictions": stats['total_predictions'],
                "avg_inference_time_ms": stats['avg_inference_time_ms'],
                "avg_confidence": stats['avg_confidence'],
            },
            "system": system_stats,
        }
        
    except Exception as e:
        logger.error(f"Error getting stats: {e}")
        return JSONResponse(
            status_code=500,
            content={"error": str(e)}
        )


@router.get("/info")
async def service_info():
    """
    Get service information
    """
    return {
        "service": "Behavioral Biometrics ML",
        "version": "1.0.0",
        "model_version": settings.MODEL_VERSION,
        "device": settings.DEVICE,
        "max_batch_size": settings.MAX_BATCH_SIZE,
        "endpoints": {
            "health": "/health",
            "ready": "/ready",
            "live": "/live",
            "predict": "/predict",
            "batch_predict": "/batch-predict",
            "metrics": "/metrics",
            "stats": "/stats",
            "docs": "/docs",
        }
    }