use crate::control::motion::MotionPlanner;
use crate::error::Result;
use crate::protocol::ServoProtocol;
use crate::vision::GripPattern;
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SequenceStep {
    Approach,
    Open,
    Grasp,
    Lift,
    Move,
    Release,
    Complete,
}

pub struct PickupSequence {
    current_step: SequenceStep,
    grip_pattern: GripPattern,
    #[allow(dead_code)]
    motion_planner: MotionPlanner,
}

impl PickupSequence {
    pub fn new(grip_pattern: GripPattern) -> Self {
        Self {
            current_step: SequenceStep::Approach,
            grip_pattern,
            motion_planner: MotionPlanner::default(),
        }
    }

    pub fn current_step(&self) -> SequenceStep {
        self.current_step
    }

    pub fn is_complete(&self) -> bool {
        self.current_step == SequenceStep::Complete
    }

    pub fn execute<P: ServoProtocol>(
        &mut self,
        protocol: &mut P,
        finger_to_servo_map: &std::collections::HashMap<String, u8>,
    ) -> Result<()> {
        match self.current_step {
            SequenceStep::Approach => {
                println!("→ Approaching object...");
                thread::sleep(Duration::from_millis(500));
                self.current_step = SequenceStep::Open;
            }
            SequenceStep::Open => {
                println!("→ Opening hand...");
                self.open_hand(protocol, finger_to_servo_map)?;
                thread::sleep(Duration::from_millis(800));
                self.current_step = SequenceStep::Grasp;
            }
            SequenceStep::Grasp => {
                println!("→ Grasping object...");
                self.grasp_object(protocol, finger_to_servo_map)?;
                thread::sleep(Duration::from_millis(1000));
                self.current_step = SequenceStep::Lift;
            }
            SequenceStep::Lift => {
                println!("→ Lifting object...");
                thread::sleep(Duration::from_millis(800));
                self.current_step = SequenceStep::Move;
            }
            SequenceStep::Move => {
                println!("→ Moving to target position...");
                thread::sleep(Duration::from_millis(600));
                self.current_step = SequenceStep::Release;
            }
            SequenceStep::Release => {
                println!("→ Releasing object...");
                self.open_hand(protocol, finger_to_servo_map)?;
                thread::sleep(Duration::from_millis(500));
                self.current_step = SequenceStep::Complete;
            }
            SequenceStep::Complete => {
                println!("✓ Pickup sequence complete!");
            }
        }

        Ok(())
    }

    fn open_hand<P: ServoProtocol>(
        &self,
        protocol: &mut P,
        finger_to_servo_map: &std::collections::HashMap<String, u8>,
    ) -> Result<()> {
        let fingers = ["Thumb", "Index", "Middle", "Ring", "Pinky"];
        for finger in &fingers {
            if let Some(&servo_id) = finger_to_servo_map.get(*finger) {
                protocol.send_servo_command(servo_id, finger, 0.0)?;
            }
        }
        Ok(())
    }

    fn grasp_object<P: ServoProtocol>(
        &self,
        protocol: &mut P,
        finger_to_servo_map: &std::collections::HashMap<String, u8>,
    ) -> Result<()> {
        for (finger_name, angles) in &self.grip_pattern.finger_angles {
            if let Some(&servo_id) = finger_to_servo_map.get(finger_name) {
                let target_angle = angles.first().copied().unwrap_or(0.0);
                protocol.send_servo_command(servo_id, finger_name, target_angle)?;
                thread::sleep(Duration::from_millis(50));
            }
        }
        Ok(())
    }

    pub fn execute_step_by_step<P: ServoProtocol>(
        &mut self,
        protocol: &mut P,
        finger_to_servo_map: &std::collections::HashMap<String, u8>,
    ) -> Result<bool> {
        if self.is_complete() {
            return Ok(true);
        }

        self.execute(protocol, finger_to_servo_map)?;
        Ok(self.is_complete())
    }

    pub fn reset(&mut self) {
        self.current_step = SequenceStep::Approach;
    }
}

pub fn create_default_finger_servo_map() -> std::collections::HashMap<String, u8> {
    let mut map = std::collections::HashMap::new();
    map.insert("Thumb".to_string(), 0);
    map.insert("Index".to_string(), 1);
    map.insert("Middle".to_string(), 2);
    map.insert("Ring".to_string(), 3);
    map.insert("Pinky".to_string(), 4);
    map
}
