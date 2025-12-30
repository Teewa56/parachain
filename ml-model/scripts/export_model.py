"""
Export PyTorch model to ONNX format for production deployment
"""

import argparse
import torch
from pathlib import Path
import json

import sys
sys.path.append(str(Path(__file__).parent.parent))

from ml.model import BehavioralAuthNN


def export_to_onnx(
    model_path: str,
    output_path: str,
    input_size: int = 7,
    opset_version: int = 11,
):
    """
    Export PyTorch model to ONNX format
    
    Args:
        model_path: Path to PyTorch model (.pth)
        output_path: Path to save ONNX model (.onnx)
        input_size: Input feature dimension
        opset_version: ONNX opset version
    """
    print("=" * 80)
    print("EXPORTING MODEL TO ONNX")
    print("=" * 80)
    
    # Load PyTorch model
    print(f"\n1. Loading PyTorch model from {model_path}")
    model = BehavioralAuthNN(input_dim=input_size)
    model.load_state_dict(torch.load(model_path, map_location='cpu'))
    model.eval()
    print("   ✓ Model loaded")
    
    # Create dummy input
    print(f"\n2. Creating dummy input (batch_size=1, features={input_size})")
    dummy_input = torch.randn(1, input_size)
    
    # Export to ONNX
    print(f"\n3. Exporting to ONNX (opset_version={opset_version})")
    torch.onnx.export(
        model,
        dummy_input,
        output_path,
        export_params=True,
        opset_version=opset_version,
        do_constant_folding=True,
        input_names=['features'],
        output_names=['confidence'],
        dynamic_axes={
            'features': {0: 'batch_size'},
            'confidence': {0: 'batch_size'}
        }
    )
    print(f"   ✓ Exported to {output_path}")
    
    # Verify ONNX model
    print("\n4. Verifying ONNX model")
    try:
        import onnx
        onnx_model = onnx.load(output_path)
        onnx.checker.check_model(onnx_model)
        print("   ✓ ONNX model is valid")
        
        # Print model info
        print("\n5. Model Information:")
        print(f"   Input shape: {onnx_model.graph.input[0].type.tensor_type.shape}")
        print(f"   Output shape: {onnx_model.graph.output[0].type.tensor_type.shape}")
        print(f"   Opset version: {onnx_model.opset_import[0].version}")
        
    except ImportError:
        print("   ⚠ ONNX package not installed, skipping verification")
        print("   Install with: pip install onnx")
    except Exception as e:
        print(f"   ✗ Verification failed: {e}")
        return False
    
    # Test inference
    print("\n6. Testing ONNX inference")
    try:
        import onnxruntime as ort
        
        session = ort.InferenceSession(output_path)
        
        # Run inference
        test_input = dummy_input.numpy()
        outputs = session.run(None, {'features': test_input})
        
        print(f"   ✓ Inference successful")
        print(f"   Output shape: {outputs[0].shape}")
        print(f"   Sample output: {outputs[0][0][0]:.4f}")
        
    except ImportError:
        print("   ⚠ ONNX Runtime not installed, skipping inference test")
        print("   Install with: pip install onnxruntime")
    except Exception as e:
        print(f"   ✗ Inference test failed: {e}")
        return False
    
    # Save metadata
    metadata_path = Path(output_path).with_suffix('.json')
    metadata = {
        "model_type": "BehavioralAuthNN",
        "input_size": input_size,
        "opset_version": opset_version,
        "pytorch_model": str(model_path),
        "onnx_model": str(output_path),
    }
    
    with open(metadata_path, 'w') as f:
        json.dump(metadata, f, indent=2)
    
    print(f"\n   ✓ Metadata saved to {metadata_path}")
    
    print("\n" + "=" * 80)
    print("EXPORT COMPLETE")
    print("=" * 80)
    print(f"\nONNX model: {output_path}")
    print(f"Metadata: {metadata_path}")
    
    # Print usage instructions
    print("\n" + "=" * 80)
    print("USAGE IN PRODUCTION")
    print("=" * 80)
    print("""
# Python
import onnxruntime as ort
import numpy as np

session = ort.InferenceSession('model.onnx')
features = np.array([[65, 120, 85, 3, 14, 0.65, 1.41]], dtype=np.float32)
output = session.run(None, {'features': features})
confidence = output[0][0][0] * 100
print(f"Confidence: {confidence:.2f}%")

# JavaScript (using onnxruntime-web)
const session = await ort.InferenceSession.create('model.onnx');
const features = new ort.Tensor('float32', [65, 120, 85, 3, 14, 0.65, 1.41], [1, 7]);
const outputs = await session.run({ features });
const confidence = outputs.confidence.data[0] * 100;
console.log(`Confidence: ${confidence.toFixed(2)}%`);
""")
    
    return True


def compare_models(pytorch_path: str, onnx_path: str, num_samples: int = 100):
    """
    Compare PyTorch and ONNX model outputs
    
    Args:
        pytorch_path: Path to PyTorch model
        onnx_path: Path to ONNX model
        num_samples: Number of random samples to test
    """
    print("\n" + "=" * 80)
    print("COMPARING PYTORCH AND ONNX MODELS")
    print("=" * 80)
    
    # Load PyTorch model
    pytorch_model = BehavioralAuthNN(input_dim=7)
    pytorch_model.load_state_dict(torch.load(pytorch_path, map_location='cpu'))
    pytorch_model.eval()
    
    # Load ONNX model
    try:
        import onnxruntime as ort
        onnx_session = ort.InferenceSession(onnx_path)
    except ImportError:
        print("ONNX Runtime not installed. Install with: pip install onnxruntime")
        return
    
    # Generate random samples
    import numpy as np
    
    max_diff = 0
    avg_diff = 0
    
    for i in range(num_samples):
        # Random input
        features = torch.randn(1, 7)
        
        # PyTorch inference
        with torch.no_grad():
            pytorch_output = pytorch_model(features).numpy()
        
        # ONNX inference
        onnx_output = onnx_session.run(None, {'features': features.numpy()})[0]
        
        # Calculate difference
        diff = np.abs(pytorch_output - onnx_output).max()
        max_diff = max(max_diff, diff)
        avg_diff += diff
    
    avg_diff /= num_samples
    
    print(f"\nTested {num_samples} random samples:")
    print(f"  Max difference: {max_diff:.6f}")
    print(f"  Avg difference: {avg_diff:.6f}")
    
    if max_diff < 1e-5:
        print("\n✓ Models match (difference < 1e-5)")
    else:
        print(f"\n⚠ Models differ by {max_diff:.6f}")


def main():
    parser = argparse.ArgumentParser(description='Export PyTorch model to ONNX')
    parser.add_argument('--model', type=str, required=True, help='Path to PyTorch model')
    parser.add_argument('--output', type=str, required=True, help='Output ONNX path')
    parser.add_argument('--input-size', type=int, default=7, help='Input feature dimension')
    parser.add_argument('--opset-version', type=int, default=11, help='ONNX opset version')
    parser.add_argument('--compare', action='store_true', help='Compare PyTorch and ONNX outputs')
    
    args = parser.parse_args()
    
    # Create output directory
    output_path = Path(args.output)
    output_path.parent.mkdir(parents=True, exist_ok=True)
    
    # Export model
    success = export_to_onnx(
        model_path=args.model,
        output_path=args.output,
        input_size=args.input_size,
        opset_version=args.opset_version,
    )
    
    if not success:
        print("\n✗ Export failed")
        return 1
    
    # Compare models if requested
    if args.compare:
        compare_models(args.model, args.output)
    
    return 0


if __name__ == '__main__':
    exit(main())