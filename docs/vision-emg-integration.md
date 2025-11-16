# Vision and EMG Integration

This system integrates computer vision, EMG (electromyography) sensing, and robotic hand control for autonomous object manipulation. The hand detects objects via camera, classifies them, selects appropriate grip patterns, and executes pickup sequences when triggered by muscle signals.

## System Architecture

```
Camera → Object Detection → Classification → Grip Selection
                                                    ↓
EMG Sensor → Threshold Detection → Pickup Sequence Trigger
                                          ↓
                              Serial Protocol → Servo Control
```

### Core Components

**Serial Protocol** (`src/protocol/`)
- Text-based command format: `servo<id> <finger> <angle>`
- Example: `servo1 Index 90`
- Mock and hardware implementations

**EMG Handler** (`src/emg/`)
- Reads analog values (0-1023) from sensor
- Threshold detection: ≥600 triggers pickup
- Debouncing prevents false triggers (200ms default)
- State machine: Idle → Triggered → Executing → Idle

**Vision System** (`src/vision/`)
- Object detection and classification
- OpenCV integration (feature flag)
- Mock detector for testing
- Supported objects: cup, phone, bottle, pen, card, small_object

**Grip Patterns** (`src/vision/grip_patterns.rs`)
- Power Grasp: Large cylindrical objects
- Precision Grip: Flat rectangular objects
- Pinch Grip: Small thin objects
- Lateral Grip: Card-like objects
- Tripod Grip: Specialized manipulation

**Pickup Sequence** (`src/control/pickup_sequence.rs`)
- Approach → Open → Grasp → Lift → Move → Release

**Vision Controller** (`src/control/vision_controller.rs`)
- Main control loop integrating all subsystems
- Continuous EMG polling and camera frame capture
- Autonomous execution of pickup sequences

## Hardware Setup

### Components

1. EMG sensor Arduino (analog output, 0-1023)
2. Servo controller Arduino (5 servos)
3. USB camera or built-in webcam
4. Appropriate power supply for servos

### Connections

- EMG Arduino → Computer (Serial: /dev/ttyUSB0 or COM3)
- Servo Arduino → Computer (Serial: /dev/ttyUSB1 or COM4)
- Camera → Computer (USB)

### Arduino Firmware

EMG sensor:
```cpp
void setup() {
  Serial.begin(9600);
  pinMode(A0, INPUT);
}

void loop() {
  int emgValue = analogRead(A0);
  Serial.println(emgValue);
  delay(10);
}
```

Servo controller:
```cpp
#include <Servo.h>

Servo servos[5];
int servoPins[] = {3, 5, 6, 9, 10};

void setup() {
  Serial.begin(115200);
  for(int i = 0; i < 5; i++) {
    servos[i].attach(servoPins[i]);
  }
}

void loop() {
  if(Serial.available()) {
    String cmd = Serial.readStringUntil('\n');
    parseCommand(cmd);
  }
}

void parseCommand(String cmd) {
  int servoId = cmd.substring(5, cmd.indexOf(' ', 5)).toInt();
  int angle = cmd.substring(cmd.lastIndexOf(' ') + 1).toInt();
  
  if(servoId >= 0 && servoId < 5) {
    servos[servoId].write(angle);
  }
}
```

## Configuration

Edit `config/vision_control.toml`:

```toml
[camera]
device_id = 0
width = 640
height = 480
fps = 30

[emg]
serial_port = "/dev/ttyUSB0"
baud_rate = 9600
threshold = 600
debounce_ms = 200

[servo]
serial_port = "/dev/ttyUSB1"
baud_rate = 115200

[finger_servo_mapping]
Thumb = 0
Index = 1
Middle = 2
Ring = 3
Pinky = 4
```

## Building and Running

Mock hardware (testing):
```bash
cargo run --bin vision_control
```

Real hardware:
```bash
cargo run --bin vision_control --features serial
```

OpenCV integration:
```bash
cargo run --bin vision_control --features "serial,opencv"
```

Examples:
```bash
cargo run --example vision_emg_demo
```

## Operation

### Initialization
System connects to EMG and servo serial ports, initializes camera, and enters main control loop.

### Idle State
Continuously polls EMG sensor, waiting for threshold crossing (≥600).

### Triggered State
1. EMG threshold crossed
2. Capture camera frame
3. Detect and classify objects
4. Select best object by confidence and position
5. Determine grip pattern based on object type
6. Execute pickup sequence

### Object Selection Algorithm
Objects scored by:
- Detection confidence
- Distance to frame center
- Combined weighted score

### Grip Pattern Mapping
- Cup/Bottle → Power Grasp
- Phone/Book → Precision Grip
- Pen/Pencil → Pinch Grip
- Card → Lateral Grip

## API Usage

### Programmatic Control

