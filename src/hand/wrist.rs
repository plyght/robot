use crate::error::Result;
use crate::hardware::Motor;

pub struct Wrist {
    pitch_motor: Option<Box<dyn Motor>>,
    roll_motor: Option<Box<dyn Motor>>,
    yaw_motor: Option<Box<dyn Motor>>,
    current_pitch: f32,
    current_roll: f32,
    current_yaw: f32,
}

impl Wrist {
    pub fn new(
        pitch_motor: Option<Box<dyn Motor>>,
        roll_motor: Option<Box<dyn Motor>>,
        yaw_motor: Option<Box<dyn Motor>>,
    ) -> Self {
        Self {
            pitch_motor,
            roll_motor,
            yaw_motor,
            current_pitch: 0.0,
            current_roll: 0.0,
            current_yaw: 0.0,
        }
    }

    pub fn set_orientation(&mut self, pitch: f32, roll: f32, yaw: f32) -> Result<()> {
        if let Some(motor) = &mut self.pitch_motor {
            motor.set_position(pitch)?;
            self.current_pitch = pitch;
        }
        if let Some(motor) = &mut self.roll_motor {
            motor.set_position(roll)?;
            self.current_roll = roll;
        }
        if let Some(motor) = &mut self.yaw_motor {
            motor.set_position(yaw)?;
            self.current_yaw = yaw;
        }
        Ok(())
    }

    pub fn set_pitch(&mut self, pitch: f32) -> Result<()> {
        if let Some(motor) = &mut self.pitch_motor {
            motor.set_position(pitch)?;
            self.current_pitch = pitch;
        }
        Ok(())
    }

    pub fn set_roll(&mut self, roll: f32) -> Result<()> {
        if let Some(motor) = &mut self.roll_motor {
            motor.set_position(roll)?;
            self.current_roll = roll;
        }
        Ok(())
    }

    pub fn set_yaw(&mut self, yaw: f32) -> Result<()> {
        if let Some(motor) = &mut self.yaw_motor {
            motor.set_position(yaw)?;
            self.current_yaw = yaw;
        }
        Ok(())
    }

    pub fn get_orientation(&self) -> (f32, f32, f32) {
        (self.current_pitch, self.current_roll, self.current_yaw)
    }

    pub fn enable(&mut self) -> Result<()> {
        if let Some(motor) = &mut self.pitch_motor {
            motor.enable()?;
        }
        if let Some(motor) = &mut self.roll_motor {
            motor.enable()?;
        }
        if let Some(motor) = &mut self.yaw_motor {
            motor.enable()?;
        }
        Ok(())
    }

    pub fn disable(&mut self) -> Result<()> {
        if let Some(motor) = &mut self.pitch_motor {
            motor.disable()?;
        }
        if let Some(motor) = &mut self.roll_motor {
            motor.disable()?;
        }
        if let Some(motor) = &mut self.yaw_motor {
            motor.disable()?;
        }
        Ok(())
    }

    pub fn has_pitch(&self) -> bool {
        self.pitch_motor.is_some()
    }

    pub fn has_roll(&self) -> bool {
        self.roll_motor.is_some()
    }

    pub fn has_yaw(&self) -> bool {
        self.yaw_motor.is_some()
    }
}
