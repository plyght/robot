# Robot Hand Control Library

A universal Rust library for controlling robotic hands with flexible hardware abstraction supporting multiple motor types and communication protocols.

## Features

- Hardware-agnostic design with trait-based abstractions
- Support for PWM servos, stepper motors, and DC motors
- Multiple communication protocols (Serial, I2C, PWM)
- TOML-based configuration system
- Cross-platform development with mock hardware
- Zero-cost abstractions with compile-time optimization

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

The library is organized into modular layers:

- **Hardware Layer**: Motor and communication abstractions
- **Hand Model**: Finger, wrist, and joint structures
- **Configuration**: TOML-based runtime configuration
- **Control**: High-level API for hand control
- **Platform**: Platform-specific driver implementations

## Features

Enable optional features as needed:

- `mock` (default): Virtual hardware for testing
- `serial`: USB/Serial communication
- `linux-pwm`: Linux PWM interface
- `embedded`: Embedded HAL support
- `raspberry-pi`: Raspberry Pi GPIO support (Linux only)

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

See `examples/` directory:

- `basic_control.rs`: Simple finger and wrist movement
- `grasp_patterns.rs`: Predefined grasp demonstrations
- `config_example.rs`: Configuration file manipulation

## Documentation

- [Getting Started](docs/getting-started.md): Initial setup and first steps
- [Extending](docs/extending.md): Adding custom motors and protocols

## Testing

```bash
cargo test                    # Run all tests
cargo build --all-targets     # Build library and examples
cargo run --example <name>    # Run specific example
```

## License

MIT
