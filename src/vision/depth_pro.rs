use crate::{DetectedObject, HandError, Result};
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepthProRequest {
    pub command: String,
    pub image_path: Option<String>,
    pub bounding_boxes: Option<Vec<[i32; 4]>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepthProResponse {
    pub status: String,
    pub error: Option<String>,
    pub focal_length_px: Option<f64>,
    pub depth_map_shape: Option<Vec<usize>>,
    pub objects: Option<Vec<ObjectDepth>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectDepth {
    pub bbox: [i32; 4],
    pub depth_meters: f32,
    pub depth_cm: f32,
    pub depth_mean_meters: f32,
    pub depth_min_meters: f32,
}

pub struct DepthProService {
    process: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

impl DepthProService {
    pub fn new(python_path: Option<&str>) -> Result<Self> {
        let python = python_path.unwrap_or("python3");

        let mut process = Command::new(python)
            .arg("depth_service.py")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|e| HandError::Hardware(format!("Failed to start depth service: {}", e)))?;

        let stdin = process
            .stdin
            .take()
            .ok_or_else(|| HandError::Hardware("Failed to open stdin".to_string()))?;

        let stdout = process
            .stdout
            .take()
            .ok_or_else(|| HandError::Hardware("Failed to open stdout".to_string()))?;

        let mut service = DepthProService {
            process,
            stdin,
            stdout: BufReader::new(stdout),
        };

        service.wait_ready()?;

        Ok(service)
    }

    fn wait_ready(&mut self) -> Result<()> {
        std::thread::sleep(std::time::Duration::from_secs(2));
        Ok(())
    }

    pub fn process_image(
        &mut self,
        image_path: &str,
        objects: &[DetectedObject],
    ) -> Result<Vec<ObjectDepth>> {
        self.process_image_with_cleanup(image_path, objects, false)
    }

    pub fn process_image_with_cleanup(
        &mut self,
        image_path: &str,
        objects: &[DetectedObject],
        cleanup: bool,
    ) -> Result<Vec<ObjectDepth>> {
        let bboxes: Vec<[i32; 4]> = objects
            .iter()
            .map(|obj| {
                [
                    obj.bounding_box.x,
                    obj.bounding_box.y,
                    obj.bounding_box.width,
                    obj.bounding_box.height,
                ]
            })
            .collect();

        let request = DepthProRequest {
            command: "process".to_string(),
            image_path: Some(image_path.to_string()),
            bounding_boxes: Some(bboxes),
        };

        let request_json = serde_json::to_string(&request)
            .map_err(|e| HandError::Hardware(format!("Failed to serialize request: {}", e)))?;

        writeln!(self.stdin, "{}", request_json)
            .map_err(|e| HandError::Hardware(format!("Failed to write to depth service: {}", e)))?;

        self.stdin
            .flush()
            .map_err(|e| HandError::Hardware(format!("Failed to flush: {}", e)))?;

        let mut response_line = String::new();
        self.stdout.read_line(&mut response_line).map_err(|e| {
            HandError::Hardware(format!("Failed to read from depth service: {}", e))
        })?;

        let response: DepthProResponse = serde_json::from_str(&response_line)
            .map_err(|e| HandError::Hardware(format!("Failed to parse response: {}", e)))?;

        if cleanup {
            if let Err(e) = std::fs::remove_file(image_path) {
                eprintln!("Warning: Failed to cleanup temp file {}: {}", image_path, e);
            }
        }

        if response.status != "success" {
            return Err(HandError::Hardware(format!(
                "Depth service error: {:?}",
                response.error
            )));
        }

        Ok(response.objects.unwrap_or_default())
    }

    pub fn ping(&mut self) -> Result<()> {
        let request = DepthProRequest {
            command: "ping".to_string(),
            image_path: None,
            bounding_boxes: None,
        };

        let request_json = serde_json::to_string(&request)
            .map_err(|e| HandError::Hardware(format!("Failed to serialize ping: {}", e)))?;

        writeln!(self.stdin, "{}", request_json)
            .map_err(|e| HandError::Hardware(format!("Failed to write ping: {}", e)))?;

        self.stdin
            .flush()
            .map_err(|e| HandError::Hardware(format!("Failed to flush ping: {}", e)))?;

        let mut response_line = String::new();
        self.stdout
            .read_line(&mut response_line)
            .map_err(|e| HandError::Hardware(format!("Failed to read ping response: {}", e)))?;

        let response: DepthProResponse = serde_json::from_str(&response_line)
            .map_err(|e| HandError::Hardware(format!("Failed to parse ping response: {}", e)))?;

        if response.status == "ok" {
            Ok(())
        } else {
            Err(HandError::Hardware("Ping failed".to_string()))
        }
    }
}

impl Drop for DepthProService {
    fn drop(&mut self) {
        let _ = writeln!(self.stdin, r#"{{"command":"exit"}}"#);
        let _ = self.process.kill();
    }
}
