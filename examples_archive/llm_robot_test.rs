use robot_hand::control::{LlmVisionController, LlmVisionControllerConfig};
use robot_hand::{EmgReader, MockSerialController};
use robot_hand::error::Result;
use robot_hand::vision::OpenCVDetector;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== LLM-Driven Robot Hand Control Test ===\n");

    if env::var("OPENAI_API_KEY").is_err() {
        eprintln!("Warning: OPENAI_API_KEY not set. LLM planning will be disabled.");
        eprintln!("Set it with: export OPENAI_API_KEY=your_key_here\n");
    }

    let camera_id = env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    println!("Initializing camera (ID: {})...", camera_id);
    let mut detector = OpenCVDetector::new(camera_id, 0.55)?;

    let yolo_model_path = env::var("YOLO_MODEL_PATH")
        .unwrap_or_else(|_| "models/yolov8n.onnx".to_string());

    println!("Loading YOLO model from: {}", yolo_model_path);
    detector.load_yolo_model(&yolo_model_path)?;

    let emg_reader = EmgReader::new("mock", 9600, 600)?;

    let protocol = MockSerialController::new();

    let hand_tracking_model_path = env::var("HAND_MODEL_PATH")
        .unwrap_or_else(|_| "models/hand_landmarker.onnx".to_string());

    let config = LlmVisionControllerConfig {
        enable_hand_tracking: true,
        enable_llm_planning: true,
        ..Default::default()
    };

    let mut controller = LlmVisionController::new(
        detector,
        emg_reader,
        protocol,
        config,
    )?;

    let hand_tracking_status = match controller.load_hand_tracking_model(&hand_tracking_model_path) {
        Ok(_) => {
            println!("Hand tracking model loaded from: {}", hand_tracking_model_path);
            "Enabled"
        }
        Err(e) => {
            eprintln!("Warning: Failed to load hand tracking model: {}", e);
            eprintln!("Hand tracking will be disabled. LLM will estimate hand position.");
            "Disabled (model not found)"
        }
    };

    println!("\n=== System Ready ===");
    println!("Configuration:");
    println!("  - Object Detection: YOLO (OpenCV)");
    println!("  - Depth Estimation: Available (depth_service.py)");
    println!("  - Hand Tracking: {}", hand_tracking_status);
    println!("  - LLM Planning: {}",
        if env::var("OPENAI_API_KEY").is_ok() { "Enabled (gpt-5-nano-2025-08-07)" } else { "Disabled" });
    println!("  - EMG Trigger: Mock (threshold: 600)");
    println!("\nWaiting for trigger...");
    println!("Press Ctrl+C to stop\n");

    tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
        println!("\n[Simulating EMG trigger in 2 seconds...]\n");
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    });

    match controller.run_async().await {
        Ok(_) => println!("Controller stopped normally"),
        Err(e) => eprintln!("Controller error: {}", e),
    }

    Ok(())
}
