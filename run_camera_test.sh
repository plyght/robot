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

# Build-time: Only set what's needed for OpenCV build script (libclang)
# Don't set DYLD_LIBRARY_PATH during build to avoid conflicts with Rust's LLVM
BUILD_LIB_PATH=""
if [ -d "/opt/homebrew/opt/llvm/lib" ]; then
    BUILD_LIB_PATH="/opt/homebrew/opt/llvm/lib"
fi

# Find protobuf 33 library path (required by OpenCV DNN)
PROTOBUF_33_PATH=$(find /opt/homebrew/Cellar/protobuf/33.0 -name "lib" -type d 2>/dev/null | head -1)
if [ -z "$PROTOBUF_33_PATH" ]; then
    PROTOBUF_33_PATH=/opt/homebrew/Cellar/protobuf/33.0/lib
fi

# Runtime: Collect all Homebrew library directories (excluding llvm to avoid Rust conflicts)
# Include both Cellar paths and opt symlinks (OpenCV uses @@HOMEBREW_PREFIX@@/opt paths)
# Add /opt/homebrew/lib first (contains symlinks to all Homebrew libraries)
RUNTIME_LIB_PATHS="/opt/homebrew/lib:$OPENCV_LIB_PATH"
# Add protobuf 33 (highest priority)
if [ -d "$PROTOBUF_33_PATH" ]; then
    RUNTIME_LIB_PATHS="$PROTOBUF_33_PATH:$RUNTIME_LIB_PATHS"
fi
for lib_dir in /opt/homebrew/Cellar/*/lib; do
    if [ -d "$lib_dir" ] && [[ ! "$lib_dir" =~ llvm ]] && [[ ! "$lib_dir" =~ protobuf ]] && [[ ! "$RUNTIME_LIB_PATHS" =~ "$lib_dir" ]]; then
        RUNTIME_LIB_PATHS="$RUNTIME_LIB_PATHS:$lib_dir"
    fi
done
# Also add opt symlinks (OpenCV expects @@HOMEBREW_PREFIX@@/opt/*/lib paths)
for lib_dir in /opt/homebrew/opt/*/lib; do
    if [ -d "$lib_dir" ] && [[ ! "$lib_dir" =~ llvm ]] && [[ ! "$RUNTIME_LIB_PATHS" =~ "$lib_dir" ]]; then
        RUNTIME_LIB_PATHS="$RUNTIME_LIB_PATHS:$lib_dir"
    fi
done

CAMERA_ID=${1:-0}
CONFIDENCE=${2:-0.5}

echo "Building camera test..."
# Build with minimal library path (only for build script's libclang)
if [ -n "$BUILD_LIB_PATH" ]; then
    export DYLD_FALLBACK_LIBRARY_PATH=$BUILD_LIB_PATH:$DYLD_FALLBACK_LIBRARY_PATH
fi
cargo build --bin camera_test --features opencv --release
BUILD_STATUS=$?

# Unset build-time paths
unset DYLD_FALLBACK_LIBRARY_PATH
unset DYLD_LIBRARY_PATH

if [ $BUILD_STATUS -eq 0 ]; then
    echo ""
    echo "Running camera test (camera_id=$CAMERA_ID, confidence=$CONFIDENCE)..."
    echo ""
    # Set runtime library paths
    export DYLD_FALLBACK_LIBRARY_PATH=$RUNTIME_LIB_PATHS:$DYLD_FALLBACK_LIBRARY_PATH
    export DYLD_LIBRARY_PATH=$RUNTIME_LIB_PATHS:$DYLD_LIBRARY_PATH
    ./target/release/camera_test $CAMERA_ID $CONFIDENCE
else
    echo "Build failed!"
    exit 1
fi

