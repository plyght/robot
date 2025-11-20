use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GripPatternType {
    PowerGrasp,
    PrecisionGrip,
    PinchGrip,
    LateralGrip,
    TripodGrip,
}

#[derive(Debug, Clone)]
pub struct GripPattern {
    pub pattern_type: GripPatternType,
    pub finger_angles: HashMap<String, Vec<f32>>,
    pub wrist_orientation: Option<[f32; 3]>,
    pub approach_distance: f32,
}

impl GripPattern {
    pub fn power_grasp() -> Self {
        let mut finger_angles = HashMap::new();
        finger_angles.insert("Thumb".to_string(), vec![60.0, 60.0, 60.0]);
        finger_angles.insert("Index".to_string(), vec![80.0, 80.0, 80.0]);
        finger_angles.insert("Middle".to_string(), vec![80.0, 80.0, 80.0]);
        finger_angles.insert("Ring".to_string(), vec![80.0, 80.0, 80.0]);
        finger_angles.insert("Pinky".to_string(), vec![80.0, 80.0, 80.0]);

        Self {
            pattern_type: GripPatternType::PowerGrasp,
            finger_angles,
            wrist_orientation: Some([0.0, 0.0, 0.0]),
            approach_distance: 10.0,
        }
    }

    pub fn precision_grip() -> Self {
        let mut finger_angles = HashMap::new();
        finger_angles.insert("Thumb".to_string(), vec![45.0, 45.0, 30.0]);
        finger_angles.insert("Index".to_string(), vec![60.0, 50.0, 40.0]);
        finger_angles.insert("Middle".to_string(), vec![60.0, 50.0, 40.0]);
        finger_angles.insert("Ring".to_string(), vec![10.0, 10.0, 10.0]);
        finger_angles.insert("Pinky".to_string(), vec![10.0, 10.0, 10.0]);

        Self {
            pattern_type: GripPatternType::PrecisionGrip,
            finger_angles,
            wrist_orientation: Some([5.0, 0.0, 0.0]),
            approach_distance: 8.0,
        }
    }

    pub fn pinch_grip() -> Self {
        let mut finger_angles = HashMap::new();
        finger_angles.insert("Thumb".to_string(), vec![50.0, 40.0, 30.0]);
        finger_angles.insert("Index".to_string(), vec![70.0, 60.0, 50.0]);
        finger_angles.insert("Middle".to_string(), vec![20.0, 20.0, 20.0]);
        finger_angles.insert("Ring".to_string(), vec![10.0, 10.0, 10.0]);
        finger_angles.insert("Pinky".to_string(), vec![10.0, 10.0, 10.0]);

        Self {
            pattern_type: GripPatternType::PinchGrip,
            finger_angles,
            wrist_orientation: Some([0.0, 0.0, 0.0]),
            approach_distance: 6.0,
        }
    }

    pub fn lateral_grip() -> Self {
        let mut finger_angles = HashMap::new();
        finger_angles.insert("Thumb".to_string(), vec![80.0, 70.0, 60.0]);
        finger_angles.insert("Index".to_string(), vec![90.0, 90.0, 90.0]);
        finger_angles.insert("Middle".to_string(), vec![20.0, 20.0, 20.0]);
        finger_angles.insert("Ring".to_string(), vec![10.0, 10.0, 10.0]);
        finger_angles.insert("Pinky".to_string(), vec![10.0, 10.0, 10.0]);

        Self {
            pattern_type: GripPatternType::LateralGrip,
            finger_angles,
            wrist_orientation: Some([0.0, 10.0, 0.0]),
            approach_distance: 7.0,
        }
    }

    pub fn tripod_grip() -> Self {
        let mut finger_angles = HashMap::new();
        finger_angles.insert("Thumb".to_string(), vec![45.0, 40.0, 35.0]);
        finger_angles.insert("Index".to_string(), vec![65.0, 55.0, 45.0]);
        finger_angles.insert("Middle".to_string(), vec![65.0, 55.0, 45.0]);
        finger_angles.insert("Ring".to_string(), vec![15.0, 15.0, 15.0]);
        finger_angles.insert("Pinky".to_string(), vec![10.0, 10.0, 10.0]);

        Self {
            pattern_type: GripPatternType::TripodGrip,
            finger_angles,
            wrist_orientation: Some([3.0, 0.0, 0.0]),
            approach_distance: 7.0,
        }
    }

    pub fn for_object_type(object_type: &str) -> Self {
        match object_type {
            "cup" | "mug" | "glass" => Self::power_grasp(),
            "bottle" => Self::power_grasp(),
            "phone" | "book" => Self::precision_grip(),
            "pen" | "pencil" => Self::pinch_grip(),
            "card" => Self::lateral_grip(),
            _ => Self::power_grasp(),
        }
    }
}

pub fn get_object_grip_mapping() -> HashMap<&'static str, GripPatternType> {
    let mut mapping = HashMap::new();
    mapping.insert("cup", GripPatternType::PowerGrasp);
    mapping.insert("mug", GripPatternType::PowerGrasp);
    mapping.insert("bottle", GripPatternType::PowerGrasp);
    mapping.insert("phone", GripPatternType::PrecisionGrip);
    mapping.insert("book", GripPatternType::PrecisionGrip);
    mapping.insert("pen", GripPatternType::PinchGrip);
    mapping.insert("pencil", GripPatternType::PinchGrip);
    mapping.insert("card", GripPatternType::LateralGrip);
    mapping.insert("small_object", GripPatternType::PowerGrasp);
    mapping
}
