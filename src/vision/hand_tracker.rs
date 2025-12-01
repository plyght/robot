use crate::error::Result;
use crate::control::HandPose;

#[cfg(feature = "opencv")]
use opencv::core::Mat;

#[cfg(feature = "opencv")]
use ort::session::Session;

#[cfg(feature = "opencv")]
pub struct HandTracker {
    session: Option<Session>,
    input_size: (i32, i32),
    min_detection_confidence: f32,
    frame_width: i32,
    frame_height: i32,
    camera_fov_horizontal: f32,
    camera_fov_vertical: f32,
}

#[cfg(feature = "opencv")]
impl HandTracker {
    pub fn new(frame_width: i32, frame_height: i32) -> Self {
        Self {
            session: None,
            input_size: (224, 224),
            min_detection_confidence: 0.5,
            frame_width,
            frame_height,
            camera_fov_horizontal: 60.0,
            camera_fov_vertical: 45.0,
        }
    }

    pub fn load_model(&mut self, model_path: &str) -> Result<()> {
        use crate::error::HandError;

        let session = Session::builder()
            .map_err(|e| HandError::Hardware(format!("Failed to create ONNX session: {}", e)))?
            .commit_from_file(model_path)
            .map_err(|e| HandError::Hardware(format!("Failed to load hand tracking ONNX model: {}", e)))?;

        self.session = Some(session);
        println!("Loaded hand tracking ONNX model from: {}", model_path);
        Ok(())
    }

    pub fn detect_hands(&mut self, frame: &Mat, depth_cm: Option<f32>) -> Result<Vec<HandPose>> {
        use crate::error::HandError;

        let input_array = self.preprocess_frame(frame)?;

        let input_tensor = ort::value::Value::from_array(input_array)
            .map_err(|e| HandError::Hardware(format!("Failed to create input tensor: {}", e)))?;

        let session = self
            .session
            .as_mut()
            .ok_or_else(|| HandError::Hardware("Hand tracking model not loaded".to_string()))?;

        let outputs = session
            .run(ort::inputs![input_tensor])
            .map_err(|e| HandError::Hardware(format!("Hand tracking inference failed: {}", e)))?;

        let (_, landmarks_output) = outputs
            .iter()
            .next()
            .ok_or_else(|| HandError::Hardware("No output from hand tracking model".to_string()))?;

        let (_, data) = landmarks_output
            .try_extract_tensor::<f32>()
            .map_err(|e| HandError::Hardware(format!("Failed to extract landmarks: {}", e)))?;

        let hands = Self::parse_landmarks_from_slice(
            data,
            depth_cm,
            self.frame_width,
            self.frame_height,
            self.camera_fov_horizontal,
            self.camera_fov_vertical,
        )?;

        Ok(hands)
    }

    fn preprocess_frame(&self, frame: &Mat) -> Result<ndarray::Array4<f32>> {
        use crate::error::HandError;
        use opencv::{core, imgproc, prelude::*};

        let mut resized = Mat::default();
        let target_size = core::Size::new(self.input_size.0, self.input_size.1);
        imgproc::resize(
            frame,
            &mut resized,
            target_size,
            0.0,
            0.0,
            imgproc::INTER_LINEAR,
        )
        .map_err(|e| HandError::Hardware(format!("Failed to resize frame: {}", e)))?;

        let mut rgb = Mat::default();
        imgproc::cvt_color(&resized, &mut rgb, imgproc::COLOR_BGR2RGB, 0, core::AlgorithmHint::ALGO_HINT_DEFAULT)
            .map_err(|e| HandError::Hardware(format!("Failed to convert color: {}", e)))?;

        let rows = rgb.rows();
        let cols = rgb.cols();
        let channels = rgb.channels();

        let data = rgb.data_bytes()
            .map_err(|e| HandError::Hardware(format!("Failed to get frame data: {}", e)))?;

        let mut input = ndarray::Array4::<f32>::zeros((1, 3, self.input_size.1 as usize, self.input_size.0 as usize));

        for y in 0..rows {
            for x in 0..cols {
                let idx = (y * cols * channels + x * channels) as usize;
                if idx + 2 < data.len() {
                    let r = data[idx] as f32 / 255.0;
                    let g = data[idx + 1] as f32 / 255.0;
                    let b = data[idx + 2] as f32 / 255.0;

                    input[[0, 0, y as usize, x as usize]] = r;
                    input[[0, 1, y as usize, x as usize]] = g;
                    input[[0, 2, y as usize, x as usize]] = b;
                }
            }
        }

        Ok(input)
    }

