#!/usr/bin/env python3
"""
Depth Pro service for robot hand depth estimation using Core ML.
Processes images and returns metric depth maps.
"""

import sys
import json
import os
import numpy as np
from PIL import Image
import coremltools as ct

try:
    from coremltools.models import MLModel
except ImportError:
    print(json.dumps({"error": "coremltools not installed. Run: pip install coremltools"}))
    sys.exit(1)

try:
    from scipy import ndimage
    HAS_SCIPY = True
except ImportError:
    HAS_SCIPY = False


class DepthService:
    def __init__(self, model_path=None):
        print("Loading Core ML Depth Pro model...", file=sys.stderr)
        
        if model_path is None:
            home_dir = os.path.expanduser("~")
            model_path = os.path.join(home_dir, "coreml-depthpro", "DepthProNormalizedInverseDepthPruned10QuantizedLinear.mlpackage")
        
        if not os.path.exists(model_path):
            print(json.dumps({
                "error": f"Model not found at {model_path}. Run setup_depth_pro.sh to download it."
            }), file=sys.stderr)
            sys.exit(1)
        
        try:
            self.model = MLModel(model_path, compute_units=ct.ComputeUnit.ALL)
            print("Using Core ML with Neural Engine acceleration", file=sys.stderr)
        except Exception as e:
            print(json.dumps({"error": f"Failed to load Core ML model: {e}"}), file=sys.stderr)
            sys.exit(1)
        
        print("Depth Pro ready!", file=sys.stderr)
    
    def _preprocess_image(self, image_path):
        """Load and preprocess image for Core ML model.
        
        Optimized for fast successive calls - minimal allocations.
        """
        image = Image.open(image_path).convert("RGB")
        original_size = image.size
        
        input_size = 1536
        image_resized = image.resize((input_size, input_size), Image.Resampling.LANCZOS)
        
        return image_resized, original_size
    
    def _estimate_focal_length(self, image_width, image_height):
        """Estimate focal length from image dimensions."""
        diagonal = np.sqrt(image_width**2 + image_height**2)
        focal_length_px = diagonal / np.sqrt(2)
        return focal_length_px
    
    def _convert_normalized_inverse_depth_to_meters(self, normalized_inverse_depth, image_width, image_height):
        """
        Convert normalized inverse depth to metric depth in meters.
        
        The Core ML model outputs normalized inverse depth (0-1 range, where higher = closer).
        We convert: depth = scale / (normalized_inverse_depth + epsilon)
        """
        epsilon = 1e-6
        
        normalized_inverse_depth = np.clip(normalized_inverse_depth, epsilon, 1.0 - epsilon)
        
        diagonal = np.sqrt(image_width**2 + image_height**2)
        
        scale_factor = diagonal * 0.001
        
        depth_meters = scale_factor / (normalized_inverse_depth + epsilon)
        
        depth_meters = np.clip(depth_meters, 0.05, 5.0)
        
        return depth_meters
    
    def process_image(self, image_path, bounding_boxes=None):
        """
        Process image and return depth map.
        
        Args:
            image_path: Path to image file
            bounding_boxes: Optional list of [x, y, w, h] boxes to get depth for
        
        Returns:
            {
                "depth_map": base64 encoded numpy array,
                "focal_length_px": float,
                "objects": [{"bbox": [x,y,w,h], "depth_meters": float, "depth_cm": float}]
            }
        """
        image_preprocessed, original_size = self._preprocess_image(image_path)
        
        try:
            prediction = self.model.predict({"image": image_preprocessed})
        except Exception as e:
            raise RuntimeError(f"Core ML inference failed: {e}")
        
        depth_output = prediction.get("depth", None)
        if depth_output is None:
            depth_output = list(prediction.values())[0]
        
        if isinstance(depth_output, np.ndarray):
            depth_map_raw = depth_output
        else:
            depth_map_raw = np.asarray(depth_output)
        
        while len(depth_map_raw.shape) > 2:
            if depth_map_raw.shape[0] == 1:
                depth_map_raw = depth_map_raw[0]
            elif len(depth_map_raw.shape) == 3:
                depth_map_raw = depth_map_raw.squeeze()
            else:
                depth_map_raw = depth_map_raw[0]
        
        original_width, original_height = original_size
        
        if depth_map_raw.shape[:2] != (original_height, original_width):
            if HAS_SCIPY:
                scale_y = original_height / depth_map_raw.shape[0]
                scale_x = original_width / depth_map_raw.shape[1]
                depth_map_raw = ndimage.zoom(depth_map_raw, (scale_y, scale_x), order=1, prefilter=False)
            else:
                depth_min = depth_map_raw.min()
                depth_max = depth_map_raw.max()
                depth_range = depth_max - depth_min if depth_max > depth_min else 1.0
                
                depth_normalized = (depth_map_raw - depth_min) / depth_range
                depth_uint16 = (depth_normalized * 65535).astype(np.uint16)
                
                depth_pil = Image.fromarray(depth_uint16)
                depth_pil = depth_pil.resize((original_width, original_height), Image.Resampling.LANCZOS)
                depth_resized = np.array(depth_pil).astype(np.float32) / 65535.0
                
                depth_map_raw = depth_resized * depth_range + depth_min
        
        depth_map = self._convert_normalized_inverse_depth_to_meters(
            depth_map_raw, original_width, original_height
        )
        
        focal_length = self._estimate_focal_length(original_width, original_height)
        
        result = {
            "focal_length_px": float(focal_length),
            "depth_map_shape": list(depth_map.shape),
        }
        
        if bounding_boxes:
            objects = []
            for bbox in bounding_boxes:
                x, y, w, h = bbox
                x, y, w, h = int(x), int(y), int(w), int(h)
                
                x = max(0, min(x, depth_map.shape[1] - 1))
                y = max(0, min(y, depth_map.shape[0] - 1))
                w = max(1, min(w, depth_map.shape[1] - x))
                h = max(1, min(h, depth_map.shape[0] - y))
                
                depth_region = depth_map[y:y+h, x:x+w]
                
                median_depth = float(np.median(depth_region))
                mean_depth = float(np.mean(depth_region))
                min_depth = float(np.min(depth_region))
                
                objects.append({
                    "bbox": [x, y, w, h],
                    "depth_meters": median_depth,
                    "depth_cm": median_depth * 100,
                    "depth_mean_meters": mean_depth,
                    "depth_min_meters": min_depth,
                })
            
            result["objects"] = objects
        
        return result
    
    def run_server(self):
        """Run as a service, reading JSON requests from stdin."""
        print("Depth service ready. Waiting for requests...", file=sys.stderr)
        
        for line in sys.stdin:
            try:
                request = json.loads(line.strip())
                
                if request.get("command") == "process":
                    image_path = request.get("image_path")
                    bboxes = request.get("bounding_boxes", [])
                    
                    result = self.process_image(image_path, bboxes)
                    result["status"] = "success"
                    
                    print(json.dumps(result), flush=True)
                
                elif request.get("command") == "ping":
                    print(json.dumps({"status": "ok"}), flush=True)
                
                elif request.get("command") == "exit":
                    break
                
            except Exception as e:
                error_result = {
                    "status": "error",
                    "error": str(e)
                }
                print(json.dumps(error_result), flush=True)


def main():
    if len(sys.argv) > 1 and sys.argv[1] == "test":
        service = DepthService()
        
        if len(sys.argv) < 3:
            print("Usage: depth_service.py test <image_path> [x,y,w,h]")
            sys.exit(1)
        
        image_path = sys.argv[2]
        bboxes = None
        
        if len(sys.argv) >= 7:
            x, y, w, h = map(int, sys.argv[3:7])
            bboxes = [[x, y, w, h]]
        
        result = service.process_image(image_path, bboxes)
        print(json.dumps(result, indent=2))
    
    else:
        service = DepthService()
        service.run_server()


if __name__ == "__main__":
    main()

