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
use ort::session::Session;

#[cfg(feature = "opencv")]
pub struct OpenCVDetector {
    camera: opencv::videoio::VideoCapture,
    frame_width: i32,
    frame_height: i32,
    confidence_threshold: f32,
    class_names: Vec<String>,
    session: Option<Session>,
    input_size: (i32, i32),
}

#[cfg(feature = "opencv")]
impl OpenCVDetector {
    pub fn new(camera_id: i32, confidence_threshold: f32) -> Result<Self> {
        use opencv::{videoio, prelude::*};
        use crate::error::HandError;
        
        let mut camera = videoio::VideoCapture::new(camera_id, videoio::CAP_ANY)
            .map_err(|e| HandError::Hardware(format!("Failed to open camera: {}", e)))?;
        
        if !camera.is_opened()
            .map_err(|e| HandError::Hardware(format!("Camera not opened: {}", e)))? 
        {
            return Err(HandError::Hardware("Camera failed to open".to_string()));
        }
        
        let frame_width = camera.get(videoio::CAP_PROP_FRAME_WIDTH)
            .map_err(|e| HandError::Hardware(format!("Failed to get frame width: {}", e)))? as i32;
        let frame_height = camera.get(videoio::CAP_PROP_FRAME_HEIGHT)
            .map_err(|e| HandError::Hardware(format!("Failed to get frame height: {}", e)))? as i32;
        
        let class_names = vec![
            "person", "bicycle", "car", "motorcycle", "airplane", "bus", "train", "truck", "boat",
            "traffic light", "fire hydrant", "stop sign", "parking meter", "bench", "bird", "cat",
            "dog", "horse", "sheep", "cow", "elephant", "bear", "zebra", "giraffe", "backpack",
            "umbrella", "handbag", "tie", "suitcase", "frisbee", "skis", "snowboard", "sports ball",
            "kite", "baseball bat", "baseball glove", "skateboard", "surfboard", "tennis racket",
            "bottle", "wine glass", "cup", "fork", "knife", "spoon", "bowl", "banana", "apple",
            "sandwich", "orange", "broccoli", "carrot", "hot dog", "pizza", "donut", "cake",
            "chair", "couch", "potted plant", "bed", "dining table", "toilet", "tv", "laptop",
            "mouse", "remote", "keyboard", "cell phone", "microwave", "oven", "toaster", "sink",
            "refrigerator", "book", "clock", "vase", "scissors", "teddy bear", "hair drier",
            "toothbrush"
        ].iter().map(|s| s.to_string()).collect();
        
        Ok(Self {
            camera,
            frame_width,
            frame_height,
            confidence_threshold,
            class_names,
            session: None,
            input_size: (640, 640),
        })
    }
    
    pub fn load_yolo_model(&mut self, model_path: &str) -> Result<()> {
        use crate::error::HandError;
        
        let session = Session::builder()
            .map_err(|e| HandError::Hardware(format!("Failed to create session builder: {}", e)))?
            .commit_from_file(model_path)
            .map_err(|e| HandError::Hardware(format!("Failed to load YOLO model: {}", e)))?;
        self.session = Some(session);
        println!("Loaded YOLO model from: {}", model_path);
        Ok(())
    }
    
    
    pub fn get_frame(&mut self) -> Result<opencv::core::Mat> {
        use crate::error::HandError;
        use opencv::prelude::VideoCaptureTrait;
        
        let mut frame = opencv::core::Mat::default();
        self.camera.read(&mut frame)
            .map_err(|e| HandError::Hardware(format!("Failed to read frame: {}", e)))?;
        
        Ok(frame)
    }
    
