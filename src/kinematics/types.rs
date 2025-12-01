use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Position3D {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Position3D {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0, z: 0.0 }
    }

    pub fn distance_to(&self, other: &Position3D) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Orientation {
    pub pitch: f32,
    pub roll: f32,
    pub yaw: f32,
}

impl Orientation {
    pub fn new(pitch: f32, roll: f32, yaw: f32) -> Self {
        Self { pitch, roll, yaw }
    }

    pub fn zero() -> Self {
        Self { pitch: 0.0, roll: 0.0, yaw: 0.0 }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JointAngles {
    pub thumb: f32,
    pub index: f32,
    pub middle: f32,
    pub ring: f32,
    pub pinky: f32,
    pub wrist_pitch: Option<f32>,
    pub wrist_roll: Option<f32>,
}

impl JointAngles {
    pub fn new(thumb: f32, index: f32, middle: f32, ring: f32, pinky: f32) -> Self {
        Self {
            thumb,
            index,
            middle,
            ring,
            pinky,
            wrist_pitch: None,
            wrist_roll: None,
        }
    }

    pub fn with_wrist(mut self, pitch: f32, roll: f32) -> Self {
        self.wrist_pitch = Some(pitch);
        self.wrist_roll = Some(roll);
        self
    }

    pub fn open() -> Self {
        Self::new(0.0, 0.0, 0.0, 0.0, 0.0)
    }

    pub fn closed() -> Self {
        Self::new(90.0, 90.0, 90.0, 90.0, 90.0)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HandPose {
    pub position: Position3D,
    pub orientation: Orientation,
    pub joint_angles: JointAngles,
}

impl HandPose {
    pub fn new(position: Position3D, orientation: Orientation, joint_angles: JointAngles) -> Self {
        Self {
            position,
            orientation,
            joint_angles,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FingerLinkLengths {
    pub proximal: f32,
    pub middle: f32,
    pub distal: f32,
}

impl FingerLinkLengths {
    pub fn new(proximal: f32, middle: f32, distal: f32) -> Self {
        Self { proximal, middle, distal }
    }

    pub fn total_length(&self) -> f32 {
        self.proximal + self.middle + self.distal
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HandGeometry {
    pub palm_width: f32,
    pub palm_length: f32,
    pub thumb_offset_x: f32,
    pub thumb_offset_y: f32,
    pub finger_spacing: f32,
    pub thumb_links: FingerLinkLengths,
    pub finger_links: FingerLinkLengths,
}

impl Default for HandGeometry {
    fn default() -> Self {
        Self {
            palm_width: 8.0,
            palm_length: 10.0,
            thumb_offset_x: -2.0,
            thumb_offset_y: 3.0,
            finger_spacing: 2.0,
            thumb_links: FingerLinkLengths::new(3.5, 2.5, 2.0),
            finger_links: FingerLinkLengths::new(4.0, 3.0, 2.5),
        }
    }
}
