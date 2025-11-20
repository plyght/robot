#!/bin/bash

echo "========================================="
echo "  Setting up Apple Depth Pro"
echo "========================================="
echo ""

if ! command -v python3 &> /dev/null; then
    echo "❌ Python 3 not found. Please install Python 3.9+"
    exit 1
fi

PYTHON_VERSION=$(python3 --version | awk '{print $2}' | cut -d. -f1,2)
echo "Detected Python version: $PYTHON_VERSION"

if [[ "$PYTHON_VERSION" == "3.14" ]]; then
    echo "Python 3.14 detected - using compatible numpy version..."
fi

echo "Creating virtual environment..."
python3 -m venv venv_depth_pro

echo "Activating virtual environment..."
source venv_depth_pro/bin/activate

echo ""
echo "Installing dependencies..."
pip install --upgrade pip

echo ""
echo "Installing Apple Depth Pro..."

PYTHON_VERSION=$(python3 --version | awk '{print $2}' | cut -d. -f1,2)
if [[ "$PYTHON_VERSION" == "3.14" ]]; then
    echo "Fixing numpy compatibility for Python 3.14..."
    pip install numpy==2.2.0
    pip install git+https://github.com/apple/ml-depth-pro.git
else
    pip install git+https://github.com/apple/ml-depth-pro.git
fi

echo ""
echo "Downloading pretrained model..."
mkdir -p checkpoints
cd checkpoints

if [ ! -f "depth_pro.pt" ]; then
    echo "Downloading depth_pro.pt..."
    curl -L -o depth_pro.pt "https://ml-site.cdn-apple.com/models/depth-pro/depth_pro.pt"
else
    echo "Model already exists"
fi

cd ..

echo ""
echo "========================================="
echo "✅ Depth Pro installed successfully!"
echo "========================================="
echo ""
echo "Test it:"
echo "  source venv_depth_pro/bin/activate"
echo "  python3 depth_service.py test data/example.jpg"
echo ""
echo "Or use in robot system:"
echo "  cargo run --bin depth_integration_test --features opencv"
echo ""

