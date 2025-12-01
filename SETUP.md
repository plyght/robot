# Robot Hand Control System - Setup Guide

## Overview

This system integrates computer vision, depth estimation, hand tracking, and LLM-based planning to control a robot hand for object manipulation tasks.

**Complete Pipeline:**
1. Camera captures video frames
2. YOLO detects objects
3. Depth Pro estimates object depth
4. Hand tracking (optional) detects hand pose and position
5. LLM generates movement commands based on scene analysis
6. Forward/Inverse kinematics compute joint angles
7. Robot hand executes movement sequence

---

## Prerequisites

### Hardware
- Mac with camera (built-in or USB)
- Robot hand connected via USB serial (e.g., `/dev/cu.usbmodem1101`)
- Robot hand servo configuration:
  - Servo ID 1: Ring finger (normal)
  - Servo ID 2: Middle finger (normal)
  - Servo ID 3: Pinky finger (normal)
  - Servo ID 4: Index finger (INVERTED - 180=open, 0=closed)
  - Servo ID 5: Thumb (normal)
  - Servo ID 10: Wrist pitch (optional)
  - Servo ID 11: Wrist roll (optional)

### Software
- Rust toolchain (2021 edition or later)
- Python 3.9+
- OpenCV
- OpenAI API key (for LLM planning)

---

## Installation

### 1. Install System Dependencies

#### macOS
```bash
brew install opencv python@3.9
```

#### Ubuntu/Linux
```bash
sudo apt-get install libopencv-dev python3 python3-pip
```

### 2. Install Python Dependencies

```bash
pip3 install coremltools pillow numpy scipy
```

### 3. Build Rust Project

```bash
cargo build --features opencv,serial --release
```

---

## Model Setup

### Required Models

1. **YOLO Object Detection**
   - Path: `models/yolov8n.onnx`
   - Already included (12MB)

2. **Depth Estimation**
   - Service: `depth_service.py` (included)
   - Apple Depth Pro CoreML model (Mac only)
   - Auto-initialized on first run

3. **Hand Tracking (Optional)**
   - Path: `models/hand_landmarker.onnx`
   - Already included (86KB)
   - Converted from MediaPipe

All models are already present in the `models/` directory.

---

## Configuration

### Environment Variables

Create or add to your shell config (`~/.zshrc`, `~/.bashrc`, or `~/.config/fish/config.fish`):

```bash
export OPENAI_API_KEY="your-openai-api-key-here"
export YOLO_MODEL_PATH="models/yolov8n.onnx"
export HAND_MODEL_PATH="models/hand_landmarker.onnx"
```

Reload your shell:
```bash
source ~/.zshrc  # or ~/.bashrc or ~/.config/fish/config.fish
```

---

## Usage

### Test Vision Pipeline (No Robot)

Test camera, object detection, depth estimation, and hand tracking without hardware:

```bash
cargo run --features opencv --release -- --auto --mock
```

This will:
- Open camera ID 0
- Detect objects automatically when they appear
- Use mock servo controller (no hardware needed)
- Print movement commands to console

### Run with Real Robot Hand

Connect your robot hand via USB, then:

```bash
cargo run --features opencv,serial --release -- \
  --port /dev/cu.usbmodem1101 \
  --auto
```

Replace `/dev/cu.usbmodem1101` with your actual serial port.

### Command-Line Options

```
--port <device>      Serial port (e.g., /dev/cu.usbmodem1101)
--camera <id>        Camera ID (default: 0)
--auto               Auto-trigger mode (no EMG needed)
--mock               Use mock controller (for testing)
--help               Show help message
```

### Examples

**Test with different camera:**
```bash
cargo run --features opencv --release -- --camera 1 --auto --mock
```

**Run with real hardware:**
```bash
cargo run --features opencv,serial --release -- \
  --port /dev/cu.usbmodem1101 \
  --auto
```

**Manual control (EMG trigger):**
```bash
cargo run --features opencv,serial --release -- \
  --port /dev/cu.usbmodem1101
```

---

## Manual Robot Control

For direct servo control without vision:

```bash
cargo run --bin simple_control --features serial -- /dev/cu.usbmodem1101
```

Interactive commands:
- `open` - Open all fingers
- `close` - Close all fingers
- `all <angle>` - Move all fingers to angle (0-180)
- `<finger> <angle>` - Move specific finger
  - Fingers: `pinky`, `index`, `middle`, `ring`, `1`, `2`, `3`, `4`
