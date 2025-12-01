#!/bin/bash

OPENCV_VERSION=$(ls -1 /opt/homebrew/Cellar/opencv/ 2>/dev/null | head -1)
if [ -z "$OPENCV_VERSION" ]; then
    echo "Error: OpenCV not found. Install with: brew install opencv"
    exit 1
fi

OPENCV_LIB_PATH=/opt/homebrew/Cellar/opencv/$OPENCV_VERSION/lib

export OPENCV_INCLUDE_PATHS=/opt/homebrew/Cellar/opencv/$OPENCV_VERSION/include/opencv4
export OPENCV_LINK_PATHS=$OPENCV_LIB_PATH
export OPENCV_LINK_LIBS=opencv_core,opencv_highgui,opencv_imgproc,opencv_videoio,opencv_imgcodecs

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

CAMERA_ID=0
MODE=""

while [[ $# -gt 0 ]]; do
    case $1 in
        --stream|-s)
            MODE="--stream"
            shift
            ;;
        *)
            CAMERA_ID=$1
            shift
            ;;
    esac
done

echo "Building depth integration test..."
if [ -n "$BUILD_LIB_PATH" ]; then
    export DYLD_FALLBACK_LIBRARY_PATH=$BUILD_LIB_PATH:$DYLD_FALLBACK_LIBRARY_PATH
fi
cargo build --bin depth_integration_test --features opencv --release
BUILD_STATUS=$?

unset DYLD_FALLBACK_LIBRARY_PATH
unset DYLD_LIBRARY_PATH

if [ $BUILD_STATUS -eq 0 ]; then
    echo ""
    echo "========================================="
    echo "  Starting Depth Pro Integration Test"
    echo "========================================="
    echo ""
    if [ "$MODE" = "--stream" ]; then
        echo "Mode: ⚡ CONTINUOUS STREAM (robot mode)"
        echo "      Updates depth as fast as possible"
        echo "      ~2 Hz continuous updates"
    else
        echo "Mode: MANUAL (use --stream for robot mode)"
    fi
    echo "Camera ID: $CAMERA_ID"
    echo ""
    echo "Instructions:"
    echo "  - Wait for camera and YOLO to load"
    echo "  - Depth Pro runs in background (non-blocking)"
    echo "  - Press 'q' to quit"
    echo ""
    export DYLD_FALLBACK_LIBRARY_PATH=$RUNTIME_LIB_PATHS:$DYLD_FALLBACK_LIBRARY_PATH
    export DYLD_LIBRARY_PATH=$RUNTIME_LIB_PATHS:$DYLD_LIBRARY_PATH
    ./target/release/depth_integration_test $CAMERA_ID $MODE
else
    echo "Build failed!"
    exit 1
fi

