# AI Model Integration Guide

## Object Tracking Data Format

The vision system outputs structured data for AI model processing, designed for depth estimation and robot control.

## Data Structure

```rust
pub struct ObjectTrackingData {
    pub object: DetectedObject,
    pub center_x_norm: f32,
    pub center_y_norm: f32,
    pub width_norm: f32,
    pub height_norm: f32,
    pub area_ratio: f32,
    pub estimated_depth_cm: f32,
    pub horizontal_angle_deg: f32,
    pub vertical_angle_deg: f32,
    pub frame_width: i32,
    pub frame_height: i32,
    pub timestamp_ms: u64,
}
```

### Key Features

**Normalized Coordinates** (0.0 to 1.0)
- center_x_norm, center_y_norm: Object center position
- width_norm, height_norm: Bounding box dimensions
- Scale-invariant for different resolutions

**Angular Position** (degrees)
- horizontal_angle_deg: -30 (left) to +30 (right)
- vertical_angle_deg: -22.5 (bottom) to +22.5 (top)
- Direct mapping to joint angles

**Size Ratios**
- area_ratio: Object area as fraction of frame
- Useful for distance estimation (larger ratio indicates closer object)

**Depth Estimate** (centimeters)
- estimated_depth_cm: Initial heuristic-based distance
- Can be improved with trained AI model

## Usage Example

```rust
use robot_hand::{OpenCVDetector, create_tracking_data, ObjectDetector};

let mut detector = OpenCVDetector::new(0, 0.5)?;
detector.load_yolo_model("models/yolov8n.onnx")?;

let objects = detector.detect_objects()?;
let (width, height) = detector.get_frame_size();

for obj in objects {
    let tracking = create_tracking_data(&obj, width, height);
    
    let distance = depth_model.estimate(&tracking);
    
    robot.move_to_pickup(distance, obj.label);
}
```

## JSON Output Format

When compiled with serde_support:

```json
{
  "object": {
    "label": "bottle",
    "confidence": 0.85,
    "bounding_box": {
      "x": 500,
      "y": 300,
      "width": 150,
      "height": 200
    },
    "distance": 0.5
  },
  "center_x_norm": 0.52,
  "center_y_norm": 0.46,
  "width_norm": 0.078,
  "height_norm": 0.185,
  "area_ratio": 0.014,
  "estimated_depth_cm": 45.0,
  "horizontal_angle_deg": 1.2,
  "vertical_angle_deg": -2.7,
  "frame_width": 1920,
  "frame_height": 1080,
  "timestamp_ms": 1700000000000
}
```

## Training Custom Depth Model

### Collecting Training Data

Place objects at known distances and capture images:

```bash
cargo build --bin collect_training_data --features opencv --release

./target/release/collect_training_data bottle 20
```

Press SPACE to capture images at the specified distance (20cm in this example). Move the object and repeat for different distances: 20, 25, 30, 35, 40, 45, 50cm.

Output files:
- training_data/bottle_20cm_001_crop.jpg
- training_data/bottle_20cm_001_context.jpg
- training_data/bottle_20cm_001_full.jpg
- training_data/bottle_20cm_001_data.json
- training_data/labels.csv

### Model Architecture

Combine image features with metadata for accurate depth estimation:

```python
import torch
import torch.nn as nn
from torchvision import models

class DepthEstimator(nn.Module):
    def __init__(self):
        super().__init__()
        self.image_net = models.resnet18(pretrained=True)
        self.image_net.fc = nn.Linear(512, 64)
        
        self.meta_net = nn.Sequential(
            nn.Linear(5, 32),
            nn.ReLU(),
            nn.Linear(32, 64)
        )
        
        self.fusion = nn.Sequential(
            nn.Linear(128, 64),
            nn.ReLU(),
            nn.Linear(64, 1)
        )
    
    def forward(self, image, metadata):
        img_features = self.image_net(image)
        meta_features = self.meta_net(metadata)
        combined = torch.cat([img_features, meta_features], dim=1)
        distance = self.fusion(combined)
        return distance
```

### Training Script

```python
import pandas as pd
import torch
from torchvision import transforms
from PIL import Image
from torch.utils.data import Dataset, DataLoader

df = pd.read_csv('training_data/labels.csv')

class DepthDataset(Dataset):
    def __init__(self, df):
        self.df = df
        self.transform = transforms.Compose([
            transforms.Resize((224, 224)),
            transforms.ToTensor(),
        ])
    
    def __len__(self):
        return len(self.df)
    
    def __getitem__(self, idx):
        row = self.df.iloc[idx]
        img = Image.open(f"training_data/{row['crop_image']}")
        img = self.transform(img)
        distance = torch.tensor([row['distance_cm']], dtype=torch.float32)
        return img, distance

dataset = DepthDataset(df)
loader = DataLoader(dataset, batch_size=8, shuffle=True)

model = models.resnet18(pretrained=True)
model.fc = torch.nn.Linear(512, 1)
optimizer = torch.optim.Adam(model.parameters(), lr=0.001)
criterion = torch.nn.MSELoss()

for epoch in range(50):
    total_loss = 0
    for images, distances in loader:
        pred = model(images)
        loss = criterion(pred, distances)
        
        optimizer.zero_grad()
        loss.backward()
        optimizer.step()
        
        total_loss += loss.item()
    
    print(f"Epoch {epoch+1}/50, Loss: {total_loss/len(loader):.2f}")

dummy_input = torch.randn(1, 3, 224, 224)
torch.onnx.export(model, dummy_input, "models/depth_model.onnx")
```

### Integration in Rust

```rust
let depth_session = ort::Session::builder()?
    .commit_from_file("models/depth_model.onnx")?;

for obj in objects {
    let tracking = create_tracking_with_image(&obj, &frame)?;
    
    let input_tensor = preprocess_for_depth_model(&tracking.cropped_object)?;
    
    let outputs = depth_session.run(ort::inputs![input_tensor])?;
    let distance_cm: f32 = outputs[0].extract_tensor::<f32>()?[0];
    
    println!("{} is at {:.1}cm", obj.label, distance_cm);
}
```

## Test Commands

View tracking output:

```bash
./run_tracking_test.sh
```

Or build manually:

```bash
cargo run --bin tracking_output_test --features opencv --release
```

## Performance

- Detection: approximately 7 FPS at 640x640 input
- Tracking data creation: less than 1ms
- JSON serialization: less than 1ms per object
- Suitable for real-time robot control

## Integration Pipeline

```
Camera -> YOLO Detection -> ObjectTrackingData -> AI Depth Model -> Robot Control
```

1. Camera captures frame
2. YOLO identifies objects with bounding boxes and confidence scores
3. Convert to normalized coordinates and calculate angles
4. Optional: AI model estimates accurate 3D position
5. Control system plans joint movements
6. Robot executes pickup sequence

## Accuracy Comparison

| Approach | Accuracy | Requirements |
|----------|----------|--------------|
| Bounding box only | ±30% | None |
| Pre-trained depth model | ±10% | Model download |
| Custom trained model | ±5% | Training data |

## Best Practices

- Collect training data with varied lighting conditions
- Include multiple object orientations
- Use at least 50-100 training samples
- Validate on separate test set
- Fine-tune for specific camera setup
- Combine multiple depth cues when possible