- `q` - Quit

Example session:
```
> open
✓ Hand opened
> index 45
✓ Finger index (servo 4) moved to 45° (sent: 135°)
> close
✓ Hand closed
```

---

## Architecture

### Project Structure

```
robot/
├── src/
│   ├── bin/
│   │   ├── simple_control.rs       # Manual servo control CLI
│   │   └── test_serial.rs          # Serial connection test
│   ├── control/
│   │   ├── llm_planner.rs          # OpenAI LLM integration
│   │   ├── llm_vision_controller.rs # Main async controller
│   │   └── pickup_sequence.rs      # Fallback grip logic
│   ├── hardware/
│   │   └── servo_map.rs            # Unified servo configuration
│   ├── kinematics/
│   │   ├── forward.rs              # Forward kinematics solver
│   │   ├── inverse.rs              # Inverse kinematics solver
│   │   └── types.rs                # Position, JointAngles, etc.
│   ├── vision/
│   │   ├── depth_pro.rs            # Depth estimation service
│   │   ├── hand_tracker.rs         # ONNX hand tracking
│   │   └── mod.rs                  # YOLO object detection
│   └── main.rs                     # Primary entry point
├── models/
│   ├── yolov8n.onnx               # Object detection model
│   └── hand_landmarker.onnx       # Hand tracking model
├── scripts/                        # Shell scripts
├── examples_archive/               # Old test programs
└── depth_service.py               # Python depth estimation
```

### Data Flow

```
Camera → YOLO → Depth Pro → Hand Tracker (optional)
                    ↓
            Scene State (target object, depth, hand pose)
                    ↓
            LLM Planner (OpenAI API)
                    ↓
        Movement Commands (MoveToPosition, Grasp, etc.)
                    ↓
    Inverse Kinematics (target → joint angles)
                    ↓
        Servo Map (translate angles, handle inverted servos)
                    ↓
            Serial Protocol → Robot Hand
                    ↓
    Forward Kinematics (update hand position tracking)
```

---

## Troubleshooting

### Camera Not Found
```
Error: Failed to open camera
```
**Solution:** Check camera ID with `ls /dev/video*` (Linux) or System Preferences (Mac)

### Serial Port Permission Denied
```
Error: Permission denied: /dev/cu.usbmodem1101
```
**Solution (Mac):**
```bash
sudo chmod 666 /dev/cu.usbmodem1101
```

### YOLO Model Not Found
```
Error: Failed to load YOLO model
```
**Solution:** Verify `models/yolov8n.onnx` exists or set `YOLO_MODEL_PATH`

### Depth Service Fails
```
Warning: Depth detection service failed to initialize
```
**Solution:** Ensure Python dependencies installed:
```bash
pip3 install coremltools numpy pillow scipy
```

### LLM Planning Disabled
```
⚠️  LLM planning disabled
```
**Solution:** Set OpenAI API key:
```bash
export OPENAI_API_KEY="sk-..."
```

### Servo Not Moving / Wrong Finger Moves
**Solution:** Verify servo mapping in `src/hardware/servo_map.rs:68-76`

The hardware default mapping is:
```rust
Ring:   Servo 1 (normal)
Middle: Servo 2 (normal)
Pinky:  Servo 3 (normal)
Index:  Servo 4 (INVERTED)
Thumb:  Servo 5 (normal)
```

---

## Performance

- **LLM latency:** 500-2000ms per query (model dependent)
- **YOLO inference:** ~140ms per frame (640x640)
- **Depth estimation:** ~200ms per frame
- **Hand tracking:** ~50ms per frame
- **Total pipeline:** 1-3 seconds from trigger to first movement

---

## Next Steps

1. **Test vision pipeline** with `--auto --mock`
2. **Connect robot hand** and test with `simple_control`
3. **Run full system** with `--port /dev/cu.usbmodem1101 --auto`
4. **Calibrate kinematics** - adjust `HandGeometry` in `src/kinematics/types.rs` if needed
5. **Tune LLM prompts** - edit system prompt in `src/control/llm_planner.rs`

---

## Support

For issues, check:
- Serial connection: `cargo run --bin test_serial --features serial`
- Camera feed: Previous test programs in `examples_archive/`
- Logs: stdout shows detailed pipeline execution

For configuration changes, see `src/hardware/servo_map.rs` (servo mapping) and `src/kinematics/types.rs` (hand geometry).
