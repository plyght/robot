use robot_hand::{
    create_default_finger_servo_map, BoundingBox, DetectedObject, EmgReader, MockObjectDetector,
    MockSerialController, Result, VisionController, VisionControllerConfig,
};
use std::sync::{Arc, Mutex};
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
    let (emg_reader, protocol) = {
        let args: Vec<String> = std::env::args().collect();
        let servo_port = if args.len() > 1 {
            println!("Using servo port from argument: {}", args[1]);
            args[1].clone()
        } else {
            print!("Enter servo serial port (e.g., /dev/cu.usbmodem1101): ");
            io::stdout().flush()?;
            let mut servo_port = String::new();
            io::stdin().read_line(&mut servo_port)?;
            servo_port.trim().to_string()
        };
        
        let emg_port = if args.len() > 2 && args[2] != "mock" {
            println!("Using EMG port from argument: {}", args[2]);
            Some(args[2].clone())
        } else {
            println!("No EMG port provided, using mock EMG reader");
            None
        };
        
        let emg_reader = if let Some(port) = emg_port {
            EmgReader::new(&port, 9600, 600)?
        } else {
            EmgReader::new("mock", 9600, 600)?
        };
        
        let protocol = robot_hand::TextSerialController::new(&servo_port, 115200)?;
        (emg_reader, protocol)
    };

    #[cfg(not(feature = "serial"))]
    let emg_reader = EmgReader::new("mock", 9600, 600)?;

    #[cfg(not(feature = "serial"))]
    let protocol = MockSerialController::new();

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
        let args: Vec<String> = std::env::args().collect();
        let manual_mode = args.len() > 2 && args[2] == "mock";
        
        println!("\n==============================================");
        println!("  LIVE MODE (Hardware Connected)");
        println!("==============================================\n");
        
        if manual_mode {
            println!("Manual Control Mode Active");
            println!("Commands:");
            println!("  't' or Enter - Trigger pickup sequence");
            println!("  'q' - Quit");
            println!("Press Ctrl+C to stop.\n");
            
            let controller = Arc::new(Mutex::new(controller));
            let controller_clone = Arc::clone(&controller);
            
            let controller_handle = thread::spawn(move || {
                let mut ctrl = controller_clone.lock().unwrap();
                ctrl.running = true;
                println!("Vision + EMG Control System Started");
                println!("Threshold: {} | Waiting for trigger...\n", 600);
                
                while ctrl.running {
                    if let Err(e) = ctrl.run_step() {
                        eprintln!("Controller error: {}", e);
                        break;
                    }
                    if !ctrl.running {
                        break;
                    }
                    thread::sleep(Duration::from_millis(10));
                }
            });
            
            loop {
                let mut input = String::new();
                if std::io::stdin().read_line(&mut input).is_ok() {
                    let cmd = input.trim();
                    match cmd {
                        "t" | "" => {
                            let mut ctrl = controller.lock().unwrap();
                            if let Err(e) = ctrl.inject_emg_trigger(650) {
                                eprintln!("Error triggering: {}", e);
                            } else {
                                println!("\nTrigger sent! Processing...");
                            }
                        }
                        "q" => {
                            println!("\nQuitting...");
                            let mut ctrl = controller.lock().unwrap();
                            ctrl.stop();
                            break;
                        }
                        _ => {
                            println!("Unknown command. Use 't' to trigger, 'q' to quit.");
                        }
                    }
                }
            }
            
            controller_handle.join().ok();
        } else {
            println!("System ready. Waiting for EMG trigger...");
            println!("Press Ctrl+C to stop.\n");
            controller.run()?;
        }
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

