use super::forward::ForwardKinematics;
use super::types::{HandGeometry, JointAngles, Position3D};
use crate::error::Result;

pub struct InverseKinematics {
    fk: ForwardKinematics,
    max_iterations: usize,
    tolerance: f32,
}

impl InverseKinematics {
    pub fn new(geometry: HandGeometry, base_position: Position3D) -> Self {
        Self {
            fk: ForwardKinematics::new(geometry, base_position),
            max_iterations: 100,
            tolerance: 0.5,
        }
    }

    pub fn with_default_geometry(base_position: Position3D) -> Self {
        Self::new(HandGeometry::default(), base_position)
    }

    pub fn solve_for_grasp_position(
        &self,
        target: Position3D,
        initial_guess: Option<JointAngles>,
    ) -> Result<JointAngles> {
        let base = self.fk.base_position();
        let distance = base.distance_to(&target);

        let max_reach = self.fk.geometry().finger_links.total_length()
            + self.fk.geometry().palm_length;

        if distance > max_reach {
            return Ok(self.approach_position(target));
        }

        if distance < 2.0 {
            return Ok(JointAngles::open());
        }

        let mut current = initial_guess.unwrap_or_else(|| JointAngles::open());

        for iteration in 0..self.max_iterations {
            let grasp_center = self.fk.compute_grasp_center(&current);
            let error = target.distance_to(&grasp_center);

            if error < self.tolerance {
                return Ok(current);
            }

            let delta_x = target.x - grasp_center.x;
            let delta_y = target.y - grasp_center.y;
            let delta_z = target.z - grasp_center.z;

            let learning_rate = 0.1 * (1.0 - iteration as f32 / self.max_iterations as f32);

            if delta_z > 0.0 {
                current.thumb = (current.thumb - delta_z * learning_rate * 10.0).clamp(0.0, 90.0);
                current.index = (current.index - delta_z * learning_rate * 10.0).clamp(0.0, 90.0);
                current.middle = (current.middle - delta_z * learning_rate * 10.0).clamp(0.0, 90.0);
                current.ring = (current.ring - delta_z * learning_rate * 10.0).clamp(0.0, 90.0);
                current.pinky = (current.pinky - delta_z * learning_rate * 10.0).clamp(0.0, 90.0);
            } else {
                current.thumb = (current.thumb + delta_z.abs() * learning_rate * 10.0).clamp(0.0, 90.0);
                current.index = (current.index + delta_z.abs() * learning_rate * 10.0).clamp(0.0, 90.0);
                current.middle = (current.middle + delta_z.abs() * learning_rate * 10.0).clamp(0.0, 90.0);
                current.ring = (current.ring + delta_z.abs() * learning_rate * 10.0).clamp(0.0, 90.0);
                current.pinky = (current.pinky + delta_z.abs() * learning_rate * 10.0).clamp(0.0, 90.0);
            }

            if let Some(pitch) = current.wrist_pitch {
                let new_pitch = pitch + delta_y * learning_rate * 5.0;
                current.wrist_pitch = Some(new_pitch.clamp(-45.0, 45.0));
            }

            if let Some(roll) = current.wrist_roll {
                let new_roll = roll + delta_x * learning_rate * 5.0;
                current.wrist_roll = Some(new_roll.clamp(-45.0, 45.0));
            }
        }

        Ok(current)
    }

    pub fn solve_for_object_grasp(
        &self,
        object_position: Position3D,
        object_size_cm: f32,
    ) -> Result<JointAngles> {
        let closure_amount = (object_size_cm / 8.0).clamp(0.0, 1.0);
        let base_closure = 90.0 * (1.0 - closure_amount);

        let mut joints = JointAngles::new(
            base_closure * 0.8,
            base_closure,
            base_closure,
            base_closure,
            base_closure * 0.9,
        );

        let approach_vector = Position3D::new(
            object_position.x - self.fk.base_position().x,
            object_position.y - self.fk.base_position().y,
            object_position.z - self.fk.base_position().z,
        );

        let pitch = (approach_vector.z.atan2(
            (approach_vector.x.powi(2) + approach_vector.y.powi(2)).sqrt()
        )).to_degrees();
        let roll = approach_vector.y.atan2(approach_vector.x).to_degrees();

        joints.wrist_pitch = Some(pitch.clamp(-30.0, 30.0));
        joints.wrist_roll = Some(roll.clamp(-30.0, 30.0));

        Ok(joints)
    }

    fn approach_position(&self, target: Position3D) -> JointAngles {
        let base = self.fk.base_position();

        let dx = target.x - base.x;
        let dy = target.y - base.y;
        let dz = target.z - base.z;

        let pitch = dz.atan2((dx.powi(2) + dy.powi(2)).sqrt()).to_degrees();
        let roll = dy.atan2(dx).to_degrees();

        JointAngles::open().with_wrist(
            pitch.clamp(-45.0, 45.0),
            roll.clamp(-45.0, 45.0),
        )
    }

    pub fn update_base_position(&mut self, new_position: Position3D) {
        self.fk.update_base_position(new_position);
    }

    pub fn forward_kinematics(&self) -> &ForwardKinematics {
        &self.fk
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_solve_for_nearby_target() {
        let ik = InverseKinematics::with_default_geometry(Position3D::zero());
        let target = Position3D::new(0.0, 0.0, 15.0);
        let result = ik.solve_for_grasp_position(target, None);

        assert!(result.is_ok());
        let joints = result.unwrap();
        assert!(joints.thumb < 45.0);
    }

    #[test]
    fn test_solve_for_far_target() {
        let ik = InverseKinematics::with_default_geometry(Position3D::zero());
        let target = Position3D::new(0.0, 0.0, 50.0);
        let result = ik.solve_for_grasp_position(target, None);

        assert!(result.is_ok());
        let joints = result.unwrap();
        assert_eq!(joints.thumb, 0.0);
    }

    #[test]
    fn test_object_grasp() {
        let ik = InverseKinematics::with_default_geometry(Position3D::zero());
        let object_pos = Position3D::new(0.0, 0.0, 20.0);
        let result = ik.solve_for_object_grasp(object_pos, 5.0);

        assert!(result.is_ok());
        let joints = result.unwrap();
        assert!(joints.thumb > 0.0 && joints.thumb < 90.0);
    }
}
