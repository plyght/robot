# Depth Estimation Integration

## Overview

This system integrates Apple Depth Pro for accurate metric depth estimation, providing actual distances in centimeters for autonomous robotic control.

## Architecture

```
Camera -> YOLO Detection -> Depth Pro -> Distance Estimation -> Robot Control
```

The pipeline:
1. Camera captures frame
2. YOLO detects objects with bounding boxes
3. Depth Pro analyzes full image to generate depth map
4. Extract depth values at object bounding box locations
5. Calculate median depth for each object
6. Feed distances to control system

## Installation

### Prerequisites

- Python 3.12 or later
- Rust with cargo
- OpenCV (for camera capture)

### Setup Script

```bash
./setup_depth_pro.sh
```

This creates a Python virtual environment, installs Apple's depth_pro package, and downloads the pretrained model (approximately 1.5GB).

### Verify Installation

```bash
source venv_depth_pro/bin/activate
python3 depth_service.py test data/example.jpg
```

## Usage

### Automatic Cleanup

The system provides automatic cleanup of temporary image files to prevent storage buildup. Use `process_image_with_cleanup` with the cleanup flag set to `true`:

```rust
use robot_hand::{ensure_temp_dir, cleanup_temp_files};

ensure_temp_dir()?;

// ... during operation ...
depth_service.process_image_with_cleanup("temp/frame.jpg", &objects, true)?;

// Or cleanup all temp files at shutdown
cleanup_temp_files();
```

### Basic Integration

```rust
use robot_hand::{OpenCVDetector, DepthProService, ObjectDetector};
use std::fs;

let mut detector = OpenCVDetector::new(0, 0.55)?;
detector.load_yolo_model("models/yolov8n.onnx")?;

let mut depth_service = DepthProService::new(Some("venv_depth_pro/bin/python3"))?;

fs::create_dir_all("temp")?;

loop {
    let frame = detector.get_frame()?;
    let objects = detector.detect_objects()?;
    
    opencv::imgcodecs::imwrite("temp/frame.jpg", &frame, &opencv::core::Vector::new())?;
    
    let depths = depth_service.process_image_with_cleanup("temp/frame.jpg", &objects, true)?;
    
    for (obj, depth) in objects.iter().zip(depths.iter()) {
        println!("{} at {:.1}cm", obj.label, depth.depth_cm);
    }
}
```

The `process_image_with_cleanup` method automatically deletes the temporary image file after processing to avoid storage buildup.

### Running Tests

Manual mode (press SPACE to capture depth):

```bash
./run_depth_test.sh
```

Automatic continuous mode (recommended for robot control):

```bash
./run_depth_test.sh --auto
```

Custom capture interval:

```bash
./run_depth_test.sh --auto 30  # Every 30 frames
```

## API Reference

### DepthProService

```rust
let mut depth_service = DepthProService::new(Some("venv_depth_pro/bin/python3"))?;

let depths: Vec<ObjectDepth> = depth_service.process_image(
    "path/to/image.jpg",
    &detected_objects
)?;

depth_service.ping()?;
```

### ObjectDepth

```rust
pub struct ObjectDepth {
    pub bbox: [i32; 4],
    pub depth_meters: f32,
    pub depth_cm: f32,
    pub depth_mean_meters: f32,
    pub depth_min_meters: f32,
}
```

## Performance

- Depth Pro inference: 300-500ms per frame (GPU) or 1-2s (CPU)
- YOLO detection: 140ms per frame
- Total pipeline: approximately 500ms per frame

### Stream Mode

Stream mode provides continuous depth updates for robot control:

```bash
./run_depth_test.sh --stream
```

Features:
- Non-blocking operation
- UI runs at full camera FPS (approximately 7 FPS)
- Depth updates at approximately 2 Hz
- Background processing prevents UI freezing
- Always-fresh cached depth data

## Accuracy

Depth Pro provides metric depth (actual meters) without calibration:
- Absolute scale measurement
- High frequency detail preservation
- Automatic focal length estimation
- Expected accuracy: 5-10% for objects 20cm-2m away

## Robot Control Integration

### Continuous Operation

```rust
let cached_depths = Arc::new(Mutex::new(Vec::new()));
let depth_queue = spawn_depth_worker(cached_depths.clone());

loop {
    let frame = camera.get_frame();
    let objects = yolo.detect(frame);
    
    depth_queue.send((frame, objects));
    
    let depths = cached_depths.lock().unwrap();
    
    if let Some(target_depth) = depths.get(0) {
        let prompt = format!(
            "Target: {} at {:.0}cm, {:.1} degrees from center",
            target_depth.label, 
            target_depth.depth_cm,
            calculate_angle(&target_depth)
        );
        
        let commands = control_system.plan(prompt);
        robot.execute(commands);
    }
    
    display.show(frame, depths);
}
```

## Storage Management

### Automatic Cleanup

To prevent temporary image files from accumulating on your system:

1. **Per-image cleanup**: Use `process_image_with_cleanup` with `cleanup = true`
   ```rust
   depth_service.process_image_with_cleanup("temp/frame.jpg", &objects, true)?;
   ```

2. **Batch cleanup**: Remove all temporary images at once
   ```rust
   use robot_hand::cleanup_temp_files;
   cleanup_temp_files();
   ```

3. **Directory management**: Ensure temp directory exists before use
   ```rust
   use robot_hand::ensure_temp_dir;
   ensure_temp_dir()?;
   ```

### Best Practices

- Call `ensure_temp_dir()` at application startup
- Use `process_image_with_cleanup(..., true)` during normal operation
- Call `cleanup_temp_files()` on application shutdown
- Temporary files are automatically deleted after depth processing when using cleanup mode
- The temp directory is in `.gitignore` and will not be committed

## Troubleshooting

### Module not found: depth_pro

```bash
source venv_depth_pro/bin/activate
pip install git+https://github.com/apple/ml-depth-pro.git
```

### Service fails to start

Verify Python path:

```bash
which python3
```

Use the correct path in DepthProService::new().

### Model download fails

Manual download:

```bash
cd checkpoints
curl -L -o depth_pro.pt "https://ml-site.cdn-apple.com/models/depth-pro/depth_pro.pt"
```

### Slow inference

- Use GPU acceleration if available (CUDA or Apple Silicon MPS)
- Reduce camera resolution
- Process every Nth frame instead of every frame

## Alternative Approaches

### Why Images Are Essential

Bounding box data alone provides rough estimates but lacks critical information:
- Cannot determine actual object size (toy vs. full-size bottle)
- Misses object orientation (rotated objects appear smaller)
- Lacks depth cues (shadows, texture, occlusions)
- Fails with unknown objects

Visual depth estimation uses:
- Relative object sizes
- Texture gradients (blur indicates distance)
- Shadow analysis
- Occlusion relationships
- Perspective lines
- Learned object size patterns

### Pre-trained Alternatives

Other models for depth estimation:
- MiDaS: General purpose depth estimation
- Depth-Anything: State-of-the-art as of 2024
- ZoeDepth: Metric depth with meter output

### Custom Model Training

For specific camera setups, train a custom model:

1. Collect training data at known distances
2. Train CNN combining image features and metadata
3. Export to ONNX format
4. Integrate via ort crate in Rust

See docs/ai-integration.md for detailed instructions.

