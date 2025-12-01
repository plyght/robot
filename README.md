# Robot Hand Control System

**Complete LLM-driven robot hand control with computer vision, depth estimation, hand tracking, and inverse kinematics.**

## Features

### Vision & Perception
- **YOLO Object Detection** - Real-time object detection with YOLOv8
- **Depth Estimation** - Apple Depth Pro integration for metric depth
- **Hand Tracking** - MediaPipe-based hand pose estimation (optional)
- **Camera Integration** - OpenCV VideoCapture support

### Intelligence & Planning
- **LLM Planning** - OpenAI GPT integration for adaptive movement generation
- **Inverse Kinematics** - Automatic target position → joint angle solving
- **Forward Kinematics** - Real-time hand position tracking
- **Scene Analysis** - Multi-object awareness and obstacle avoidance

### Hardware Control
- **Unified Servo Mapping** - Hardware abstraction with automatic angle translation
- **Inverted Servo Support** - Handles reversed servo orientations
- **Serial Protocol** - USB serial communication (115200 baud)
- **Mock Mode** - Test without hardware

### System Architecture
- **Async Runtime** - Tokio-based async execution
- **Modular Design** - Clean separation of vision, planning, kinematics, and control
- **Configurable** - Command-line options and environment variables

## Quick Start

### Test Vision Pipeline (No Hardware)

```bash
cargo run --features opencv --release -- --auto --mock
```

### Run with Real Robot

```bash
cargo run --features opencv,serial --release -- \
  --port /dev/cu.usbmodem1101 \
  --auto
```

**See [SETUP.md](SETUP.md) for complete installation and usage guide.**

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
robot-hand = { path = "." }

# With features
robot-hand = { path = ".", features = ["serial", "opencv"] }
```

## Basic Usage

```rust
use robot_hand::{HandConfig, HandController};

fn main() -> robot_hand::Result<()> {
    let config = HandConfig::from_file("config/default.toml")?;
    let mut hand = HandController::new(config)?;
    
    hand.initialize()?;
    hand.move_finger(0, &[45.0, 30.0, 20.0])?;
    hand.grasp(50.0)?;
    hand.shutdown()?;
    
    Ok(())
}
```

## Configuration

TOML configuration defines hand structure and hardware mapping:

```toml
[communication]
protocol = "serial"
serial_port = "/dev/ttyUSB0"
baud_rate = 115200

[[fingers]]
name = "Index"

[[fingers.joints]]
name = "MCP"
motor_type = "pwmservo"
channel = 0
min_angle = 0.0
max_angle = 90.0
offset = 0.0
min_pulse = 500
max_pulse = 2500
```

See `config/default.toml` for complete examples.

## Architecture

Modular layer design:

- **Hardware**: Motor and communication abstractions
- **Hand Model**: Finger, wrist, and joint structures
- **Control**: High-level control API and motion planning
- **Vision**: Object detection, tracking, and depth estimation
- **EMG**: Muscle signal processing and threshold detection
- **Protocol**: Text-based serial and hardware communication
- **Platform**: Platform-specific implementations

## Cargo Features

- `mock` (default): Virtual hardware for testing
- `serial`: USB/Serial communication
- `opencv`: Computer vision with OpenCV and YOLO
- `linux-pwm`: Linux PWM interface
- `embedded`: Embedded HAL support
- `raspberry-pi`: Raspberry Pi GPIO

## Platform Support

| Platform | Development | Hardware Control | Vision |
|----------|-------------|------------------|--------|
| Linux | Yes | Yes (all features) | Yes |
| Raspberry Pi | Yes | Yes (all features) | Yes |
| macOS | Yes | Serial only | Yes |
| Windows | Yes | Serial only | Yes |

## API Reference

### HandController

```rust
hand.initialize() -> Result<()>
hand.shutdown() -> Result<()>
hand.move_finger(finger_id: usize, angles: &[f32]) -> Result<()>
hand.open_hand() -> Result<()>
hand.close_hand() -> Result<()>
hand.grasp(object_size: f32) -> Result<()>
hand.move_wrist(orientation: [f32; 3]) -> Result<()>
```

### Vision System

```rust
let mut detector = OpenCVDetector::new(camera_id, confidence)?;
detector.load_yolo_model("models/yolov8n.onnx")?;
let objects = detector.detect_objects()?;
```

### Depth Estimation

```rust
use robot_hand::{ensure_temp_dir, cleanup_temp_files};

ensure_temp_dir()?;

let mut depth_service = DepthProService::new(Some("venv_depth_pro/bin/python3"))?;
let depths = depth_service.process_image_with_cleanup("temp/frame.jpg", &objects, true)?;

cleanup_temp_files();
```

## Examples

- `basic_control.rs`: Basic finger and wrist movement
- `grasp_patterns.rs`: Predefined grasp patterns
- `config_example.rs`: Configuration file usage
- `vision_emg_demo.rs`: Autonomous object manipulation

## Documentation

- [Getting Started](docs/getting-started.md): Installation and configuration
- [Vision and EMG Integration](docs/vision-emg-integration.md): Autonomous operation
- [Camera Test](docs/camera-test.md): Vision system testing
- [Depth Estimation](docs/depth-estimation.md): Depth Pro integration
- [AI Integration](docs/ai-integration.md): Custom AI model training
- [Extending](docs/extending.md): Custom motors and protocols

## Testing

Run unit tests:

```bash
cargo test
```

Build all targets:

```bash
cargo build --all-targets
```

Test vision system:

```bash
./run_camera_test.sh
./run_depth_test.sh --auto
```

Run examples:

```bash
cargo run --example basic_control
cargo run --bin vision_control --features serial,opencv
```

## License

MIT

