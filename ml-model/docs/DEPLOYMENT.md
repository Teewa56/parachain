Deployment Guide
Production Deployment Guide
Complete guide for deploying the ML service to production.
Prerequisites

Docker & Docker Compose
2+ CPU cores
2GB+ RAM
10GB+ storage

Quick Start
1. Build Docker Image
bashdocker build -f docker/Dockerfile -t behavioral-ml:1.0.0 .
2. Start Services
bashdocker-compose -f docker/docker-compose.yml up -d
3. Verify Deployment
bashcurl http://localhost:8000/health
Production Configuration
Environment Variables
Create .env file:
bashMODEL_PATH=/app/models/production/model.pth
SCALER_PATH=/app/models/production/scaler.pkl
MODEL_VERSION=1.0.0
DEVICE=cpu
HOST=0.0.0.0
PORT=8000
WORKERS=4
DEBUG=false
ALLOWED_ORIGINS=["https://yourdomain.com"]
MAX_BATCH_SIZE=100
REQUEST_TIMEOUT=30
Resource Limits
In docker-compose.yml:
yamldeploy:
  resources:
    limits:
      cpus: '2.0'
      memory: 2G
    reservations:
      cpus: '1.0'
      memory: 1G
Kubernetes Deployment
Deployment YAML
yamlapiVersion: apps/v1
kind: Deployment
metadata:
  name: behavioral-ml
spec:
  replicas: 3
  selector:
    matchLabels:
      app: behavioral-ml
  template:
    metadata:
      labels:
        app: behavioral-ml
    spec:
      containers:
      - name: ml-service
        image: behavioral-ml:1.0.0
        ports:
        - containerPort: 8000
        resources:
          limits:
            cpu: "1000m"
            memory: "1Gi"
          requests:
            cpu: "500m"
            memory: "512Mi"
        livenessProbe:
          httpGet:
            path: /live
            port: 8000
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 8000
          initialDelaySeconds: 10
          periodSeconds: 5
Monitoring
Prometheus Setup

Deploy Prometheus:

bashdocker-compose up prometheus

Access dashboard:

http://localhost:9090
Grafana Dashboard

Deploy Grafana:

bashdocker-compose up grafana

Import dashboard from grafana/dashboard.json

Key Metrics to Monitor

Request rate
Latency (P50, P95, P99)
Error rate
Model confidence distribution
CPU/Memory usage

Scaling
Horizontal Scaling
bashdocker-compose up --scale ml-service=4
Load Balancing
Use nginx or cloud load balancer:
nginxupstream ml_backend {
    least_conn;
    server ml-service-1:8000;
    server ml-service-2:8000;
    server ml-service-3:8000;
}
Security
Best Practices

API Authentication: Implement API keys
Rate Limiting: Prevent abuse
Input Validation: Strict validation
HTTPS Only: TLS encryption
Network Isolation: Private subnet

Firewall Rules
bash# Allow only necessary ports
ufw allow 8000/tcp  # API
ufw allow 9090/tcp  # Prometheus
ufw deny 22/tcp     # SSH (use bastion)
Backup & Recovery
Model Backup
bash# Backup models
tar -czf models-backup-$(date +%Y%m%d).tar.gz models/

# Restore
tar -xzf models-backup-20240115.tar.gz
Database Backup
If using database for logging:
bashpg_dump behavioral_logs > backup.sql
Troubleshooting
Model Not Loading
bash# Check model file
docker exec ml-service ls -lh /app/models/production/

# Check logs
docker logs ml-service
High Latency
bash# Check CPU usage
docker stats ml-service

# Scale up
docker-compose up --scale ml-service=6
Memory Issues
bash# Increase memory limit
docker-compose up -d --force-recreate
Rollback Plan
bash# Tag current version
docker tag behavioral-ml:latest behavioral-ml:stable

# Deploy new version
docker-compose up -d

# If issues, rollback
docker tag behavioral-ml:stable behavioral-ml:latest
docker-compose up -d --force-recreate
Maintenance
Model Updates

Train new model
Validate performance
Deploy to staging
A/B test (50/50 split)
Gradual rollout
Monitor metrics

Log Rotation
bash# Configure logrotate
/var/log/behavioral-ml/*.log {
    daily
    rotate 7
    compress
    delaycompress
    notifempty
}