# Robot Hand Control Library

Universal Rust library for controlling robotic hands with flexible hardware abstraction supporting multiple motor types and communication protocols.

## Features

- Hardware-agnostic trait-based architecture
- PWM servos, stepper motors, and DC motor support
- Serial, I2C, and PWM communication protocols
- TOML configuration system
- Vision and EMG integration for autonomous object manipulation
- Mock hardware support for cross-platform development

## Quick Start

```bash
cargo test
cargo run --example basic_control
```

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
robot-hand = { path = "." }

# With features
robot-hand = { path = ".", features = ["serial"] }
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

Create a TOML configuration file defining your hand structure:

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

[wrist]
[wrist.pitch]
name = "Pitch"
motor_type = "pwmservo"
channel = 15
min_angle = -45.0
max_angle = 45.0
offset = 0.0
```

## Architecture

Modular layer design:

- **Hardware**: Motor and communication abstractions
- **Hand Model**: Finger, wrist, and joint structures
- **Control**: High-level control API and motion planning
- **Vision/EMG**: Object detection and muscle signal integration
- **Protocol**: Text-based serial and hardware communication
- **Platform**: Platform-specific implementations

## Cargo Features

- `mock` (default): Virtual hardware for testing
- `serial`: USB/Serial communication
- `linux-pwm`: Linux PWM interface
- `embedded`: Embedded HAL support
- `raspberry-pi`: Raspberry Pi GPIO (Linux only)

## Platform Support

| Platform | Development | Hardware Control |
|----------|-------------|------------------|
| Linux | Yes | Yes (all features) |
| Raspberry Pi | Yes | Yes (all features) |
| macOS | Yes | Serial only |
| Windows | Yes | Serial only |

## API Reference

### HandController

```rust
// Initialization
hand.initialize() -> Result<()>
hand.shutdown() -> Result<()>

// Finger control
hand.move_finger(finger_id: usize, angles: &[f32]) -> Result<()>
hand.open_hand() -> Result<()>
hand.close_hand() -> Result<()>

// Grasping
hand.grasp(object_size: f32) -> Result<()>

// Wrist control
hand.move_wrist(orientation: [f32; 3]) -> Result<()>
```

### Configuration

```rust
HandConfig::from_file(path) -> Result<HandConfig>
HandConfig::from_string(toml) -> Result<HandConfig>
config.validate() -> Result<()>
config.to_file(path) -> Result<()>
```

## Examples

- `basic_control.rs`: Finger and wrist movement
- `grasp_patterns.rs`: Predefined grasp patterns
- `config_example.rs`: Configuration file usage
- `vision_emg_demo.rs`: Autonomous object manipulation

## Documentation

- [Getting Started](docs/getting-started.md): Installation and configuration
- [Vision+EMG Integration](docs/vision-emg-integration.md): Autonomous object pickup
- [Extending](docs/extending.md): Custom motors and protocols

## Testing

```bash
cargo test
cargo build --all-targets
cargo run --example basic_control
cargo run --bin vision_control
```

## License

MIT
