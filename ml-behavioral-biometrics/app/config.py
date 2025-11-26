"""
Configuration management for FastAPI application
"""

from pydantic_settings import BaseSettings
from typing import List


class Settings(BaseSettings):
    """Application settings"""
    
    # Server
    HOST: str = "0.0.0.0"
    PORT: int = 8000
    WORKERS: int = 4
    DEBUG: bool = False
    
    # Model
    MODEL_PATH: str = "models/production/model.pth"
    SCALER_PATH: str = "models/production/scaler.pkl"
    MODEL_VERSION: str = "1.0.0"
    DEVICE: str = "cpu"  # or "cuda"
    
    # CORS
    ALLOWED_ORIGINS: List[str] = ["*"]
    
    # Performance
    MAX_BATCH_SIZE: int = 100
    REQUEST_TIMEOUT: int = 30
    
    # Monitoring
    ENABLE_METRICS: bool = True
    
    class Config:
        env_file = ".env"
        case_sensitive = True


# Global settings instance
settings = Settings()