```rust
use robot_hand::{
    VisionController, VisionControllerConfig,
    MockObjectDetector, EmgReader, TextSerialController,
    DetectedObject, BoundingBox,
};

fn main() -> robot_hand::Result<()> {
    let mut detector = MockObjectDetector::new(640, 480);
    detector.add_mock_object(DetectedObject {
        label: "cup".to_string(),
        confidence: 0.95,
        bounding_box: BoundingBox {
            x: 200, y: 150, width: 100, height: 120,
        },
        distance: 0.35,
    });

    let emg_reader = EmgReader::new("/dev/ttyUSB0", 9600, 600)?;
    let protocol = TextSerialController::new("/dev/ttyUSB1", 115200)?;
    let config = VisionControllerConfig::default();

    let mut controller = VisionController::new(
        detector,
        emg_reader,
        protocol,
        config,
    );

    controller.run()?;
    Ok(())
}
```

### Manual Pickup Sequence

```rust
use robot_hand::{
    PickupSequence, GripPattern, MockSerialController,
    create_default_finger_servo_map,
};

fn main() -> robot_hand::Result<()> {
    let grip = GripPattern::power_grasp();
    let mut sequence = PickupSequence::new(grip);
    let mut protocol = MockSerialController::new();
    let servo_map = create_default_finger_servo_map();

    while !sequence.is_complete() {
        sequence.execute_step_by_step(&mut protocol, &servo_map)?;
    }

    Ok(())
}
```

### Custom Grip Patterns

```rust
use robot_hand::GripPattern;
use std::collections::HashMap;

fn custom_grip() -> GripPattern {
    let mut finger_angles = HashMap::new();
    finger_angles.insert("Thumb".to_string(), vec![70.0, 65.0, 60.0]);
    finger_angles.insert("Index".to_string(), vec![85.0, 80.0, 75.0]);
    finger_angles.insert("Middle".to_string(), vec![85.0, 80.0, 75.0]);
    finger_angles.insert("Ring".to_string(), vec![50.0, 45.0, 40.0]);
    finger_angles.insert("Pinky".to_string(), vec![50.0, 45.0, 40.0]);

    GripPattern {
        pattern_type: robot_hand::GripPatternType::PowerGrasp,
        finger_angles,
        wrist_orientation: Some([5.0, 0.0, -3.0]),
        approach_distance: 12.0,
    }
}
```

## Troubleshooting

### EMG Not Triggering
- Verify threshold value in configuration
- Check EMG Arduino serial output: `screen /dev/ttyUSB0 9600`
- Confirm serial port and baud rate
- Test sensor connection

### Servos Not Responding
- Verify serial port and baud rate (115200)
- Check servo power supply
- Test Arduino serial connection
- Confirm servo wiring

### No Object Detection
- Verify camera permissions and connection
- Test with mock detector first
- Check confidence threshold settings
- Ensure proper lighting conditions

### Serial Port Access
Linux/macOS:
```bash
sudo chmod 666 /dev/ttyUSB0
sudo chmod 666 /dev/ttyUSB1
```

Or add user to dialout group:
```bash
sudo usermod -a -G dialout $USER
```

## Performance Tuning

- EMG polling: 10ms interval (100Hz optimal)
- Camera capture: 100ms interval (10 FPS balances CPU and responsiveness)
- Debouncing: 200ms prevents false triggers
- Servo speed: Add delays between movements for smooth motion

## Testing Without Hardware

Mock implementations available for all subsystems:
- `MockObjectDetector`: Simulated camera with predefined objects
- `MockSerialController`: Prints commands to terminal
- `MockEmgReader`: Injectable test values

Run complete demo:
```bash
cargo run --bin vision_control
```

Demo shows:
- Object detection (mock: cup, phone, bottle)
- EMG trigger simulation (value=650)
- Object selection (highest confidence + centered)
- Grip pattern selection (Power Grasp for cup)
- Full pickup sequence execution

## Adding OpenCV

1. Install OpenCV: `brew install opencv` (macOS) or platform equivalent
2. Add opencv crate to Cargo.toml
3. Build with feature: `cargo build --features "serial,opencv"`
4. Replace mock detector with real vision implementation

## Safety Considerations

- Test with mock hardware before connecting real servos
- Keep emergency stop accessible
- Use conservative servo speeds during initial testing
- Monitor servo current draw and temperature
- Maintain safe operating area during autonomous operation
- Implement limit switches or soft limits as appropriate

## Extension Points

The system is designed for modularity:

- Add grip patterns in `src/vision/grip_patterns.rs`
- Extend object classification in `src/vision/mod.rs`
- Customize pickup sequences in `src/control/pickup_sequence.rs`
- Modify control logic in `src/control/vision_controller.rs`

## Future Enhancements

- Voice control integration
- Multiple EMG channels for complex gestures
- Force feedback and adaptive gripping
- Machine learning for preference learning
- Web interface for monitoring and control
- Real-time trajectory visualization
