#!/bin/bash

echo "========================================="
echo "  Warming up Core ML Depth Pro Model"
echo "========================================="
echo ""
echo "This will preload the model so it's ready when you need it."
echo "The model will stay loaded in memory (~1-2GB RAM)."
echo ""
echo "Once loaded, the service stays running and ready."
echo "Press Ctrl+C to stop the warmup service."
echo ""

PYTHON_PATH="venv_depth_pro_system/bin/python3"
if [ ! -f "$PYTHON_PATH" ]; then
    PYTHON_PATH="venv_depth_pro/bin/python3"
fi

if [ ! -f "$PYTHON_PATH" ]; then
    echo "Error: Python environment not found. Run ./setup_depth_pro.sh first."
    exit 1
fi

echo "Starting depth service with model preload..."
echo "Model loading may take 2-5 minutes..."
echo ""

# Start the service and capture its PID
$PYTHON_PATH depth_service.py &
DEPTH_PID=$!

echo "Depth service PID: $DEPTH_PID"
echo ""
echo "Waiting for model to load..."
echo "(You'll see 'Depth Pro ready!' when it's loaded)"
echo ""
echo "Once ready, the service will stay running."
echo "You can now use the camera/YOLO integration - it will connect instantly!"
echo ""

# Wait for the process (it stays running)
wait $DEPTH_PID

