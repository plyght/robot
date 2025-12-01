#!/bin/bash
set -e

echo "Converting TFLite hand model to ONNX..."
echo "This only needs to be run ONCE"
echo ""

if ! command -v python3 &> /dev/null; then
    echo "ERROR: python3 not found"
    exit 1
fi

echo "Installing conversion tool (one-time)..."
python3 -m pip install --break-system-packages -q tf2onnx tensorflow

echo "Converting models/hand_landmarker.tflite → models/hand_landmarker.onnx..."
python3 -m tf2onnx.convert \
    --tflite models/hand_landmarker.tflite \
    --output models/hand_landmarker.onnx \
    --opset 13

if [ -s models/hand_landmarker.onnx ]; then
    echo ""
    echo "✅ SUCCESS! ONNX model created:"
    ls -lh models/hand_landmarker.onnx
    echo ""
    echo "You can now run: cargo run --bin llm_robot_test --features opencv --release"
else
    echo ""
    echo "❌ Conversion failed. Trying alternative method..."
    echo ""
    echo "Downloading pre-converted model instead..."

    # Try MediaPipe Palm Detection model (simpler, more likely to work)
    wget -q https://github.com/PINTO0309/PINTO_model_zoo/raw/main/033_Hand_Detection_and_Tracking/01_hand_landmark/hand_landmark_192x192.onnx \
        -O models/hand_landmarker.onnx 2>/dev/null || true

    if [ -s models/hand_landmarker.onnx ]; then
        echo "✅ Downloaded alternative hand tracking model"
        ls -lh models/hand_landmarker.onnx
    else
        echo "❌ Could not obtain ONNX model"
        echo ""
        echo "WORKAROUND: Run with hand tracking disabled:"
        echo "Edit src/bin/llm_robot_test.rs line 38:"
        echo "  enable_hand_tracking: false,"
        exit 1
    fi
fi

echo ""
echo "Done! The .onnx file will now work with pure Rust (no Python needed at runtime)"
