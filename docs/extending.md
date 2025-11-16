# Extending the Library

## Adding Custom Motor Types

Implement the `Motor` trait for new motor types:

```rust
use crate::error::{HandError, Result};
use crate::hardware::Motor;

pub struct CustomMotor {
    id: usize,
    current_position: f32,
    enabled: bool,
    min_angle: f32,
    max_angle: f32,
}

impl Motor for CustomMotor {
    fn set_position(&mut self, angle: f32) -> Result<()> {
        if angle < self.min_angle || angle > self.max_angle {
            return Err(HandError::InvalidJointAngle {
                joint_id: self.id,
                angle,
                min: self.min_angle,
                max: self.max_angle,
            });
        }
        self.current_position = angle;
        Ok(())
    }

    fn get_position(&self) -> Result<f32> {
        Ok(self.current_position)
    }

    fn enable(&mut self) -> Result<()> {
        self.enabled = true;
        Ok(())
    }

    fn disable(&mut self) -> Result<()> {
        self.enabled = false;
        Ok(())
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn get_limits(&self) -> (f32, f32) {
        (self.min_angle, self.max_angle)
    }
}
```

### Integrating Custom Motors

Add to configuration enum in `src/config/config.rs`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MotorType {
    PwmServo,
    Stepper,
    Dc,
    Custom,
}
```

Update controller in `src/control/controller.rs`:

```rust
fn create_motor(
    joint_config: &JointConfig,
    _controller: &Box<dyn MotorController>,
) -> Result<Box<dyn Motor>> {
    match joint_config.motor_type {
        MotorType::Custom => Ok(Box::new(CustomMotor::new(
            joint_config.channel as usize,
            joint_config.min_angle,
            joint_config.max_angle,
        ))),
        // ... existing types
    }
}
```

## Adding Communication Protocols

Implement the `MotorController` trait:

```rust
use crate::error::Result;
use crate::hardware::MotorController;

pub struct CustomController {
    device: String,
}

impl CustomController {
    pub fn new(device: &str) -> Result<Self> {
        Ok(Self {
            device: device.to_string(),
        })
    }
}

impl MotorController for CustomController {
    fn write_pwm(&mut self, channel: u8, value: u16) -> Result<()> {
        // Implementation
        Ok(())
    }

    fn read_pwm(&mut self, channel: u8) -> Result<u16> {
        // Implementation
        Ok(0)
    }

    fn write_data(&mut self, address: u8, data: &[u8]) -> Result<()> {
        // Implementation
        Ok(())
    }

    fn read_data(&mut self, address: u8, buffer: &mut [u8]) -> Result<usize> {
        // Implementation
        Ok(buffer.len())
    }
}
```

### Integrating Custom Protocols

Add to protocol enum:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Protocol {
    Serial,
    I2c,
    Mock,
    Custom,
}
```

Update controller creation:

```rust
fn create_controller(config: &HandConfig) -> Result<Box<dyn MotorController>> {
    match config.communication.protocol {
        Protocol::Custom => Ok(Box::new(CustomController::new(
            &config.communication.serial_port,
        )?)),
        // ... existing protocols
    }
}
```

## Adding Sensor Support

Define sensor trait:

```rust
pub trait Sensor: Send {
    fn read(&mut self) -> Result<f32>;
    fn calibrate(&mut self) -> Result<()>;
}
```

Implement for specific sensors:

```rust
pub struct ForceSensor {
    channel: u8,
    controller: Box<dyn MotorController>,
    calibration_offset: f32,
}

impl Sensor for ForceSensor {
    fn read(&mut self) -> Result<f32> {
        let raw = self.controller.read_pwm(self.channel)?;
        Ok((raw as f32 * 0.1) - self.calibration_offset)
    }

    fn calibrate(&mut self) -> Result<()> {
        let mut sum = 0.0;
        for _ in 0..10 {
            let raw = self.controller.read_pwm(self.channel)?;
            sum += raw as f32;
        }
        self.calibration_offset = sum / 10.0;
        Ok(())
    }
}
```

### Integrating Sensors

Extend joint structure in `src/hand/finger.rs`:

```rust
pub struct Joint {
    motor: Box<dyn Motor>,
    sensor: Option<Box<dyn Sensor>>,
    name: String,
    offset: f32,
}

impl Joint {
    pub fn read_sensor(&mut self) -> Result<Option<f32>> {
        if let Some(sensor) = &mut self.sensor {
            Ok(Some(sensor.read()?))
        } else {
            Ok(None)
        }
    }
}
```

## Platform-Specific Implementations

Use feature flags for platform-specific code:

```rust
#[cfg(feature = "raspberry-pi")]
pub mod raspberry_pi {
    use crate::hardware::MotorController;
    
    pub struct RpiController {
        // Raspberry Pi specific fields
    }
    
    impl MotorController for RpiController {
        // Implementation
    }
}
```

Add feature to `Cargo.toml`:

```toml
[features]
raspberry-pi = ["dep:rppal"]

[dependencies]
rppal = { version = "0.18", optional = true }
```

## Example: Dynamixel Servo Implementation

