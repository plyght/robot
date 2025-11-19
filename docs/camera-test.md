# Camera Object Detection Test

This document describes how to use the camera test program to test object detection independently from the robot hand control.

## Prerequisites

1. **OpenCV** - Install via Homebrew:
   ```bash
   brew install opencv
   ```

2. **LLVM** - Required for OpenCV bindings (usually already installed):
   ```bash
   brew install llvm
   ```

3. **Webcam or USB Camera** - Built-in or external camera connected to your Mac.

## Building

### Option 1: Using the Helper Script (Recommended)

```bash
./run_camera_test.sh [camera_id] [confidence_threshold]
```

Examples:
```bash
./run_camera_test.sh        # Use default camera (0) and confidence (0.5)
./run_camera_test.sh 0 0.7  # Camera 0, confidence 0.7
./run_camera_test.sh 1      # Camera 1, default confidence
```

### Option 2: Manual Build

Set environment variables:
```bash
export OPENCV_INCLUDE_PATHS=/opt/homebrew/Cellar/opencv/4.12.0_14/include/opencv4
export OPENCV_LINK_PATHS=/opt/homebrew/Cellar/opencv/4.12.0_14/lib
export OPENCV_LINK_LIBS=opencv_core,opencv_highgui,opencv_imgproc,opencv_videoio,opencv_dnn
export DYLD_FALLBACK_LIBRARY_PATH=/opt/homebrew/opt/llvm/lib:$DYLD_FALLBACK_LIBRARY_PATH
```

Build and run:
```bash
cargo build --bin camera_test --features opencv --release
./target/release/camera_test [camera_id] [confidence_threshold]
```

## Usage

The camera test program opens a window showing the live camera feed with detected objects highlighted.

### Command-Line Arguments

- **camera_id** (optional, default: 0) - The camera device ID
  - 0 = Built-in camera (usually)
  - 1, 2, etc. = External USB cameras
  
- **confidence_threshold** (optional, default: 0.5) - Minimum confidence for detections
  - Range: 0.0 to 1.0
  - Lower = more detections (may include false positives)
  - Higher = fewer, more confident detections

### Examples

```bash
# Use built-in camera with default confidence
./run_camera_test.sh

# Use external camera (ID 1)
./run_camera_test.sh 1

# Use built-in camera with higher confidence threshold
./run_camera_test.sh 0 0.8
```

## Controls

- **q** or **ESC** - Quit the program

## Display

The camera test window shows:
- Live camera feed
- Green bounding boxes around detected objects
- Object labels with confidence percentages
- Statistics: FPS, current object count, total objects detected

## Current Implementation

The current implementation provides:
- Live camera capture and display
- Basic window controls
- FPS monitoring
- Frame statistics

**Note:** Object detection is currently simplified. A full YOLO or other detection model integration would be needed for actual object recognition.

## Troubleshooting

### Camera Not Opening

- Check camera permissions in System Settings → Privacy & Security → Camera
- Try a different camera ID (0, 1, 2, etc.)
- Ensure no other application is using the camera

### Build Errors

If you get OpenCV-related build errors:

1. Verify OpenCV installation:
   ```bash
   brew list opencv
   ```

2. Check the OpenCV version and update paths in `run_camera_test.sh` if different:
   ```bash
   ls /opt/homebrew/Cellar/opencv/
   ```

3. Reinstall OpenCV if needed:
   ```bash
   brew reinstall opencv
   ```

### Window Not Appearing

- Ensure you're running from a terminal (not via SSH)
- Check that X11 or native windowing is available

## Next Steps

To add actual object detection:

1. Download a YOLO model (e.g., YOLOv8)
2. Use the `load_yolo_model` method in `OpenCVDetector`
3. Implement YOLO inference in the `detect_with_color` method
4. Or use OpenCV's DNN module with pre-trained models

Example:
```rust
detector.load_yolo_model("yolov8.weights", "yolov8.cfg")?;
```

## Integration with Robot Hand

Once object detection is working, you can integrate it with the robot hand control:

```bash
# Run vision control with real object detection
cargo run --bin vision_control --features "serial,opencv" -- /dev/cu.usbmodem1101 mock
```

This will use the same `OpenCVDetector` for live object detection during robot operation.

