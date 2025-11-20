#[cfg(feature = "opencv")]
fn main() -> Result<()> {
    use opencv::{core, highgui, imgproc};
    use robot_hand::{create_tracking_data, OpenCVDetector};
    use std::env;

    let args: Vec<String> = env::args().collect();

    let camera_id = if args.len() > 1 {
        args[1].parse::<i32>().unwrap_or(0)
    } else {
        0
    };

    let confidence_threshold = if args.len() > 2 {
        args[2].parse::<f32>().unwrap_or(0.5)
    } else {
        0.5
    };

    println!("\n========================================");
    println!("  Object Tracking Output Test");
    println!("========================================\n");
    println!("Camera ID: {}", camera_id);
    println!("Confidence Threshold: {:.2}", confidence_threshold);
    println!("\nInitializing camera...");

    let mut detector = OpenCVDetector::new(camera_id, confidence_threshold)?;
    let (width, height) = detector.get_frame_size();

    println!("Camera opened successfully!");
    println!("Resolution: {}x{}", width, height);

    println!("\nLoading YOLO model...");
    let model_path = "models/yolov8n.onnx";
    match detector.load_yolo_model(model_path) {
        Ok(_) => println!("YOLO model loaded successfully!"),
        Err(e) => {
            eprintln!("Warning: Could not load YOLO model: {}", e);
            eprintln!("Running without object detection...");
        }
    }

    println!("\nPress 'q' or ESC to quit\n");
    println!("--- Tracking Output (for AI model input) ---\n");

    let window_name = "Tracking Output Test";
    highgui::named_window(window_name, highgui::WINDOW_AUTOSIZE)
        .map_err(|e| robot_hand::HandError::Hardware(format!("Window creation failed: {}", e)))?;

    let mut frame_count = 0;
    let start_time = Instant::now();
    let mut last_print = Instant::now();

    loop {
        let loop_start = Instant::now();
        frame_count += 1;

        let mut frame = detector.get_frame()?;
        let objects = detector.detect_objects()?;

        if !objects.is_empty() && last_print.elapsed().as_millis() > 500 {
            println!(
                "\n=== Frame {} - {} objects detected ===",
                frame_count,
                objects.len()
            );

            for (idx, obj) in objects.iter().enumerate() {
                let tracking = create_tracking_data(obj, width, height);

                println!("\nObject {}: {}", idx + 1, obj.label);
                println!("  Confidence: {:.1}%", obj.confidence * 100.0);
                println!("  Position:");
                println!(
                    "    - Center: ({:.3}, {:.3}) normalized",
                    tracking.center_x_norm, tracking.center_y_norm
                );
                println!(
                    "    - Horizontal angle: {:.1}°",
                    tracking.horizontal_angle_deg
                );
                println!("    - Vertical angle: {:.1}°", tracking.vertical_angle_deg);
                println!("  Size:");
                println!(
                    "    - Dimensions: {}x{} px",
                    obj.bounding_box.width, obj.bounding_box.height
                );
                println!("    - Area ratio: {:.3}%", tracking.area_ratio * 100.0);
                println!("  Distance:");
                println!(
                    "    - Estimated depth: {:.0} cm",
                    tracking.estimated_depth_cm
                );
                println!(
                    "  Raw bounding box: x={}, y={}, w={}, h={}",
                    obj.bounding_box.x,
                    obj.bounding_box.y,
                    obj.bounding_box.width,
                    obj.bounding_box.height
                );

                if let Ok(json) = serde_json::to_string_pretty(&tracking) {
                    println!("\n  JSON output:\n{}", json);
                }
            }

            last_print = Instant::now();
        }

        for obj in &objects {
            let color = core::Scalar::new(0.0, 255.0, 0.0, 0.0);

            imgproc::rectangle(
                &mut frame,
                core::Rect::new(
                    obj.bounding_box.x,
                    obj.bounding_box.y,
                    obj.bounding_box.width,
                    obj.bounding_box.height,
                ),
                color,
                2,
                imgproc::LINE_8,
                0,
            )
            .ok();

            let tracking = create_tracking_data(obj, width, height);
            let label_text = format!(
                "{} {:.0}cm {:.0}%",
                obj.label,
                tracking.estimated_depth_cm,
                obj.confidence * 100.0
            );

            imgproc::put_text(
                &mut frame,
                &label_text,
                core::Point::new(obj.bounding_box.x, obj.bounding_box.y - 5),
                imgproc::FONT_HERSHEY_SIMPLEX,
                0.5,
                color,
                1,
                imgproc::LINE_8,
                false,
            )
            .ok();
        }

        let elapsed = start_time.elapsed().as_secs_f32();
        let fps = frame_count as f32 / elapsed;
        let fps_text = format!("FPS: {:.1} | Objects: {}", fps, objects.len());

        imgproc::put_text(
            &mut frame,
            &fps_text,
            core::Point::new(10, 30),
            imgproc::FONT_HERSHEY_SIMPLEX,
            0.7,
            core::Scalar::new(0.0, 255.0, 0.0, 0.0),
            2,
            imgproc::LINE_8,
            false,
        )
        .ok();

        highgui::imshow(window_name, &frame)
            .map_err(|e| robot_hand::HandError::Hardware(format!("Failed to show frame: {}", e)))?;

        let key = highgui::wait_key(1).map_err(|e| {
            robot_hand::HandError::Hardware(format!("Failed to wait for key: {}", e))
        })?;

        if key == 'q' as i32 || key == 27 {
            break;
        }

        let loop_time = loop_start.elapsed().as_millis();
        if loop_time < 16 {
            std::thread::sleep(std::time::Duration::from_millis((16 - loop_time) as u64));
        }
    }

    let elapsed = start_time.elapsed().as_secs_f32();
    let avg_fps = frame_count as f32 / elapsed;

    println!("\n========================================");
    println!("Statistics:");
    println!("  Total frames: {}", frame_count);
    println!("  Duration: {:.1}s", elapsed);
    println!("  Average FPS: {:.1}", avg_fps);
    println!("========================================\n");

    Ok(())
}

#[cfg(not(feature = "opencv"))]
fn main() {
    eprintln!("This example requires the 'opencv' feature to be enabled.");
    eprintln!("Run with: cargo run --bin tracking_output_test --features opencv");
    std::process::exit(1);
}
