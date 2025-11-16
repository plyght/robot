use crate::error::{HandError, Result};
use crate::hardware::motor::{Motor, MotorController};

pub struct PwmServo {
    channel: u8,
    current_position: f32,
    enabled: bool,
    min_angle: f32,
    max_angle: f32,
    min_pulse: u16,
    max_pulse: u16,
    controller: Box<dyn MotorController>,
}

impl PwmServo {
    pub fn new(
        channel: u8,
        min_angle: f32,
        max_angle: f32,
        min_pulse: u16,
        max_pulse: u16,
        controller: Box<dyn MotorController>,
    ) -> Self {
        Self {
            channel,
            current_position: 0.0,
            enabled: false,
            min_angle,
            max_angle,
            min_pulse,
            max_pulse,
            controller,
        }
    }

    fn angle_to_pulse(&self, angle: f32) -> u16 {
        let range = self.max_angle - self.min_angle;
        let normalized = (angle - self.min_angle) / range;
        let pulse_range = self.max_pulse - self.min_pulse;
        self.min_pulse + (normalized * pulse_range as f32) as u16
    }
}

impl Motor for PwmServo {
    fn set_position(&mut self, angle: f32) -> Result<()> {
        if angle < self.min_angle || angle > self.max_angle {
            return Err(HandError::InvalidJointAngle {
                joint_id: self.channel as usize,
                angle,
                min: self.min_angle,
                max: self.max_angle,
            });
        }

        let pulse = self.angle_to_pulse(angle);
        self.controller.write_pwm(self.channel, pulse)?;
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

pub struct StepperMotor {
    id: usize,
    current_position: f32,
    enabled: bool,
    min_angle: f32,
    max_angle: f32,
    steps_per_revolution: u32,
}

impl StepperMotor {
    pub fn new(id: usize, min_angle: f32, max_angle: f32, steps_per_revolution: u32) -> Self {
        Self {
            id,
            current_position: 0.0,
            enabled: false,
            min_angle,
            max_angle,
            steps_per_revolution,
        }
    }

    fn angle_to_steps(&self, angle: f32) -> i32 {
        let range = self.max_angle - self.min_angle;
        let normalized = (angle - self.min_angle) / range;
        (normalized * self.steps_per_revolution as f32) as i32
    }

    fn steps_to_angle(&self, steps: i32) -> f32 {
        let range = self.max_angle - self.min_angle;
        let normalized = steps as f32 / self.steps_per_revolution as f32;
        self.min_angle + (normalized * range)
    }

    pub fn get_steps_per_revolution(&self) -> u32 {
        self.steps_per_revolution
    }

    pub fn get_current_steps(&self) -> i32 {
        self.angle_to_steps(self.current_position)
    }

    pub fn set_steps(&mut self, steps: i32) -> Result<()> {
        let angle = self.steps_to_angle(steps);
        self.set_position(angle)
    }
}

impl Motor for StepperMotor {
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

pub struct DcMotor {
    id: usize,
    current_position: f32,
    enabled: bool,
    min_angle: f32,
    max_angle: f32,
}

impl DcMotor {
    pub fn new(id: usize, min_angle: f32, max_angle: f32) -> Self {
        Self {
            id,
            current_position: 0.0,
            enabled: false,
            min_angle,
            max_angle,
        }
    }
}

impl Motor for DcMotor {
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
