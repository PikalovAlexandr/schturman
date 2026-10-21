#!/bin/bash

# Kill existing services
echo "Stopping existing services..."
pkill -9 -f 'schturman-auth-service' || true
pkill -9 -f 'part-service' || true
pkill -9 -f 'document-service' || true
pkill -9 -f 'bom-service' || true
pkill -9 -f 'change-service' || true
pkill -9 -f 'ai-orchestrator' || true

sleep 2

# Environment variables
export DATABASE_URL="postgresql://schturman:schturman_password@localhost:5434/schturman"
export REDIS_URL="redis://localhost:6379"
export JWT_SECRET="test_secret_key_12345"
export JWT_REFRESH_SECRET="test_refresh_secret_67890"
export STORAGE_PATH="/tmp/schturman-documents"

BIN_DIR="/mnt/storage/Штурман/target/release"

# Start services in background
echo "Starting auth-service on :3001..."
$BIN_DIR/schturman-auth-service > /tmp/auth-service.log 2>&1 &

echo "Starting part-service on :3002..."
$BIN_DIR/part-service > /tmp/part-service.log 2>&1 &

echo "Starting document-service on :3003..."
$BIN_DIR/document-service > /tmp/document-service.log 2>&1 &

echo "Starting bom-service on :3004..."
$BIN_DIR/bom-service > /tmp/bom-service.log 2>&1 &

echo "Starting change-service on :3005..."
$BIN_DIR/change-service > /tmp/change-service.log 2>&1 &

echo "Starting ai-orchestrator on :3006..."
$BIN_DIR/ai-orchestrator > /tmp/ai-orchestrator.log 2>&1 &

sleep 3

echo ""
echo "Service status:"
for port in 3001 3002 3003 3004 3005 3006; do
  if nc -z localhost $port 2>/dev/null; then
    echo "✓ Port $port is active"
  else
    echo "✗ Port $port is NOT responding"
  fi
done

echo ""
echo "Logs available at /tmp/*-service.log"
