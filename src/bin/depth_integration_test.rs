#[cfg(feature = "opencv")]
fn main() -> Result<()> {
    use opencv::{core, highgui, imgcodecs, imgproc};
    use robot_hand::{DepthProService, OpenCVDetector};

    let args: Vec<String> = env::args().collect();

    let mut camera_id = 0;
    let mut stream_mode = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--stream" | "-s" => {
                stream_mode = true;
            }
            _ => {
                if let Ok(id) = args[i].parse::<i32>() {
                    camera_id = id;
                }
            }
        }
        i += 1;
    }

    println!("\n========================================");
    println!("  Depth Pro Integration Test");
    println!("========================================\n");
    println!("Camera ID: {}", camera_id);
    if stream_mode {
        println!("Mode: CONTINUOUS STREAM (robot mode)");
        println!("      Checking depth as fast as possible");
    } else {
        println!("Mode: MANUAL (press SPACE)");
    }

    println!("\nInitializing camera...");
    let mut detector = OpenCVDetector::new(camera_id, 0.55)?;
    let (width, height) = detector.get_frame_size();
    println!("Camera: {}x{}", width, height);

    println!("\nLoading YOLO model...");
    detector.load_yolo_model("models/yolov8n.onnx")?;
    println!("✓ YOLO loaded");

    println!("\nStarting Depth Pro service...");
    println!("(This may take 10-20 seconds to load the model...)");
    let python_path = "venv_depth_pro/bin/python3";
    let mut depth_service = DepthProService::new(Some(python_path))?;
    println!("✓ Depth Pro ready!");

    if stream_mode {
        println!("\n⚡ STREAM MODE: Continuous depth updates");
        println!("   Updates as fast as possible (~2 Hz)");
        println!("   Perfect for robot control!");
        println!("\nPress 'q' to quit\n");
    } else {
        println!("\nPress 'q' to quit, SPACE to capture depth\n");
    }

    let window_name = "Depth Pro Integration";
    highgui::named_window(window_name, highgui::WINDOW_AUTOSIZE)
        .map_err(|e| robot_hand::HandError::Hardware(format!("Window creation failed: {}", e)))?;

    let temp_dir = std::path::Path::new("temp");
    std::fs::create_dir_all(temp_dir).ok();

    let mut frame_count = 0;
    let start_time = Instant::now();

    let cached_depths: Arc<Mutex<Vec<robot_hand::ObjectDepth>>> = Arc::new(Mutex::new(Vec::new()));
    let cached_objects: Arc<Mutex<Vec<robot_hand::DetectedObject>>> =
        Arc::new(Mutex::new(Vec::new()));
    let depth_computing: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
    let last_depth_time: Arc<Mutex<Option<Instant>>> = Arc::new(Mutex::new(None));

    let (tx, rx): (
        Sender<(PathBuf, Vec<robot_hand::DetectedObject>)>,
        Receiver<(PathBuf, Vec<robot_hand::DetectedObject>)>,
    ) = channel();

    if stream_mode {
        let depths_arc = Arc::clone(&cached_depths);
        let objects_arc = Arc::clone(&cached_objects);
        let computing_arc = Arc::clone(&depth_computing);
        let time_arc = Arc::clone(&last_depth_time);

        thread::spawn(move || {
            println!("[Depth Worker] Starting continuous depth stream...");

            let mut depth_service =
                match robot_hand::DepthProService::new(Some("venv_depth_pro/bin/python3")) {
                    Ok(s) => {
                        println!("[Depth Worker] Service ready!");
                        s
                    }
                    Err(e) => {
                        eprintln!("[Depth Worker] Failed to start: {}", e);
                        return;
                    }
                };

            loop {
                if let Ok((image_path, objects)) = rx.recv() {
                    if objects.is_empty() {
                        std::fs::remove_file(&image_path).ok();
                        continue;
                    }

                    let depth_start = Instant::now();
                    let path_str = image_path.to_str().unwrap_or("temp/unknown.jpg");

                    match depth_service.process_image(path_str, &objects) {
                        Ok(depths) => {
                            let depth_time = depth_start.elapsed();

                            {
                                *depths_arc.lock().unwrap() = depths.clone();
                                *objects_arc.lock().unwrap() = objects.clone();
                                *time_arc.lock().unwrap() = Some(Instant::now());
                            }

                            println!(
                                "\n⚡ DEPTH UPDATE ({:.1}s) - {} objects",
                                depth_time.as_secs_f32(),
                                depths.len()
                            );
                            for (idx, (obj, depth)) in objects.iter().zip(depths.iter()).enumerate()
                            {
                                println!("   {} - {}: {:.1}cm", idx + 1, obj.label, depth.depth_cm);
                            }
                        }
                        Err(e) => {
                            eprintln!("[Depth Worker] Depth error: {}", e);
                        }
                    }

                    std::fs::remove_file(&image_path).ok();

                    *computing_arc.lock().unwrap() = false;
                }
            }
        });
    }

    loop {
        let loop_start = Instant::now();
        frame_count += 1;

        let frame = match detector.get_frame() {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Frame error: {}", e);
                std::thread::sleep(std::time::Duration::from_millis(100));
                continue;
            }
        };

        let mut display_frame = frame.clone();

        let objects = match detector.detect_objects() {
            Ok(objs) => objs,
            Err(e) => {
                eprintln!("Detection error: {}", e);
                std::thread::sleep(std::time::Duration::from_millis(100));
                continue;
            }
        };

        for obj in &objects {
            let color = core::Scalar::new(0.0, 255.0, 0.0, 0.0);

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

        let elapsed = start_time.elapsed().as_secs_f32();
        let fps = frame_count as f32 / elapsed;

        let is_computing = *depth_computing.lock().unwrap();
        let cached_depth_count = cached_depths.lock().unwrap().len();

        let depth_hz = if let Some(last_time) = *last_depth_time.lock().unwrap() {
            let age = last_time.elapsed().as_secs_f32();
            if age < 5.0 {
                format!(" | Depth: {:.1}s ago", age)
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        let status = if stream_mode {
            if is_computing {
                format!(
                    "FPS: {:.1} | Objects: {} | ⚡ Computing...{}",
                    fps,
                    objects.len(),
                    depth_hz
                )
            } else if cached_depth_count > 0 {
                format!(
                    "FPS: {:.1} | Objects: {} | ⚡ Stream active{}",
                    fps,
                    objects.len(),
                    depth_hz
                )
            } else {
                format!(
                    "FPS: {:.1} | Objects: {} | ⚡ Waiting for objects...",
                    fps,
                    objects.len()
                )
            }
        } else {
            if is_computing {
                format!(
                    "FPS: {:.1} | Objects: {} | Computing...",
                    fps,
                    objects.len()
                )
            } else if cached_depth_count > 0 {
                format!("FPS: {:.1} | Objects: {} | Depth ready", fps, objects.len())
            } else {
                format!("FPS: {:.1} | Objects: {} | Press SPACE", fps, objects.len())
            }
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

        let key = highgui::wait_key(1).map_err(|e| {
            robot_hand::HandError::Hardware(format!("Failed to wait for key: {}", e))
        })?;

        if key == 'q' as i32 || key == 27 {
            break;
        }

        if stream_mode && !objects.is_empty() {
            if !is_computing {
                let timestamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis();
                let temp_path = PathBuf::from(format!("temp/stream_{}.jpg", timestamp));

                if let Ok(_) = opencv::imgcodecs::imwrite(
                    temp_path.to_str().unwrap(),
                    &frame,
                    &opencv::core::Vector::new(),
                ) {
                    if let Err(e) = tx.send((temp_path, objects.clone())) {
                        eprintln!("Failed to send to depth worker: {}", e);
                    }
                }
            }
        }

        let should_capture = !stream_mode && key == 32 && !objects.is_empty();

        if should_capture {
            if is_computing {
                continue;
            }

            *depth_computing.lock().unwrap() = true;
            let temp_path = format!("temp/depth_frame_{}.jpg", frame_count);
            let frame_clone = frame.clone();
            let objects_clone = objects.clone();

            let depths_arc = Arc::clone(&cached_depths);
            let objects_arc = Arc::clone(&cached_objects);
            let computing_arc = Arc::clone(&depth_computing);
            let temp_path_clone = temp_path.clone();

            thread::spawn(move || {
                let depth_start = Instant::now();

                if let Err(e) = opencv::imgcodecs::imwrite(
                    &temp_path_clone,
                    &frame_clone,
                    &opencv::core::Vector::new(),
                ) {
                    eprintln!("Failed to save frame: {}", e);
                    *computing_arc.lock().unwrap() = false;
                    return;
                }

                let mut temp_depth_service =
                    match robot_hand::DepthProService::new(Some("venv_depth_pro/bin/python3")) {
                        Ok(s) => s,
                        Err(e) => {
                            eprintln!("Failed to create depth service: {}", e);
                            *computing_arc.lock().unwrap() = false;
                            return;
                        }
                    };

                match temp_depth_service.process_image(&temp_path_clone, &objects_clone) {
                    Ok(depths) => {
                        let depth_time = depth_start.elapsed().as_millis();

                        println!("\n=== DEPTH ANALYSIS ({}ms) ===", depth_time);
                        for (idx, (obj, depth)) in
                            objects_clone.iter().zip(depths.iter()).enumerate()
                        {
                            println!(
                                "Object {}: {} - {:.1}cm",
                                idx + 1,
                                obj.label,
                                depth.depth_cm
                            );
                        }
                        println!("============================\n");

                        *depths_arc.lock().unwrap() = depths;
                        *objects_arc.lock().unwrap() = objects_clone;
                    }
                    Err(e) => {
                        eprintln!("Depth computation error: {}", e);
                    }
                }

                std::fs::remove_file(&temp_path_clone).ok();
                *computing_arc.lock().unwrap() = false;
            });
        }

        {
            let cached_d = cached_depths.lock().unwrap();
            let cached_o = cached_objects.lock().unwrap();

            if !cached_d.is_empty() && !cached_o.is_empty() {
                for (obj, depth) in cached_o.iter().zip(cached_d.iter()) {
                    let depth_color = core::Scalar::new(255.0, 165.0, 0.0, 0.0);
                    let depth_text = format!("{:.0}cm", depth.depth_cm);

                    imgproc::put_text(
                        &mut display_frame,
                        &depth_text,
                        core::Point::new(
                            obj.bounding_box.x,
                            obj.bounding_box.y + obj.bounding_box.height + 20,
                        ),
                        imgproc::FONT_HERSHEY_SIMPLEX,
                        0.6,
                        depth_color,
                        2,
                        imgproc::LINE_8,
                        false,
                    )
                    .ok();
                }
            }
        }

        let loop_time = loop_start.elapsed().as_millis();
        if loop_time < 16 {
            std::thread::sleep(std::time::Duration::from_millis((16 - loop_time) as u64));
        }
    }

    println!("\n========================================");
    println!("Test complete!");
    println!("========================================\n");

    Ok(())
}

#[cfg(not(feature = "opencv"))]
fn main() {
    eprintln!("This example requires the 'opencv' feature.");
    eprintln!("Run with: cargo run --bin depth_integration_test --features opencv");
    std::process::exit(1);
}