    fn detect_with_yolo(&mut self, frame: &opencv::core::Mat) -> Result<Vec<DetectedObject>> {
        use opencv::{core, imgproc, prelude::*};
        use crate::error::HandError;
        use ort::value::Value;
        
        let session = match &mut self.session {
            Some(s) => s,
            None => return Ok(Vec::new()),
        };
        
        let mut resized = core::Mat::default();
        imgproc::resize(
            frame,
            &mut resized,
            core::Size::new(self.input_size.0, self.input_size.1),
            0.0,
            0.0,
            imgproc::INTER_LINEAR,
        )
        .map_err(|e| HandError::Hardware(format!("Failed to resize frame: {}", e)))?;
        
        let mut rgb = core::Mat::default();
        imgproc::cvt_color_def(&resized, &mut rgb, imgproc::COLOR_BGR2RGB)
            .map_err(|e| HandError::Hardware(format!("Failed to convert color: {}", e)))?;
        
        let (rows, cols) = (rgb.rows(), rgb.cols());
        let mut input_data = Vec::<f32>::with_capacity((rows * cols * 3) as usize);
        
        for c in 0..3 {
            for y in 0..rows {
                for x in 0..cols {
                    let pixel = rgb.at_2d::<core::Vec3b>(y, x)
                        .map_err(|e| HandError::Hardware(format!("Failed to access pixel: {}", e)))?;
                    input_data.push(pixel[c] as f32 / 255.0);
                }
            }
        }
        
        let input_array = ndarray::Array4::from_shape_vec((1, 3, self.input_size.1 as usize, self.input_size.0 as usize), input_data)
            .map_err(|e| HandError::Hardware(format!("Failed to create input array: {}", e)))?;
        
        let input_tensor = Value::from_array(input_array)
            .map_err(|e| HandError::Hardware(format!("Failed to create tensor: {}", e)))?;
        
        let outputs = session.run(ort::inputs![input_tensor])
            .map_err(|e| HandError::Hardware(format!("Inference failed: {}", e)))?;
        
        let (_name, output_value) = outputs.iter().next()
            .ok_or_else(|| HandError::Hardware("No output from model".to_string()))?;
        
        let (shape, data) = output_value
            .try_extract_tensor::<f32>()
            .map_err(|e| HandError::Hardware(format!("Failed to extract output: {}", e)))?;
        
        let shape_vec: Vec<usize> = shape.iter().map(|&d| d as usize).collect();
        if shape_vec.len() < 3 {
            return Ok(Vec::new());
        }
        
        let batch_size = shape_vec[0];
        let dim1 = shape_vec[1];
        let dim2 = shape_vec[2];
        
        let (num_detections, num_features) = if dim1 == 84 || dim1 > dim2 {
            (dim2, dim1)
        } else {
            (dim1, dim2)
        };
        
        if batch_size != 1 || num_detections == 0 || num_features < 84 {
            return Ok(Vec::new());
        }
        
        let scale_x = frame.cols() as f32 / self.input_size.0 as f32;
        let scale_y = frame.rows() as f32 / self.input_size.1 as f32;
        let mut candidates = Vec::new();
        
        let transposed = dim1 == 84 || (dim1 > 80 && dim1 < dim2);
        
        for i in 0..num_detections {
            let x_center_raw = if transposed {
                data[0 * num_detections + i]
            } else {
                data[i * num_features + 0]
            };
            let y_center_raw = if transposed {
                data[1 * num_detections + i]
            } else {
                data[i * num_features + 1]
            };
            let width_raw = if transposed {
                data[2 * num_detections + i]
            } else {
                data[i * num_features + 2]
            };
            let height_raw = if transposed {
                data[3 * num_detections + i]
            } else {
                data[i * num_features + 3]
            };
            
            let mut max_score = 0.0f32;
            let mut class_id = 0usize;
            
            for j in 4..num_features.min(84) {
                let logit = if transposed {
                    data[j * num_detections + i]
                } else {
                    data[i * num_features + j]
                };
                let score = 1.0 / (1.0 + (-logit).exp());
                if score > max_score {
                    max_score = score;
                    class_id = j - 4;
                }
            }
            
            
            if max_score < 0.6 {
                continue;
            }
            
            {
                let x = ((x_center_raw - width_raw / 2.0) * scale_x) as i32;
                let y = ((y_center_raw - height_raw / 2.0) * scale_y) as i32;
                let w = (width_raw * scale_x) as i32;
                let h = (height_raw * scale_y) as i32;
                
                if w <= 0 || h <= 0 {
                    continue;
                }
                
                if x < -w || y < -h || x >= frame.cols() + w || y >= frame.rows() + h {
                    continue;
                }
                
                let label = if class_id < self.class_names.len() {
                    self.class_names[class_id].clone()
                } else {
                    format!("class_{}", class_id)
                };
                
                candidates.push(DetectedObject {
                    label,
                    confidence: max_score,
                    bounding_box: BoundingBox {
                        x: x.max(0),
                        y: y.max(0),
                        width: w.min(frame.cols() - x.max(0)),
                        height: h.min(frame.rows() - y.max(0)),
                    },
                    distance: 0.5,
                });
            }
        }
        
        let detections = apply_nms(candidates, 0.5);
        Ok(detections)
    }
}

#[cfg(feature = "opencv")]
impl ObjectDetector for OpenCVDetector {
    fn detect_objects(&mut self) -> Result<Vec<DetectedObject>> {
        let frame = self.get_frame()?;
        self.detect_with_yolo(&frame)
    }

    fn get_frame_size(&self) -> (i32, i32) {
        (self.frame_width, self.frame_height)
    }
}

fn apply_nms(mut candidates: Vec<DetectedObject>, iou_threshold: f32) -> Vec<DetectedObject> {
    if candidates.is_empty() {
        return Vec::new();
    }
    
    candidates.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal));
    
    let mut detections = Vec::new();
    let mut used = vec![false; candidates.len()];
    
    for i in 0..candidates.len() {
        if used[i] {
            continue;
        }
        
        detections.push(candidates[i].clone());
        used[i] = true;
        
        for j in (i + 1)..candidates.len() {
            if used[j] {
                continue;
            }
            
            let iou = calculate_iou(&candidates[i].bounding_box, &candidates[j].bounding_box);
            if iou > iou_threshold {
                used[j] = true;
            }
        }
    }
    
    detections
}

fn calculate_iou(box1: &BoundingBox, box2: &BoundingBox) -> f32 {
    let x1 = box1.x.max(box2.x);
    let y1 = box1.y.max(box2.y);
    let x2 = (box1.x + box1.width).min(box2.x + box2.width);
    let y2 = (box1.y + box1.height).min(box2.y + box2.height);
    
    if x2 <= x1 || y2 <= y1 {
        return 0.0;
    }
    
    let intersection = ((x2 - x1) * (y2 - y1)) as f32;
    let area1 = (box1.width * box1.height) as f32;
    let area2 = (box2.width * box2.height) as f32;
    let union = area1 + area2 - intersection;
    
    if union <= 0.0 {
        return 0.0;
    }
    
    intersection / union
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

