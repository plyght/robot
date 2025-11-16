# Getting Started

## Prerequisites

- Rust 1.70 or later
- Basic understanding of robotics terminology

## Installation

Clone or add to your project:

```bash
cargo init --lib
# Add robot-hand dependency to Cargo.toml
```

## Verification

Run the test suite to verify installation:

```bash
cargo test
```

Expected output:
```
test result: ok. 7 passed; 0 failed; 0 ignored
```

## First Example

Run the basic control example with mock hardware:

```bash
cargo run --example basic_control
```

This demonstrates finger movement, wrist control, and basic operations without requiring physical hardware.

## Hardware Configuration

When your robotic hand arrives, follow these steps:

### 1. Identify Communication Protocol

Determine how your hand connects:

- USB/Serial: Most common, cross-platform
- I2C: Direct bus communication
- PWM: Direct pin control (embedded/Raspberry Pi)

### 2. Create Configuration File

Copy and modify `config/default.toml`:

```toml
[communication]
protocol = "serial"
serial_port = "/dev/ttyUSB0"  # Linux/macOS
# serial_port = "COM3"        # Windows
baud_rate = 115200
```

### 3. Map Hardware Channels

Define each finger and joint:

```toml
[[fingers]]
name = "Thumb"

[[fingers.joints]]
name = "Base"
motor_type = "pwmservo"
channel = 0
min_angle = 0.0
max_angle = 90.0
offset = 0.0
min_pulse = 500
max_pulse = 2500
```

Adjust parameters based on your hardware specifications.

### 4. Test Configuration

Build with appropriate features:

```bash
# For serial/USB connection
cargo build --features serial

# For mock testing
cargo build --features mock
```

### 5. Initial Hardware Test

Create a minimal test program:

```rust
use robot_hand::{HandConfig, HandController};

fn main() -> robot_hand::Result<()> {
    let config = HandConfig::from_file("config/my_hand.toml")?;
    let mut hand = HandController::new(config)?;
    
    hand.initialize()?;
    println!("Initialization successful");
    
    // Test single joint
    hand.move_finger(0, &[45.0])?;
    
    std::thread::sleep(std::time::Duration::from_secs(1));
    
    hand.move_finger(0, &[0.0])?;
    hand.shutdown()?;
    
    Ok(())
}
```

Run with:
```bash
cargo run --features serial
```

## Calibration

Adjust configuration values based on physical testing:

### Angle Limits

Test minimum and maximum positions manually, then update `min_angle` and `max_angle` in configuration.

### Offsets

If joints don't align at zero position, adjust the `offset` parameter.

### Pulse Widths (PWM Servos)

Tune `min_pulse` and `max_pulse` for full range of motion:

- Start with conservative values (500-2500)
- Gradually adjust based on actual servo response
- Stay within servo specifications

## Common Issues

### Permission Denied (Serial Port)

Linux/macOS:
```bash
sudo chmod 666 /dev/ttyUSB0
# Or add user to dialout group
sudo usermod -a -G dialout $USER
```

### Compilation Errors

Ensure feature flags match your platform:

- Development: `--features mock`
- Serial hardware: `--features serial`
- Raspberry Pi: `--features raspberry-pi` (Linux only)

### Unexpected Movement

- Verify channel mappings
- Check angle limits
- Confirm motor type configuration
- Review calibration offsets

## Next Steps

Once basic control is working:

1. Test all fingers and joints individually
2. Implement application-specific control logic
3. Add sensor feedback (see [Extending](extending.md))
4. Optimize motion parameters

## Development Workflow

### Without Hardware

```bash
cargo test
cargo run --example basic_control --features mock
```

### With Hardware

```bash
cargo build --features serial
cargo run --example basic_control --features serial
```

### Cross-Platform

Develop on any platform, deploy on target:

```bash
# Development on macOS/Windows
cargo build --features mock

# Deploy to Linux/Raspberry Pi
cargo build --target armv7-unknown-linux-gnueabihf --features serial
```

## Platform-Specific Notes

### Linux

All features supported. May require permissions for hardware access.

### Raspberry Pi

Full GPIO support with `raspberry-pi` feature. Requires Linux kernel headers.

### macOS/Windows

Use `mock` or `serial` features. Platform-specific GPIO features unavailable.

## Configuration Examples

### Single Joint Per Finger

```toml
[[fingers]]
name = "Index"

[[fingers.joints]]
name = "Flex"
motor_type = "pwmservo"
channel = 0
min_angle = 0.0
max_angle = 90.0
offset = 0.0
```

### Multiple Joints Per Finger

```toml
[[fingers]]
name = "Index"

[[fingers.joints]]
name = "MCP"
motor_type = "pwmservo"
channel = 0
min_angle = 0.0
max_angle = 90.0
offset = 0.0

[[fingers.joints]]
name = "PIP"
motor_type = "pwmservo"
channel = 1
min_angle = 0.0
max_angle = 90.0
offset = 0.0

[[fingers.joints]]
name = "DIP"
motor_type = "pwmservo"
channel = 2
min_angle = 0.0
max_angle = 90.0
offset = 0.0
```

### Stepper Motors

```toml
[[fingers.joints]]
name = "Base"
motor_type = "stepper"
channel = 0
min_angle = 0.0
max_angle = 180.0
offset = 0.0
```

### I2C Communication

```toml
[communication]
protocol = "i2c"
i2c_address = 64  # 0x40 in hex
```

## API Usage Patterns

### Basic Movement

```rust
hand.move_finger(finger_id, &[angle1, angle2, angle3])?;
```

### Synchronized Movement

```rust
hand.open_hand()?;
std::thread::sleep(Duration::from_secs(1));
hand.close_hand()?;
```

### Grasp Control

```rust
// Parameter is object size in mm
hand.grasp(30.0)?;  // Small object
hand.grasp(60.0)?;  // Large object
```

### Wrist Control

```rust
// [pitch, roll, yaw] in degrees
hand.move_wrist([10.0, 0.0, -5.0])?;
```

### Motion Planning

```rust
use robot_hand::MotionPlanner;

let planner = MotionPlanner::default();
let trajectory = planner.interpolate_trajectory(
    &[0.0, 0.0, 0.0],
    &[90.0, 45.0, 30.0],
    50  // steps
);

for pose in trajectory {
    hand.move_finger(0, &pose)?;
    std::thread::sleep(Duration::from_millis(20));
}
```

