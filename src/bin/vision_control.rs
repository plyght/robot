use robot_hand::{
    create_default_finger_servo_map, BoundingBox, DetectedObject, EmgReader, MockObjectDetector,
    MockSerialController, Result, VisionController, VisionControllerConfig,
};
use std::thread;
use std::time::Duration;

#[cfg(feature = "serial")]
use std::io::{self, Write};

fn main() -> Result<()> {
    println!("==============================================");
    println!("  Vision + EMG Robotic Hand Control System");
    println!("==============================================\n");

    println!("Configuration:");
    println!("  - EMG Threshold: 600 (0-1023 range)");
    println!("  - Serial Protocol: servo<id> <finger> <angle>");
    println!("  - Vision: Mock detector (replace with OpenCV)");
    println!("  - Control Mode: Autonomous pickup on EMG trigger\n");

    let detector = create_mock_detector();

    #[cfg(feature = "serial")]
    let emg_reader = {
        print!("Enter EMG serial port (e.g., /dev/ttyUSB0 or COM3): ");
        io::stdout().flush()?;
        let mut emg_port = String::new();
        io::stdin().read_line(&mut emg_port)?;
        let emg_port = emg_port.trim();
        EmgReader::new(emg_port, 9600, 600)?
    };

    #[cfg(not(feature = "serial"))]
    let emg_reader = EmgReader::new("mock", 9600, 600)?;

    #[cfg(feature = "serial")]
    let protocol = {
        print!("Enter servo serial port (e.g., /dev/ttyUSB1 or COM4): ");
        io::stdout().flush()?;
        let mut servo_port = String::new();
        io::stdin().read_line(&mut servo_port)?;
        let servo_port = servo_port.trim();
        robot_hand::TextSerialController::new(servo_port, 115200)?
    };

    #[cfg(not(feature = "serial"))]
    let protocol = MockSerialController::new();

    let config = VisionControllerConfig {
        camera_poll_interval: Duration::from_millis(100),
        emg_poll_interval: Duration::from_millis(10),
        finger_to_servo_map: create_default_finger_servo_map(),
    };

    let mut controller = VisionController::new(detector, emg_reader, protocol, config);

    #[cfg(not(feature = "serial"))]
    {
        println!("\n==============================================");
        println!("  DEMO MODE (Mock Hardware)");
        println!("==============================================\n");
        println!("Starting demo in 2 seconds...");
        thread::sleep(Duration::from_secs(2));

        let controller_handle = thread::spawn(move || {
            if let Err(e) = controller.run() {
                eprintln!("Controller error: {}", e);
            }
        });

        thread::sleep(Duration::from_secs(1));

        println!("\n[DEMO] Simulating EMG trigger (value=650)...\n");
        thread::sleep(Duration::from_secs(5));

        controller_handle.join().ok();
    }

    #[cfg(feature = "serial")]
    {
        println!("\n==============================================");
        println!("  LIVE MODE (Hardware Connected)");
        println!("==============================================\n");
        println!("System ready. Waiting for EMG trigger...");
        println!("Press Ctrl+C to stop.\n");

        controller.run()?;
    }

    Ok(())
}

fn create_mock_detector() -> MockObjectDetector {
    let mut detector = MockObjectDetector::new(640, 480);

    detector.add_mock_object(DetectedObject {
        label: "cup".to_string(),
        confidence: 0.92,
        bounding_box: BoundingBox {
            x: 250,
            y: 180,
            width: 120,
            height: 150,
        },
        distance: 0.35,
    });

    detector.add_mock_object(DetectedObject {
        label: "phone".to_string(),
        confidence: 0.87,
        bounding_box: BoundingBox {
            x: 400,
            y: 220,
            width: 80,
            height: 140,
        },
        distance: 0.42,
    });

    detector.add_mock_object(DetectedObject {
        label: "bottle".to_string(),
        confidence: 0.78,
        bounding_box: BoundingBox {
            x: 150,
            y: 160,
            width: 90,
            height: 200,
        },
        distance: 0.50,
    });

    detector
}

