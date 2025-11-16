pub mod grip_patterns;

pub use grip_patterns::{GripPattern, GripPatternType};

use crate::error::Result;

#[derive(Debug, Clone)]
pub struct DetectedObject {
    pub label: String,
    pub confidence: f32,
    pub bounding_box: BoundingBox,
    pub distance: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct BoundingBox {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl BoundingBox {
    pub fn center(&self) -> (i32, i32) {
        (self.x + self.width / 2, self.y + self.height / 2)
    }

    pub fn area(&self) -> i32 {
        self.width * self.height
    }
}

pub trait ObjectDetector {
    fn detect_objects(&mut self) -> Result<Vec<DetectedObject>>;
    fn get_frame_size(&self) -> (i32, i32);
}

pub struct MockObjectDetector {
    frame_width: i32,
    frame_height: i32,
    mock_objects: Vec<DetectedObject>,
}

impl MockObjectDetector {
    pub fn new(frame_width: i32, frame_height: i32) -> Self {
        Self {
            frame_width,
            frame_height,
            mock_objects: Vec::new(),
        }
    }

    pub fn add_mock_object(&mut self, obj: DetectedObject) {
        self.mock_objects.push(obj);
    }

    pub fn clear_mock_objects(&mut self) {
        self.mock_objects.clear();
    }
}

impl ObjectDetector for MockObjectDetector {
    fn detect_objects(&mut self) -> Result<Vec<DetectedObject>> {
        Ok(self.mock_objects.clone())
    }

    fn get_frame_size(&self) -> (i32, i32) {
        (self.frame_width, self.frame_height)
    }
}

#[cfg(feature = "opencv")]
pub struct OpenCVDetector {
    frame_width: i32,
    frame_height: i32,
    confidence_threshold: f32,
}

#[cfg(feature = "opencv")]
impl OpenCVDetector {
    pub fn new(camera_id: i32, confidence_threshold: f32) -> Result<Self> {
        Ok(Self {
            frame_width: 640,
            frame_height: 480,
            confidence_threshold,
        })
    }
}

#[cfg(feature = "opencv")]
impl ObjectDetector for OpenCVDetector {
    fn detect_objects(&mut self) -> Result<Vec<DetectedObject>> {
        Ok(Vec::new())
    }

    fn get_frame_size(&self) -> (i32, i32) {
        (self.frame_width, self.frame_height)
    }
}

pub fn select_best_object(
    objects: &[DetectedObject],
    frame_center: (i32, i32),
) -> Option<&DetectedObject> {
    if objects.is_empty() {
        return None;
    }

    objects
        .iter()
        .max_by_key(|obj| {
            let (obj_x, obj_y) = obj.bounding_box.center();
            let dx = (obj_x - frame_center.0).abs();
            let dy = (obj_y - frame_center.1).abs();
            let distance_to_center = ((dx * dx + dy * dy) as f32).sqrt();

            let score = (obj.confidence * 100.0) as i32 - (distance_to_center / 10.0) as i32;
            score
        })
}

pub fn classify_object_type(label: &str) -> Option<&'static str> {
    let label_lower = label.to_lowercase();
    
    if label_lower.contains("cup")
        || label_lower.contains("mug")
        || label_lower.contains("glass")
    {
        Some("cup")
    } else if label_lower.contains("bottle") {
        Some("bottle")
    } else if label_lower.contains("phone")
        || label_lower.contains("cellphone")
        || label_lower.contains("mobile")
    {
        Some("phone")
    } else if label_lower.contains("book") || label_lower.contains("notebook") {
        Some("book")
    } else if label_lower.contains("pen") || label_lower.contains("pencil") {
        Some("pen")
    } else {
        Some("small_object")
    }
}

