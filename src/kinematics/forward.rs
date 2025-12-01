use super::types::{HandGeometry, JointAngles, Position3D};

pub struct ForwardKinematics {
    geometry: HandGeometry,
    base_position: Position3D,
}

impl ForwardKinematics {
    pub fn new(geometry: HandGeometry, base_position: Position3D) -> Self {
        Self {
            geometry,
            base_position,
        }
    }

    pub fn with_default_geometry(base_position: Position3D) -> Self {
        Self::new(HandGeometry::default(), base_position)
    }

    pub fn compute_palm_center(&self, joint_angles: &JointAngles) -> Position3D {
        let wrist_pitch = joint_angles.wrist_pitch.unwrap_or(0.0);
        let wrist_roll = joint_angles.wrist_roll.unwrap_or(0.0);

        let pitch_rad = wrist_pitch.to_radians();
        let roll_rad = wrist_roll.to_radians();

        let x = self.base_position.x
            + self.geometry.palm_length * pitch_rad.cos() * roll_rad.cos();
        let y = self.base_position.y
            + self.geometry.palm_length * roll_rad.sin();
        let z = self.base_position.z
            + self.geometry.palm_length * pitch_rad.sin();

        Position3D::new(x, y, z)
    }

    pub fn compute_finger_tip_position(
        &self,
        finger_index: usize,
        angle: f32,
        joint_angles: &JointAngles,
    ) -> Position3D {
        let palm_center = self.compute_palm_center(joint_angles);

        let links = if finger_index == 0 {
            self.geometry.thumb_links
        } else {
            self.geometry.finger_links
        };

        let angle_rad = angle.to_radians();

        let finger_offset = if finger_index == 0 {
            self.geometry.thumb_offset_x
        } else {
            (finger_index as f32 - 2.0) * self.geometry.finger_spacing
        };

        let extension = links.total_length() * (1.0 - angle / 90.0);

        let x = palm_center.x + finger_offset;
        let y = palm_center.y + if finger_index == 0 {
            self.geometry.thumb_offset_y
        } else {
            0.0
        };
        let z = palm_center.z + extension;

        Position3D::new(x, y, z)
    }

    pub fn compute_all_finger_tips(&self, joint_angles: &JointAngles) -> Vec<Position3D> {
        let angles = [
            joint_angles.thumb,
            joint_angles.index,
            joint_angles.middle,
            joint_angles.ring,
            joint_angles.pinky,
        ];

        angles
            .iter()
            .enumerate()
            .map(|(i, &angle)| self.compute_finger_tip_position(i, angle, joint_angles))
            .collect()
    }

    pub fn compute_grasp_center(&self, joint_angles: &JointAngles) -> Position3D {
        let finger_tips = self.compute_all_finger_tips(joint_angles);

        let sum_x: f32 = finger_tips.iter().map(|p| p.x).sum();
        let sum_y: f32 = finger_tips.iter().map(|p| p.y).sum();
        let sum_z: f32 = finger_tips.iter().map(|p| p.z).sum();
        let count = finger_tips.len() as f32;

        Position3D::new(sum_x / count, sum_y / count, sum_z / count)
    }

    pub fn update_base_position(&mut self, new_position: Position3D) {
        self.base_position = new_position;
    }

    pub fn base_position(&self) -> Position3D {
        self.base_position
    }

    pub fn geometry(&self) -> &HandGeometry {
        &self.geometry
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_palm_center_at_zero() {
        let fk = ForwardKinematics::with_default_geometry(Position3D::zero());
        let joints = JointAngles::open();
        let palm = fk.compute_palm_center(&joints);

        assert!((palm.x - 0.0).abs() < 0.1);
        assert!((palm.y - 0.0).abs() < 0.1);
    }

    #[test]
    fn test_finger_tips_extended() {
        let fk = ForwardKinematics::with_default_geometry(Position3D::zero());
        let joints = JointAngles::open();
        let tips = fk.compute_all_finger_tips(&joints);

        assert_eq!(tips.len(), 5);
        for tip in tips.iter() {
            assert!(tip.z > 5.0);
        }
    }

    #[test]
    fn test_finger_tips_closed() {
        let fk = ForwardKinematics::with_default_geometry(Position3D::zero());
        let joints = JointAngles::closed();
        let tips = fk.compute_all_finger_tips(&joints);

        for tip in tips.iter() {
            assert!(tip.z < 5.0);
        }
    }
}
