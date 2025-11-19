# Camera Object Detection Test - Quick Start

## Run the Test

```bash
./run_camera_test.sh
```

This will:
1. Set up the required environment variables
2. Build the camera test program
3. Open your camera and display the live feed

## Controls

- Press **'q'** or **ESC** to quit

## What You'll See

- Live camera feed in a window titled "Camera Test - Object Detection"
- FPS counter and statistics at the top
- Any detected objects will be outlined with green bounding boxes (when detection is implemented)

## Troubleshooting

### Camera Permission Denied

Go to **System Settings → Privacy & Security → Camera** and grant permission to Terminal.

### Wrong Camera

Try a different camera ID:
```bash
./run_camera_test.sh 1    # Try camera 1
./run_camera_test.sh 2    # Try camera 2
```

### Build Errors

Make sure OpenCV is installed:
```bash
brew install opencv llvm
```

## Next Steps

Once you confirm the camera works:
1. Add actual object detection (YOLO, TensorFlow Lite, etc.)
2. Integrate with robot hand control via `vision_control` binary
3. Test pickup sequences with real objects

See `docs/camera-test.md` for detailed documentation.

