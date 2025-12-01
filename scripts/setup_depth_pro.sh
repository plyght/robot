#!/bin/bash

echo "========================================="
echo "  Setting up Core ML Depth Pro"
echo "========================================="
echo ""

if ! command -v python3 &> /dev/null; then
    echo "❌ Python 3 not found. Please install Python 3.9+"
    exit 1
fi

PYTHON_CMD="python3"
if command -v /usr/bin/python3 &> /dev/null; then
    PYTHON_VERSION=$(/usr/bin/python3 --version 2>&1 | awk '{print $2}' | cut -d. -f1,2)
    if [[ "$PYTHON_VERSION" == "3.9"* ]] || [[ "$PYTHON_VERSION" == "3.10"* ]] || [[ "$PYTHON_VERSION" == "3.11"* ]]; then
        PYTHON_CMD="/usr/bin/python3"
        echo "Using system Python $PYTHON_VERSION (better Core ML compatibility)"
    else
        PYTHON_VERSION=$(python3 --version | awk '{print $2}' | cut -d. -f1,2)
        echo "Detected Python version: $PYTHON_VERSION"
        echo "Note: Python 3.9-3.11 recommended for best Core ML support"
    fi
else
    PYTHON_VERSION=$(python3 --version | awk '{print $2}' | cut -d. -f1,2)
    echo "Detected Python version: $PYTHON_VERSION"
fi

echo "Creating virtual environment with $PYTHON_CMD..."
$PYTHON_CMD -m venv venv_depth_pro

echo "Activating virtual environment..."
source venv_depth_pro/bin/activate

echo ""
echo "Installing dependencies..."
pip install --upgrade pip

echo ""
echo "Installing Core ML tools..."
pip install coremltools pillow numpy

echo ""
echo "Downloading Core ML Depth Pro model from Hugging Face..."
MODEL_DIR="$HOME/coreml-depthpro"
mkdir -p "$MODEL_DIR"

if [ ! -d "$MODEL_DIR/DepthProNormalizedInverseDepthPruned10QuantizedLinear.mlpackage" ]; then
    echo "Installing Hugging Face CLI..."
    if ! command -v huggingface-cli &> /dev/null; then
        pip install huggingface_hub[cli]
    fi
    
    echo "Downloading model to $MODEL_DIR (this may take a few minutes, ~5GB)..."
    source venv_depth_pro/bin/activate
    pip install huggingface_hub > /dev/null 2>&1
    python3 -c "
from huggingface_hub import snapshot_download
import os
model_dir = os.path.expanduser('~/coreml-depthpro')
os.makedirs(model_dir, exist_ok=True)
snapshot_download(
    repo_id='KeighBee/coreml-DepthPro',
    local_dir=model_dir,
    local_dir_use_symlinks=False,
    allow_patterns='DepthProNormalizedInverseDepthPruned10QuantizedLinear.mlpackage/**'
)
print('Model downloaded successfully!')
"
    
    if [ ! -d "$MODEL_DIR/DepthProNormalizedInverseDepthPruned10QuantizedLinear.mlpackage" ]; then
        echo "❌ Failed to download model. Trying alternative method..."
        echo "Downloading directly..."
        cd "$MODEL_DIR"
        git lfs install || echo "Git LFS not available, trying without..."
        git clone https://huggingface.co/KeighBee/coreml-DepthPro
        if [ -d "coreml-DepthPro/DepthProNormalizedInverseDepthPruned10QuantizedLinear.mlpackage" ]; then
            mv coreml-DepthPro/DepthProNormalizedInverseDepthPruned10QuantizedLinear.mlpackage .
            rm -rf coreml-DepthPro
        fi
        cd - > /dev/null
    fi
else
    echo "Model already exists at $MODEL_DIR"
fi

if [ ! -d "$MODEL_DIR/DepthProNormalizedInverseDepthPruned10QuantizedLinear.mlpackage" ]; then
    echo ""
    echo "❌ Model download failed!"
    echo ""
    echo "Please download manually:"
    echo "  1. Install: pip install huggingface_hub"
    echo "  2. Run: python3 -c \"from huggingface_hub import snapshot_download; import os; snapshot_download(repo_id='KeighBee/coreml-DepthPro', local_dir=os.path.expanduser('~/coreml-depthpro'), allow_patterns='DepthProNormalizedInverseDepthPruned10QuantizedLinear.mlpackage/**')\""
    echo ""
    echo "Or visit: https://huggingface.co/KeighBee/coreml-DepthPro/tree/main"
    exit 1
fi

echo ""
echo "========================================="
echo "✅ Core ML Depth Pro installed successfully!"
echo "========================================="
echo ""
echo "Test it:"
echo "  source venv_depth_pro/bin/activate"
echo "  python3 depth_service.py test <image_path>"
echo ""
echo "Or use in robot system:"
echo "  cargo run --bin depth_integration_test --features opencv"
echo ""

