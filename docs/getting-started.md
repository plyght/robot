# Getting Started

## Prerequisites

- Rust 1.70 or later
- Understanding of basic robotics concepts

## Verification

Verify installation:

```bash
cargo test
cargo run --example basic_control
```

The basic control example demonstrates finger and wrist movement using mock hardware.

## Hardware Configuration

### Communication Protocol

Identify your connection method:
- USB/Serial: Cross-platform, most common
- I2C: Direct bus communication
- PWM: Pin control (embedded/Raspberry Pi)

### Configuration File

Create or modify `config/default.toml`:

```toml
[communication]
protocol = "serial"
serial_port = "/dev/ttyUSB0"
baud_rate = 115200
```

### Hardware Mapping

Define fingers and joints:

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

### Build

```bash
cargo build --features serial
cargo build --features mock
```

### Initial Test

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

```bash
cargo run --features serial
```

## Calibration

### Angle Limits
Test physical range, then update `min_angle` and `max_angle`.

### Offsets
Adjust `offset` parameter if joints misalign at zero.

### PWM Pulse Widths
Start with 500-2500, tune based on servo response within specifications.

## Common Issues

### Serial Port Permission
```bash
sudo chmod 666 /dev/ttyUSB0
sudo usermod -a -G dialout $USER
```

### Compilation
Verify feature flags match your platform and hardware.

### Unexpected Movement
- Verify channel mappings
- Check angle limits
- Confirm motor type
- Review offsets

## Next Steps

1. Test all joints individually
2. Implement application logic
3. Add sensor feedback (see [Extending](extending.md))
4. Optimize motion parameters

## Development Workflow

Without hardware:
```bash
cargo test
cargo run --example basic_control --features mock
```

With hardware:
```bash
cargo build --features serial
cargo run --example basic_control --features serial
```

Cross-platform:
```bash
cargo build --features mock
cargo build --target armv7-unknown-linux-gnueabihf --features serial
```

## Platform Notes

- Linux: All features supported, may require permissions
- Raspberry Pi: Full GPIO with `raspberry-pi` feature
- macOS/Windows: Mock and serial only

## Configuration Examples

Single joint:
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

Multiple joints:

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

Stepper motors:
```toml
[[fingers.joints]]
name = "Base"
motor_type = "stepper"
channel = 0
min_angle = 0.0
max_angle = 180.0
offset = 0.0
```

I2C communication:

```toml
[communication]
protocol = "i2c"
i2c_address = 64  # 0x40 in hex
```

## API Usage

Basic movement:
```rust
hand.move_finger(finger_id, &[angle1, angle2, angle3])?;
```

Synchronized movement:
```rust
hand.open_hand()?;
std::thread::sleep(Duration::from_secs(1));
hand.close_hand()?;
```

Grasp control:
```rust
hand.grasp(30.0)?;
```

Wrist control:
```rust
hand.move_wrist([10.0, 0.0, -5.0])?;
```

Motion planning:

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

