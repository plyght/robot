use crate::error::{HandError, Result};
use crate::hardware::Motor;

pub struct Joint {
    motor: Box<dyn Motor>,
    name: String,
    offset: f32,
}

impl Joint {
    pub fn new(motor: Box<dyn Motor>, name: String, offset: f32) -> Self {
        Self {
            motor,
            name,
            offset,
        }
    }

    pub fn set_angle(&mut self, angle: f32) -> Result<()> {
        self.motor.set_position(angle + self.offset)
    }

    pub fn get_angle(&self) -> Result<f32> {
        Ok(self.motor.get_position()? - self.offset)
    }

    pub fn enable(&mut self) -> Result<()> {
        self.motor.enable()
    }

    pub fn disable(&mut self) -> Result<()> {
        self.motor.disable()
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn get_limits(&self) -> (f32, f32) {
        let (min, max) = self.motor.get_limits();
        (min - self.offset, max - self.offset)
    }
}

pub struct Finger {
    id: usize,
    name: String,
    joints: Vec<Joint>,
}

impl Finger {
    pub fn new(id: usize, name: String, joints: Vec<Joint>) -> Self {
        Self { id, name, joints }
    }

    pub fn set_pose(&mut self, angles: &[f32]) -> Result<()> {
        if angles.len() != self.joints.len() {
            return Err(HandError::InvalidJointCount {
                expected: self.joints.len(),
                actual: angles.len(),
            });
        }

        for (joint, &angle) in self.joints.iter_mut().zip(angles.iter()) {
            joint.set_angle(angle)?;
        }
        Ok(())
    }

    pub fn get_pose(&self) -> Result<Vec<f32>> {
        self.joints.iter().map(|j| j.get_angle()).collect()
    }

    pub fn enable(&mut self) -> Result<()> {
        for joint in &mut self.joints {
            joint.enable()?;
        }
        Ok(())
    }

    pub fn disable(&mut self) -> Result<()> {
        for joint in &mut self.joints {
            joint.disable()?;
        }
        Ok(())
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn joint_count(&self) -> usize {
        self.joints.len()
    }

    pub fn get_joint(&self, index: usize) -> Option<&Joint> {
        self.joints.get(index)
    }

    pub fn get_joint_mut(&mut self, index: usize) -> Option<&mut Joint> {
        self.joints.get_mut(index)
    }
}
