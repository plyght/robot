#!/usr/bin/env python3
"""
Depth Pro service for robot hand depth estimation.
Processes images and returns metric depth maps.
"""

import sys
import json
import base64
import io
import numpy as np
from PIL import Image
import torch

try:
    import depth_pro
except ImportError:
    print(json.dumps({"error": "depth_pro not installed. Run: pip install git+https://github.com/apple/ml-depth-pro.git"}))
    sys.exit(1)


class DepthService:
    def __init__(self):
        print("Loading Depth Pro model...", file=sys.stderr)
        self.model, self.transform = depth_pro.create_model_and_transforms()
        self.model.eval()
        
        if torch.cuda.is_available():
            self.model = self.model.cuda()
            print("Using GPU", file=sys.stderr)
        elif torch.backends.mps.is_available():
            self.model = self.model.to("mps")
            print("Using MPS (Apple Silicon)", file=sys.stderr)
        else:
            print("Using CPU", file=sys.stderr)
        
        print("Depth Pro ready!", file=sys.stderr)
    
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
        image, _, f_px = depth_pro.load_rgb(image_path)
        image_tensor = self.transform(image)
        
        if torch.cuda.is_available():
            image_tensor = image_tensor.cuda()
        elif torch.backends.mps.is_available():
            image_tensor = image_tensor.to("mps")
        
        with torch.no_grad():
            prediction = self.model.infer(image_tensor, f_px=f_px)
        
        depth_map = prediction["depth"].cpu().numpy()
        focal_length = float(prediction["focallength_px"])
        
        result = {
            "focal_length_px": focal_length,
            "depth_map_shape": depth_map.shape,
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