```rust
pub struct DynamixelServo {
    id: u8,
    controller: Box<dyn MotorController>,
    current_position: f32,
    min_angle: f32,
    max_angle: f32,
}

impl DynamixelServo {
    pub fn new(
        id: u8,
        controller: Box<dyn MotorController>,
        min_angle: f32,
        max_angle: f32,
    ) -> Self {
        Self {
            id,
            controller,
            current_position: 0.0,
            min_angle,
            max_angle,
        }
    }

    fn angle_to_position(&self, angle: f32) -> u16 {
        let range = self.max_angle - self.min_angle;
        let normalized = (angle - self.min_angle) / range;
        (normalized * 4095.0) as u16
    }

    fn position_to_angle(&self, position: u16) -> f32 {
        let range = self.max_angle - self.min_angle;
        let normalized = position as f32 / 4095.0;
        self.min_angle + (normalized * range)
    }
}

impl Motor for DynamixelServo {
    fn set_position(&mut self, angle: f32) -> Result<()> {
        let position = self.angle_to_position(angle);
        let data = [
            self.id,
            30,  // Goal position register
            (position & 0xFF) as u8,
            ((position >> 8) & 0xFF) as u8,
        ];
        self.controller.write_data(0, &data)?;
        self.current_position = angle;
        Ok(())
    }

    fn get_position(&self) -> Result<f32> {
        let mut buffer = [0u8; 2];
        self.controller.read_data(36, &mut buffer)?;
        let position = (buffer[0] as u16) | ((buffer[1] as u16) << 8);
        Ok(self.position_to_angle(position))
    }

    fn enable(&mut self) -> Result<()> {
        let data = [self.id, 24, 1];
        self.controller.write_data(0, &data)
    }

    fn disable(&mut self) -> Result<()> {
        let data = [self.id, 24, 0];
        self.controller.write_data(0, &data)
    }

    fn is_enabled(&self) -> bool {
        true
    }

    fn get_limits(&self) -> (f32, f32) {
        (self.min_angle, self.max_angle)
    }
}
```

## Inverse Kinematics

Add IK solver:

```rust
pub struct IKSolver {
    finger_lengths: Vec<f32>,
}

impl IKSolver {
    pub fn new(finger_lengths: Vec<f32>) -> Self {
        Self { finger_lengths }
    }

    pub fn solve(&self, target: [f32; 3]) -> Result<Vec<f32>> {
        // Simple 2D IK for demonstration
        let l1 = self.finger_lengths[0];
        let l2 = self.finger_lengths[1];
        
        let x = target[0];
        let y = target[1];
        let distance = (x * x + y * y).sqrt();
        
        if distance > l1 + l2 {
            return Err(HandError::Config(
                "Target unreachable".to_string()
            ));
        }
        
        let cos_angle2 = (distance * distance - l1 * l1 - l2 * l2) 
            / (2.0 * l1 * l2);
        let angle2 = cos_angle2.acos();
        
        let k1 = l1 + l2 * cos_angle2;
        let k2 = l2 * angle2.sin();
        let angle1 = y.atan2(x) - k2.atan2(k1);
        
        Ok(vec![angle1.to_degrees(), angle2.to_degrees()])
    }
}
```

## Advanced Motion Control

Implement trajectory smoothing:

```rust
pub struct TrajectorySmoothing {
    points: Vec<Vec<f32>>,
    timestamps: Vec<f32>,
}

impl TrajectorySmoothing {
    pub fn new() -> Self {
        Self {
            points: Vec::new(),
            timestamps: Vec::new(),
        }
    }

    pub fn add_waypoint(&mut self, point: Vec<f32>, time: f32) {
        self.points.push(point);
        self.timestamps.push(time);
    }

    pub fn interpolate(&self, t: f32) -> Option<Vec<f32>> {
        if self.points.is_empty() {
            return None;
        }

        for i in 0..self.timestamps.len() - 1 {
            if t >= self.timestamps[i] && t <= self.timestamps[i + 1] {
                let t0 = self.timestamps[i];
                let t1 = self.timestamps[i + 1];
                let alpha = (t - t0) / (t1 - t0);
                
                let result: Vec<f32> = self.points[i]
                    .iter()
                    .zip(self.points[i + 1].iter())
                    .map(|(&a, &b)| a + (b - a) * alpha)
                    .collect();
                    
                return Some(result);
            }
        }

        None
    }
}
```

## Testing Custom Implementations

Write unit tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_custom_motor() {
        let mut motor = CustomMotor::new(0, 0.0, 90.0);
        assert!(motor.set_position(45.0).is_ok());
        assert_eq!(motor.get_position().unwrap(), 45.0);
    }

    #[test]
    fn test_angle_limits() {
        let mut motor = CustomMotor::new(0, 0.0, 90.0);
        assert!(motor.set_position(100.0).is_err());
        assert!(motor.set_position(-10.0).is_err());
    }
}
```

## Best Practices

1. Always implement proper error handling
2. Validate inputs before hardware operations
3. Test with mock implementations first
4. Document hardware-specific requirements
5. Use feature flags for optional dependencies
6. Maintain backward compatibility in public APIs
7. Add comprehensive tests for new functionality

