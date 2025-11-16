use crate::error::{HandError, Result};
use crate::hand::{Finger, Wrist};

pub struct Hand {
    fingers: Vec<Finger>,
    wrist: Wrist,
    initialized: bool,
}

impl Hand {
    pub fn new(fingers: Vec<Finger>, wrist: Wrist) -> Self {
        Self {
            fingers,
            wrist,
            initialized: false,
        }
    }

    pub fn initialize(&mut self) -> Result<()> {
        for finger in &mut self.fingers {
            finger.enable()?;
        }
        self.wrist.enable()?;
        self.initialized = true;
        Ok(())
    }

    pub fn shutdown(&mut self) -> Result<()> {
        for finger in &mut self.fingers {
            finger.disable()?;
        }
        self.wrist.disable()?;
        self.initialized = false;
        Ok(())
    }

    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    pub fn set_finger_pose(&mut self, finger_id: usize, angles: &[f32]) -> Result<()> {
        let finger = self
            .fingers
            .get_mut(finger_id)
            .ok_or(HandError::InvalidFingerId(finger_id))?;
        finger.set_pose(angles)
    }

    pub fn get_finger_pose(&self, finger_id: usize) -> Result<Vec<f32>> {
        let finger = self
            .fingers
            .get(finger_id)
            .ok_or(HandError::InvalidFingerId(finger_id))?;
        finger.get_pose()
    }

    pub fn set_wrist_orientation(&mut self, pitch: f32, roll: f32, yaw: f32) -> Result<()> {
        self.wrist.set_orientation(pitch, roll, yaw)
    }

    pub fn get_wrist_orientation(&self) -> (f32, f32, f32) {
        self.wrist.get_orientation()
    }

    pub fn finger_count(&self) -> usize {
        self.fingers.len()
    }

    pub fn get_finger(&self, finger_id: usize) -> Option<&Finger> {
        self.fingers.get(finger_id)
    }

    pub fn get_finger_mut(&mut self, finger_id: usize) -> Option<&mut Finger> {
        self.fingers.get_mut(finger_id)
    }

    pub fn wrist(&self) -> &Wrist {
        &self.wrist
    }

    pub fn wrist_mut(&mut self) -> &mut Wrist {
        &mut self.wrist
    }
}