    fn parse_landmarks_from_slice(
        data: &[f32],
        depth_cm: Option<f32>,
        frame_width: i32,
        frame_height: i32,
        camera_fov_horizontal: f32,
        camera_fov_vertical: f32,
    ) -> Result<Vec<HandPose>> {
        if data.len() < 21 * 3 {
            return Ok(Vec::new());
        }

        let num_landmarks = data.len();

        let default_depth = depth_cm.unwrap_or(30.0);

        let wrist_x = data[0];
        let wrist_y = data[1];
        let wrist_z = data[2];

        let wrist_3d = Self::compute_3d_position_static(
            wrist_x,
            wrist_y,
            wrist_z,
            default_depth,
            frame_width,
            frame_height,
            camera_fov_horizontal,
            camera_fov_vertical,
        );

        let palm_indices = [0, 5, 9, 13, 17];
        let mut palm_x = 0.0;
        let mut palm_y = 0.0;
        let mut palm_z = 0.0;

        for &idx in &palm_indices {
            if idx * 3 + 2 < num_landmarks {
                palm_x += data[idx * 3];
                palm_y += data[idx * 3 + 1];
                palm_z += data[idx * 3 + 2];
            }
        }

        palm_x /= palm_indices.len() as f32;
        palm_y /= palm_indices.len() as f32;
        palm_z /= palm_indices.len() as f32;

        let palm_center = Self::compute_3d_position_static(
            palm_x,
            palm_y,
            palm_z,
            default_depth,
            frame_width,
            frame_height,
            camera_fov_horizontal,
            camera_fov_vertical,
        );

        let tip_indices = [4, 8, 12, 16, 20];
        let mut finger_tips = Vec::new();

        for &idx in &tip_indices {
            if idx * 3 + 2 < num_landmarks {
                let tip_x = data[idx * 3];
                let tip_y = data[idx * 3 + 1];
                let tip_z = data[idx * 3 + 2];

                let tip_3d = Self::compute_3d_position_static(
                    tip_x,
                    tip_y,
                    tip_z,
                    default_depth,
                    frame_width,
                    frame_height,
                    camera_fov_horizontal,
                    camera_fov_vertical,
                );
                finger_tips.push(tip_3d);
            }
        }

        let is_open = Self::is_hand_open_static(data, num_landmarks);

        let confidence = 0.8;

        let hand_pose = HandPose {
            palm_center,
            wrist_position: wrist_3d,
            finger_tips,
            is_open,
            confidence,
        };

        Ok(vec![hand_pose])
    }

    fn compute_3d_position_static(
        x_norm: f32,
        y_norm: f32,
        z_rel: f32,
        depth_cm: f32,
        frame_width: i32,
        frame_height: i32,
        camera_fov_horizontal: f32,
        camera_fov_vertical: f32,
    ) -> (f32, f32, f32) {
        let pixel_x = x_norm * frame_width as f32;
        let pixel_y = y_norm * frame_height as f32;

        let z_cm = depth_cm + (z_rel * 100.0);

        let center_x = frame_width as f32 / 2.0;
        let center_y = frame_height as f32 / 2.0;

        let offset_x_pixels = pixel_x - center_x;
        let offset_y_pixels = pixel_y - center_y;

        let angle_per_pixel_h = camera_fov_horizontal / frame_width as f32;
        let angle_per_pixel_v = camera_fov_vertical / frame_height as f32;

        let angle_x = offset_x_pixels * angle_per_pixel_h;
        let angle_y = -offset_y_pixels * angle_per_pixel_v;

        let x_cm = z_cm * angle_x.to_radians().tan();
        let y_cm = z_cm * angle_y.to_radians().tan();

        (x_cm, y_cm, z_cm)
    }

    fn is_hand_open_static(data: &[f32], num_landmarks: usize) -> bool {
        let tip_indices = [4, 8, 12, 16, 20];
        let base_indices = [2, 5, 9, 13, 17];

        let mut open_count = 0;

        for (&tip_idx, &base_idx) in tip_indices.iter().zip(base_indices.iter()) {
            if tip_idx * 3 + 2 < num_landmarks && base_idx * 3 + 2 < num_landmarks {
                let tip_x = data[tip_idx * 3];
                let tip_y = data[tip_idx * 3 + 1];
                let tip_z = data[tip_idx * 3 + 2];

                let base_x = data[base_idx * 3];
                let base_y = data[base_idx * 3 + 1];
                let base_z = data[base_idx * 3 + 2];

                let distance = ((tip_x - base_x).powi(2)
                    + (tip_y - base_y).powi(2)
                    + (tip_z - base_z).powi(2))
                .sqrt();

                if distance > 0.15 {
                    open_count += 1;
                }
            }
        }

        open_count >= 3
    }

    pub fn set_frame_size(&mut self, width: i32, height: i32) {
        self.frame_width = width;
        self.frame_height = height;
    }
}
