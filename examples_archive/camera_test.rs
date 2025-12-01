#[cfg(feature = "opencv")]
fn main() -> robot_hand::Result<()> {
    use opencv::{core, highgui, imgproc};
    use robot_hand::OpenCVDetector;
    use std::env;
    use std::time::Instant;

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
    println!("  Camera Object Detection Test");
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

    let window_name = "Camera Test - Object Detection";
    highgui::named_window(window_name, highgui::WINDOW_AUTOSIZE)
        .map_err(|e| robot_hand::HandError::Hardware(format!("Window creation failed: {}", e)))?;

    let mut frame_count = 0;
    let mut total_objects = 0;
    let start_time = Instant::now();
    let mut last_fps_update = Instant::now();
    let mut fps = 0.0;

    loop {
        let loop_start = Instant::now();

        let mut frame = detector.get_frame()?;
        let objects = detector.detect_objects()?;

        frame_count += 1;
        total_objects += objects.len();

        if last_fps_update.elapsed().as_secs() >= 1 {
            fps = frame_count as f64 / start_time.elapsed().as_secs_f64();
            last_fps_update = Instant::now();
        }

        for obj in &objects {
            let bbox = &obj.bounding_box;
            let rect = core::Rect::new(bbox.x, bbox.y, bbox.width, bbox.height);
            let color = core::Scalar::new(0.0, 255.0, 0.0, 0.0);

            imgproc::rectangle(&mut frame, rect, color, 2, imgproc::LINE_8, 0).map_err(|e| {
                robot_hand::HandError::Hardware(format!("Rectangle draw failed: {}", e))
            })?;

            let label = format!("{} ({:.1}%)", obj.label, obj.confidence * 100.0);
            let label_pos = core::Point::new(bbox.x, bbox.y.saturating_sub(10).max(20));

            imgproc::put_text(
                &mut frame,
                &label,
                label_pos,
                imgproc::FONT_HERSHEY_SIMPLEX,
                0.5,
                color,
                1,
                imgproc::LINE_8,
                false,
            )
            .map_err(|e| robot_hand::HandError::Hardware(format!("Text draw failed: {}", e)))?;
        }

        let stats = format!(
            "FPS: {:.1} | Objects: {} | Total: {}",
            fps,
            objects.len(),
            total_objects
        );
        imgproc::put_text(
            &mut frame,
            &stats,
            core::Point::new(10, 30),
            imgproc::FONT_HERSHEY_SIMPLEX,
            0.7,
            core::Scalar::new(0.0, 255.0, 255.0, 0.0),
            2,
            imgproc::LINE_8,
            false,
        )
        .map_err(|e| robot_hand::HandError::Hardware(format!("Stats text failed: {}", e)))?;

        highgui::imshow(window_name, &frame)
            .map_err(|e| robot_hand::HandError::Hardware(format!("Image show failed: {}", e)))?;

        let key = highgui::wait_key(1)
            .map_err(|e| robot_hand::HandError::Hardware(format!("Wait key failed: {}", e)))?;

        if key == 'q' as i32 || key == 27 {
            break;
        }

        let elapsed = loop_start.elapsed();
        if elapsed.as_millis() < 33 {
            std::thread::sleep(std::time::Duration::from_millis(
                33 - elapsed.as_millis() as u64,
            ));
        }
    }

    let total_time = start_time.elapsed().as_secs_f64();
    let avg_fps = frame_count as f64 / total_time;
    let avg_objects_per_frame = if frame_count > 0 {
        total_objects as f64 / frame_count as f64
    } else {
        0.0
    };

    println!("\n========================================");
    println!("  Test Complete!");
    println!("========================================");
    println!("Total frames: {}", frame_count);
    println!("Total time: {:.2}s", total_time);
    println!("Average FPS: {:.2}", avg_fps);
    println!("Total objects detected: {}", total_objects);
    println!("Average objects per frame: {:.2}", avg_objects_per_frame);
    println!("========================================\n");

    Ok(())
}

#[cfg(not(feature = "opencv"))]
fn main() -> robot_hand::Result<()> {
    eprintln!("This program requires the 'opencv' feature");
    eprintln!("Install OpenCV first: brew install opencv");
    eprintln!("Then run with: cargo run --bin camera_test --features opencv");
    Ok(())
}
