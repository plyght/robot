#[cfg(feature = "opencv")]
fn main() -> Result<()> {
    use opencv::{core, highgui, imgcodecs, imgproc};
    use robot_hand::{create_tracking_with_image, OpenCVDetector};
    use std::env;
    use std::fs;

    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        eprintln!("Usage: {} <object_label> <distance_cm>", args[0]);
        eprintln!("Example: {} bottle 35", args[0]);
        std::process::exit(1);
    }

    let object_label = &args[1];
    let distance_cm: f32 = args[2].parse().expect("Distance must be a number");

    let output_dir = "training_data";
    fs::create_dir_all(output_dir).expect("Failed to create output directory");

    println!("\n========================================");
    println!("  Training Data Collection");
    println!("========================================\n");
    println!("Target object: {}", object_label);
    println!("Distance: {}cm", distance_cm);
    println!(
        "\nPlace the {} at {}cm from camera",
        object_label, distance_cm
    );
    println!("Press SPACE to capture, 'q' to quit\n");

    let mut detector = OpenCVDetector::new(0, 0.5)?;

    println!("Loading YOLO model...");
    detector.load_yolo_model("models/yolov8n.onnx")?;
    println!("Ready!\n");

    let window_name = "Training Data Collection";
    highgui::named_window(window_name, highgui::WINDOW_AUTOSIZE)
        .map_err(|e| robot_hand::HandError::Hardware(format!("Window creation failed: {}", e)))?;

    let mut capture_count = 0;

    loop {
        let frame = detector.get_frame()?;
        let objects = detector.detect_objects()?;

        let mut display_frame = frame.clone();
        let mut target_found = false;

        for obj in &objects {
            let color = if obj
                .label
                .to_lowercase()
                .contains(&object_label.to_lowercase())
            {
                target_found = true;
                core::Scalar::new(0.0, 255.0, 0.0, 0.0)
            } else {
                core::Scalar::new(128.0, 128.0, 128.0, 0.0)
            };

            imgproc::rectangle(
                &mut display_frame,
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

            let label = format!("{} {:.0}%", obj.label, obj.confidence * 100.0);
            imgproc::put_text(
                &mut display_frame,
                &label,
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

        let status = if target_found {
            format!(
                "TARGET DETECTED - Press SPACE to capture ({})",
                capture_count
            )
        } else {
            format!("Searching for {}... ({})", object_label, capture_count)
        };

        imgproc::put_text(
            &mut display_frame,
            &status,
            core::Point::new(10, 30),
            imgproc::FONT_HERSHEY_SIMPLEX,
            0.7,
            core::Scalar::new(0.0, 255.0, 0.0, 0.0),
            2,
            imgproc::LINE_8,
            false,
        )
        .ok();

        highgui::imshow(window_name, &display_frame)
            .map_err(|e| robot_hand::HandError::Hardware(format!("Failed to show frame: {}", e)))?;

        let key = highgui::wait_key(10).map_err(|e| {
            robot_hand::HandError::Hardware(format!("Failed to wait for key: {}", e))
        })?;

        if key == 'q' as i32 || key == 27 {
            break;
        }

        if key == 32 && target_found {
            for obj in &objects {
                if obj
                    .label
                    .to_lowercase()
                    .contains(&object_label.to_lowercase())
                {
                    let tracking = create_tracking_with_image(obj, &frame)?;

                    capture_count += 1;
                    let base_name = format!(
                        "{}_{}cm_{:03}",
                        object_label, distance_cm as i32, capture_count
                    );

                    let full_path = format!("{}/{}_full.jpg", output_dir, base_name);
                    imgcodecs::imwrite(&full_path, &tracking.full_frame, &core::Vector::new())
                        .map_err(|e| {
                            robot_hand::HandError::Hardware(format!(
                                "Failed to save full frame: {}",
                                e
                            ))
                        })?;

                    let crop_path = format!("{}/{}_crop.jpg", output_dir, base_name);
                    imgcodecs::imwrite(&crop_path, &tracking.cropped_object, &core::Vector::new())
                        .map_err(|e| {
                            robot_hand::HandError::Hardware(format!("Failed to save crop: {}", e))
                        })?;

                    let ctx_path = format!("{}/{}_context.jpg", output_dir, base_name);
                    imgcodecs::imwrite(&ctx_path, &tracking.context_crop, &core::Vector::new())
                        .map_err(|e| {
                            robot_hand::HandError::Hardware(format!(
                                "Failed to save context: {}",
                                e
                            ))
                        })?;

                    let json_path = format!("{}/{}_data.json", output_dir, base_name);
                    let json = serde_json::to_string_pretty(&tracking.tracking_data).unwrap();
                    fs::write(&json_path, json).expect("Failed to save JSON");

                    let csv_path = format!("{}/labels.csv", output_dir);
                    let csv_exists = Path::new(&csv_path).exists();
                    let mut file = fs::OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(&csv_path)
                        .expect("Failed to open CSV");

                    use std::io::Write;
                    if !csv_exists {
                        writeln!(file, "object,distance_cm,full_image,crop_image,context_image,bbox_x,bbox_y,bbox_w,bbox_h,confidence").unwrap();
                    }
                    writeln!(
                        file,
                        "{},{},{}_full.jpg,{}_crop.jpg,{}_context.jpg,{},{},{},{},{:.3}",
                        obj.label,
                        distance_cm,
                        base_name,
                        base_name,
                        base_name,
                        obj.bounding_box.x,
                        obj.bounding_box.y,
                        obj.bounding_box.width,
                        obj.bounding_box.height,
                        obj.confidence
                    )
                    .unwrap();

                    println!("✓ Captured {} (Total: {})", base_name, capture_count);
                    break;
                }
            }
        }
    }

    println!("\n========================================");
    println!("Captured {} images", capture_count);
    println!("Saved to: {}/", output_dir);
    println!("========================================\n");

    Ok(())
}

#[cfg(not(feature = "opencv"))]
fn main() {
    eprintln!("This tool requires the 'opencv' feature.");
    eprintln!("Run with: cargo run --bin collect_training_data --features opencv");
    std::process::exit(1);
}
