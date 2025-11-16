use robot_hand::{
    create_default_finger_servo_map, BoundingBox, DetectedObject, EmgReader, GripPattern,
    MockObjectDetector, MockSerialController, PickupSequence, Result, VisionController,
    VisionControllerConfig,
};
use std::thread;
use std::time::Duration;

fn main() -> Result<()> {
    println!("\n========================================");
    println!("  Vision + EMG Integration Demo");
    println!("========================================\n");

    demo_grip_patterns()?;
    println!();

    demo_pickup_sequence()?;
    println!();

    demo_vision_controller()?;

    Ok(())
}

fn demo_grip_patterns() -> Result<()> {
    println!("DEMO 1: Grip Patterns");
    println!("---------------------");

    let patterns = vec![
        ("cup", GripPattern::power_grasp()),
        ("phone", GripPattern::precision_grip()),
        ("pen", GripPattern::pinch_grip()),
        ("card", GripPattern::lateral_grip()),
    ];

    for (object_type, pattern) in patterns {
        println!("  {} → {:?}", object_type, pattern.pattern_type);
        println!("    Approach distance: {:.1}cm", pattern.approach_distance);
        if let Some(wrist) = pattern.wrist_orientation {
            println!("    Wrist: [{:.1}, {:.1}, {:.1}]", wrist[0], wrist[1], wrist[2]);
        }
    }

    Ok(())
}

fn demo_pickup_sequence() -> Result<()> {
    println!("DEMO 2: Pickup Sequence");
    println!("-----------------------");

    let grip = GripPattern::power_grasp();
    let mut sequence = PickupSequence::new(grip);
    let mut protocol = MockSerialController::new();
    let servo_map = create_default_finger_servo_map();

    println!("  Executing pickup sequence for a cup...\n");

    while !sequence.is_complete() {
        println!("  Step: {:?}", sequence.current_step());
        sequence.execute_step_by_step(&mut protocol, &servo_map)?;
        thread::sleep(Duration::from_millis(300));
    }

    Ok(())
}

fn demo_vision_controller() -> Result<()> {
    println!("DEMO 3: Vision Controller Integration");
    println!("-------------------------------------");

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

    let emg_reader = EmgReader::new("mock", 9600, 600)?;
    let protocol = MockSerialController::new();
    let config = VisionControllerConfig::default();

    let mut controller = VisionController::new(detector, emg_reader, protocol, config);

    println!("  Simulating EMG trigger (value=650)...\n");
    controller.inject_emg_trigger(650)?;

    println!("  Running control loop for one cycle...\n");
    
    let handle = thread::spawn(move || {
        thread::sleep(Duration::from_secs(8));
        controller.stop();
    });

    handle.join().ok();

    println!("\n========================================");
    println!("  Demo Complete!");
    println!("========================================\n");

    Ok(())
}

