#!/bin/bash

OPENCV_VERSION=$(ls -1 /opt/homebrew/Cellar/opencv/ 2>/dev/null | head -1)
if [ -z "$OPENCV_VERSION" ]; then
    echo "Error: OpenCV not found. Install with: brew install opencv"
    exit 1
fi

OPENCV_LIB_PATH=/opt/homebrew/Cellar/opencv/$OPENCV_VERSION/lib

export OPENCV_INCLUDE_PATHS=/opt/homebrew/Cellar/opencv/$OPENCV_VERSION/include/opencv4
export OPENCV_LINK_PATHS=$OPENCV_LIB_PATH
export OPENCV_LINK_LIBS=opencv_core,opencv_highgui,opencv_imgproc,opencv_videoio

BUILD_LIB_PATH=""
if [ -d "/opt/homebrew/opt/llvm/lib" ]; then
    BUILD_LIB_PATH="/opt/homebrew/opt/llvm/lib"
fi

RUNTIME_LIB_PATHS="/opt/homebrew/lib:$OPENCV_LIB_PATH"
for lib_dir in /opt/homebrew/Cellar/*/lib; do
    if [ -d "$lib_dir" ] && [[ ! "$lib_dir" =~ llvm ]] && [[ ! "$RUNTIME_LIB_PATHS" =~ "$lib_dir" ]]; then
        RUNTIME_LIB_PATHS="$RUNTIME_LIB_PATHS:$lib_dir"
    fi
done
for lib_dir in /opt/homebrew/opt/*/lib; do
    if [ -d "$lib_dir" ] && [[ ! "$lib_dir" =~ llvm ]] && [[ ! "$RUNTIME_LIB_PATHS" =~ "$lib_dir" ]]; then
        RUNTIME_LIB_PATHS="$RUNTIME_LIB_PATHS:$lib_dir"
    fi
done

CAMERA_ID=${1:-0}
CONFIDENCE=${2:-0.5}

echo "Building tracking output test..."
if [ -n "$BUILD_LIB_PATH" ]; then
    export DYLD_FALLBACK_LIBRARY_PATH=$BUILD_LIB_PATH:$DYLD_FALLBACK_LIBRARY_PATH
fi
cargo build --bin tracking_output_test --features opencv,serde_support --release
BUILD_STATUS=$?

unset DYLD_FALLBACK_LIBRARY_PATH
unset DYLD_LIBRARY_PATH

if [ $BUILD_STATUS -eq 0 ]; then
    echo ""
    echo "Running tracking output test (camera_id=$CAMERA_ID, confidence=$CONFIDENCE)..."
    echo ""
    export DYLD_FALLBACK_LIBRARY_PATH=$RUNTIME_LIB_PATHS:$DYLD_FALLBACK_LIBRARY_PATH
    export DYLD_LIBRARY_PATH=$RUNTIME_LIB_PATHS:$DYLD_LIBRARY_PATH
    ./target/release/tracking_output_test $CAMERA_ID $CONFIDENCE
else
    echo "Build failed!"
    exit 1
fi

