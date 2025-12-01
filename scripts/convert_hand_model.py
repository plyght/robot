#!/usr/bin/env python3

import os
import sys
import zipfile
import tempfile
import shutil

def extract_tflite_from_task(task_path, output_dir):
    print(f"Extracting TFLite model from {task_path}...")

    with zipfile.ZipFile(task_path, 'r') as zip_ref:
        zip_ref.extractall(output_dir)

    for root, dirs, files in os.walk(output_dir):
        for file in files:
            if file.endswith('.tflite'):
                tflite_path = os.path.join(root, file)
                print(f"Found TFLite model: {tflite_path}")
                return tflite_path

    raise FileNotFoundError("No .tflite file found in .task archive")


def convert_tflite_to_onnx(tflite_path, onnx_path):
    print(f"Converting {tflite_path} to ONNX...")

    try:
        import tensorflow as tf
        import tf2onnx
    except ImportError:
        print("ERROR: Missing dependencies. Install with:")
        print("  pip install tensorflow tf2onnx onnx")
        sys.exit(1)

    interpreter = tf.lite.Interpreter(model_path=tflite_path)
    interpreter.allocate_tensors()

    input_details = interpreter.get_input_details()
    output_details = interpreter.get_output_details()

    print(f"Input shape: {input_details[0]['shape']}")
    print(f"Output shape: {output_details[0]['shape']}")

    print("\nConverting TFLite → ONNX...")
    print("This requires tensorflow and tf2onnx:")
    print("  pip install tensorflow tf2onnx onnx")
    print("\nAlternatively, we can use the model directly with TFLite in Rust.")
    print("However, ONNX Runtime integration is already implemented.")

    return False


def main():
    task_path = "models/hand_landmarker.task"
    onnx_path = "models/hand_landmarker.onnx"

    if not os.path.exists(task_path):
        print(f"ERROR: {task_path} not found")
        sys.exit(1)

    temp_dir = tempfile.mkdtemp()

    try:
        tflite_path = extract_tflite_from_task(task_path, temp_dir)

        final_tflite = "models/hand_landmarker.tflite"
        shutil.copy(tflite_path, final_tflite)
        print(f"\nExtracted TFLite model to: {final_tflite}")

        print("\n" + "="*60)
        print("CONVERSION NOTE:")
        print("="*60)
        print("TFLite → ONNX conversion requires complex model graph analysis.")
        print("Instead, we have 3 options:")
        print()
        print("1. Use pre-converted ONNX model (if available online)")
        print("2. Convert using AI2ONNX or onnx-tensorflow tools")
        print("3. Modify Rust code to use TFLite directly")
        print()
        print("For now, I'll search for pre-converted models...")

    finally:
        shutil.rmtree(temp_dir)


if __name__ == "__main__":
    main()
