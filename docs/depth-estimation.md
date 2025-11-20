# Depth Estimation Integration

## Overview

This system integrates Apple Depth Pro (Core ML) for accurate metric depth estimation, providing actual distances in centimeters for autonomous robotic control. The Core ML implementation leverages Apple's Neural Engine for optimized performance on Apple Silicon.

## Architecture

```
Camera -> YOLO Detection -> Core ML Depth Pro -> Distance Estimation -> Robot Control
```

The pipeline:
1. Camera captures frame
2. YOLO detects objects with bounding boxes
3. Core ML Depth Pro analyzes full image to generate depth map (using Neural Engine)
4. Extract depth values at object bounding box locations
5. Calculate median depth for each object
6. Feed distances to control system

## Installation

### Prerequisites

- **Python 3.9-3.11** (recommended for best Core ML compatibility)
- Rust with cargo
- OpenCV (for camera capture)
- macOS with Apple Silicon (M1/M2/M3) for Neural Engine acceleration

### Setup Script

```bash
./setup_depth_pro.sh
```

This script will:
1. Create a Python virtual environment (using system Python 3.9 if available)
2. Install Core ML tools (`coremltools`, `pillow`, `numpy`, `scipy`)
3. Download the Core ML Depth Pro model from Hugging Face to `~/coreml-depthpro/` (~710MB weights file)

**Note**: 
- The model download may take several minutes (~710MB weights file)
- The model will be stored in your home directory at `~/coreml-depthpro/DepthProNormalizedInverseDepthPruned10QuantizedLinear.mlpackage/`
- This directory is **not** in the repository (it's in your home folder), so each user must download it
- The `depth_service.py` script automatically finds the model at this location

### Verify Installation

```bash
source venv_depth_pro/bin/activate
python3 depth_service.py test <path_to_image.jpg>
```

Or test with a bounding box:
```bash
python3 depth_service.py test <image.jpg> 100 100 200 150
```

### Important: Gitignored Files

The following directories are gitignored and must be set up locally:
- `venv_depth_pro/` - Python virtual environment (created by setup script)
- `venv_depth_pro_system/` - Alternative Python environment (if created)
- `temp/` - Temporary image files for processing
- `test_image.jpg`, `*.jpg`, `*.png` - Test output images

These will be created automatically when you run the setup script and use the system.

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
// Note: The setup script creates venv_depth_pro. If you created venv_depth_pro_system instead,
// use: DepthProService::new(Some("venv_depth_pro_system/bin/python3"))?

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
// The service loads the Core ML model once at startup (takes ~10-20s)
// After that, process_image() calls are fast and consistent (~6.7s each)

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

- **Model loading**: ~10-20 seconds (one-time cost when service starts)
- **Depth Pro inference**: ~6-7 seconds per frame (Core ML with Neural Engine)
- **YOLO detection**: ~140ms per frame
- **Total pipeline**: ~6-7 seconds per frame (depth is the bottleneck)

**Note**: After the initial model load, successive image processing is consistent and fast (~6.7s per image). The model uses Apple's Neural Engine for acceleration on M1/M2/M3 Macs.

### Stream Mode

Stream mode provides continuous depth updates for robot control:

```bash
./run_depth_test.sh --stream
```

Features:
- Non-blocking operation
- UI runs at full camera FPS (approximately 7 FPS)
- Depth updates at ~0.15 Hz (one update per ~6.7 seconds)
- Background processing prevents UI freezing
- Always-fresh cached depth data
- Model loads once at startup, then processes images efficiently

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

### Module not found: coremltools

```bash
source venv_depth_pro/bin/activate
pip install coremltools pillow numpy scipy
```

### Model not found error

The model should be at `~/coreml-depthpro/DepthProNormalizedInverseDepthPruned10QuantizedLinear.mlpackage/`.

If missing, download manually:
```bash
pip install huggingface_hub
python3 -c "
from huggingface_hub import snapshot_download
import os
snapshot_download(
    repo_id='KeighBee/coreml-DepthPro',
    local_dir=os.path.expanduser('~/coreml-depthpro'),
    allow_patterns='DepthProNormalizedInverseDepthPruned10QuantizedLinear.mlpackage/**'
)
"
```

Or visit: https://huggingface.co/KeighBee/coreml-DepthPro/tree/main

### Service fails to start

Verify Python path and version:

```bash
which python3
python3 --version  # Should be 3.9-3.11 for best compatibility
```

If using a different Python version, update the path in `DepthProService::new()`:
```rust
let mut depth_service = DepthProService::new(Some("/path/to/python3"))?;
```

### Core ML native libraries not loading

This usually happens with Python 3.12+. The setup script will try to use system Python 3.9 if available. Alternatively:

1. Use system Python: `/usr/bin/python3` (usually Python 3.9)
2. Or install Python 3.9-3.11 via Homebrew and use that

### Slow inference

- Ensure you're on Apple Silicon (M1/M2/M3) for Neural Engine acceleration
- The ~6-7 second inference time is expected for this model size (1536x1536 input)
- Process every Nth frame instead of every frame for real-time applications
- Consider using stream mode with background processing (see Stream Mode section)

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

