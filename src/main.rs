use robot_hand::control::{LlmVisionController, LlmVisionControllerConfig};
use robot_hand::{EmgReader, MockSerialController, Position3D, ServoMap};
use robot_hand::error::Result;
use robot_hand::protocol::ServoProtocol;
use robot_hand::vision::OpenCVDetector;
use std::env;

#[cfg(feature = "serial")]
use robot_hand::TextSerialController;

fn print_usage() {
    println!("Robot Hand Control System");
    println!("========================\n");
    println!("Usage:");
    println!("  cargo run --features opencv [OPTIONS]\n");
    println!("Options:");
    println!("  --port <device>      Serial port (e.g., /dev/cu.usbmodem1101)");
    println!("  --camera <id>        Camera ID (default: 0)");
    println!("  --auto               Auto-trigger mode (no EMG needed)");
    println!("  --mock               Use mock controller (for testing)");
    println!("  --help               Show this help\n");
    println!("Environment:");
    println!("  OPENAI_API_KEY       OpenAI API key for LLM planning");
    println!("  YOLO_MODEL_PATH      Path to YOLO model (default: models/yolov8n.onnx)");
    println!("  HAND_MODEL_PATH      Path to hand tracking model\n");
    println!("Examples:");
    println!("  cargo run --features opencv -- --auto --mock");
    println!("  cargo run --features opencv -- --port /dev/cu.usbmodem1101 --auto");
}

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        print_usage();
        return Ok(());
    }

    println!("🤖 Robot Hand Control System");
    println!("============================\n");

    let camera_id = args
        .iter()
        .position(|arg| arg == "--camera")
        .and_then(|i| args.get(i + 1))
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    let serial_port = args
        .iter()
        .position(|arg| arg == "--port")
        .and_then(|i| args.get(i + 1))
        .cloned();

    let use_mock = args.contains(&"--mock".to_string()) || serial_port.is_none();
    let auto_trigger = args.contains(&"--auto".to_string()) || args.contains(&"-a".to_string());

    println!("Configuration:");
    println!("  Camera ID:        {}", camera_id);
    println!("  Mode:             {}", if use_mock { "Mock (Testing)" } else { "Real Hardware" });
    if let Some(ref port) = serial_port {
        println!("  Serial Port:      {}", port);
    }
    println!("  Trigger:          {}", if auto_trigger { "Automatic" } else { "EMG" });
    println!();

    let mut detector = OpenCVDetector::new(camera_id, 0.55)?;

    let yolo_model = env::var("YOLO_MODEL_PATH")
        .unwrap_or_else(|_| "models/yolov8n.onnx".to_string());
    detector.load_yolo_model(&yolo_model)?;
    println!("✅ Object detection ready ({})", yolo_model);

    let hand_model = env::var("HAND_MODEL_PATH")
        .unwrap_or_else(|_| "models/hand_landmarker.onnx".to_string());

    let emg_reader = EmgReader::new("mock", 9600, 600)?;

    let config = LlmVisionControllerConfig {
        servo_map: ServoMap::hardware_default(),
        enable_hand_tracking: true,
        enable_llm_planning: true,
        auto_trigger,
        hand_base_position: Position3D::new(0.0, 0.0, 0.0),
        ..Default::default()
    };

    if use_mock {
        let protocol = MockSerialController::new();
        let mut controller = LlmVisionController::new(detector, emg_reader, protocol, config)?;

        controller.load_hand_tracking_model(&hand_model).ok();
        run_controller(controller, auto_trigger).await
    } else {
        #[cfg(feature = "serial")]
        {
            let port = serial_port.expect("Serial port required when not using --mock");
            let protocol = TextSerialController::new(&port, 115200)?;
            let mut controller = LlmVisionController::new(detector, emg_reader, protocol, config)?;

            controller.load_hand_tracking_model(&hand_model).ok();
            run_controller(controller, auto_trigger).await
        }

        #[cfg(not(feature = "serial"))]
        {
            eprintln!("Error: Serial feature not enabled");
            eprintln!("Build with: cargo build --features opencv,serial");
            std::process::exit(1);
        }
    }
}

async fn run_controller<P: ServoProtocol>(
    mut controller: LlmVisionController<OpenCVDetector, P>,
    auto_trigger: bool,
) -> Result<()> {
    if env::var("OPENAI_API_KEY").is_ok() {
        println!("✅ LLM planning enabled");
    } else {
        println!("⚠️  LLM planning disabled (set OPENAI_API_KEY)");
    }

    println!();
    println!("🎯 System Ready!");
    println!();
    if auto_trigger {
        println!("Waiting for objects in view...");
    } else {
        println!("Waiting for EMG trigger...");
    }
    println!("Press Ctrl+C to stop\n");

    controller.run_async().await?;
    Ok(())
}

