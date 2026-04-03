#!/usr/bin/env python3
"""
Stable Audio Open - INT8 量化脚本
将模型从 FP32/FP16 量化到 INT8，大幅减小体积和显存占用
"""
import sys
import os

os.environ['HF_ENDPOINT'] = 'https://hf-mirror.com'

import argparse
import json
import shutil
from pathlib import Path

import torch
from diffusers import StableAudioPipeline
from safetensors.torch import load_file, save_file

def get_model_path(base_path: str = None):
    if base_path:
        return base_path
    script_dir = os.path.dirname(os.path.abspath(__file__))
    return os.path.join(script_dir, "model", "f21265c1e2710b3bd2386596943f0007f55f802e")

def get_quantized_model_path(base_model_path: str):
    return base_model_path + "_int8"

def format_size(size_bytes):
    if size_bytes >= 1024**3:
        return f"{size_bytes / 1024**3:.2f} GB"
    elif size_bytes >= 1024**2:
        return f"{size_bytes / 1024**2:.2f} MB"
    else:
        return f"{size_bytes / 1024:.2f} KB"

def get_safetensors_size(path):
    if os.path.exists(path):
        tensors = load_file(path)
        total = sum(t.nelement() * t.element_size() for t in tensors.values())
        return total
    return 0

def quantize_tensor_fp16(tensor):
    return tensor.to(torch.float16)

def quantize_tensor_int8_dynamic(tensor):
    if tensor.dtype in [torch.float32, torch.float16]:
        orig_size = tensor.nelement() * tensor.element_size()
        
        scale = tensor.abs().max()
        if scale == 0:
            scale = 1.0
        
        quantized = (tensor / scale * 127).to(torch.int8)
        
        return {
            'data': quantized,
            'scale': scale,
            'dtype': str(tensor.dtype)
        }
    return tensor

def dequantize_tensor_int8(qtensor):
    if isinstance(qtensor, dict) and 'data' in qtensor:
        return (qtensor['data'].float() / 127) * qtensor['scale']
    return qtensor

def quantify_size_reduction(original_path, quantized_path):
    original_size = get_safetensors_size(original_path)
    quantized_size = os.path.getsize(quantized_path)
    reduction = (1 - quantized_size / original_size) * 100 if original_size > 0 else 0
    return original_size, quantized_size, reduction

def quantize_model_components(model_path: str, output_path: str, method: str = "fp16"):
    print("\n" + "=" * 60)
    print("Stable Audio Open - 模型量化工具")
    print("=" * 60)
    print(f"源模型: {model_path}")
    print(f"输出路径: {output_path}")
    print(f"量化方法: {method}")
    print()
    
    os.makedirs(output_path, exist_ok=True)
    
    components = [
        ("transformer", "diffusion_pytorch_model.safetensors", 4.03),
        ("vae", "diffusion_pytorch_model.safetensors", 0.60),
        ("text_encoder", "model.safetensors", 0.42),
        ("projection_model", "diffusion_pytorch_model.safetensors", 0.0015),
    ]
    
    total_original = 0
    total_quantized = 0
    
    for subfolder, filename, original_gb in components:
        src_dir = os.path.join(model_path, subfolder)
        src_file = os.path.join(src_dir, filename)
        dst_dir = os.path.join(output_path, subfolder)
        dst_file = os.path.join(dst_dir, filename)
        
        if not os.path.exists(src_file):
            print(f"跳过 {subfolder}/{filename} (不存在)")
            continue
        
        print(f"\n处理 {subfolder}/{filename}...")
        print(f"  原始大小: {original_gb:.2f} GB")
        
        os.makedirs(dst_dir, exist_ok=True)
        
        tensors = load_file(src_file)
        original_size = sum(t.nelement() * t.element_size() for t in tensors.values())
        total_original += original_size
        
        if method == "fp16":
            quantized_tensors = {k: v.to(torch.float16) for k, v in tensors.items()}
            save_file(quantized_tensors, dst_file)
            quantized_size = os.path.getsize(dst_file)
            
            reduction = (1 - quantized_size / original_size) * 100
            total_quantized += quantized_size
            
            print(f"  FP16 量化后: {format_size(quantized_size)} (减少 {reduction:.1f}%)")
            
        elif method == "int8":
            quantized_tensors = {}
            scales = {}
            
            for key, tensor in tensors.items():
                if tensor.dtype in [torch.float32, torch.float16] and tensor.ndim >= 2:
                    scale = tensor.abs().max()
                    if scale > 0:
                        quantized = (tensor / scale * 127).round().to(torch.int8)
                        quantized_tensors[f"{key}.data"] = quantized
                        scales[f"{key}.scale"] = torch.tensor([scale], dtype=torch.float32)
                    else:
                        quantized_tensors[key] = tensor.to(torch.int8)
                        scales[key] = torch.tensor([1.0], dtype=torch.float32)
                else:
                    quantized_tensors[key] = tensor.to(torch.int8)
                    scales[key] = torch.tensor([1.0], dtype=torch.float32)
            
            quantized_tensors.update(scales)
            save_file(quantized_tensors, dst_file)
            quantized_size = os.path.getsize(dst_file)
            
            reduction = (1 - quantized_size / original_size) * 100
            total_quantized += quantized_size
            
            print(f"  INT8 量化后: {format_size(quantized_size)} (减少 {reduction:.1f}%)")
    
    for item in ["tokenizer", "scheduler"]:
        src = os.path.join(model_path, item)
        if os.path.exists(src):
            dst = os.path.join(output_path, item)
            if not os.path.exists(dst):
                shutil.copytree(src, dst)
                print(f"  复制 {item}/ (配置必需)")
    
    for item in ["model_index.json", "config.json"]:
        src = os.path.join(model_path, item)
        if os.path.exists(src):
            shutil.copy2(src, os.path.join(output_path, item))
    
    for subfolder in ["text_encoder", "transformer", "vae", "projection_model"]:
        src_config = os.path.join(model_path, subfolder, "config.json")
        dst_dir = os.path.join(output_path, subfolder)
        if os.path.exists(src_config) and os.path.exists(dst_dir):
            shutil.copy2(src_config, os.path.join(dst_dir, "config.json"))
    
    print("\n" + "-" * 60)
    print(f"量化完成!")
    print(f"原始总大小: {format_size(total_original)}")
    print(f"量化后大小: {format_size(total_quantized)}")
    print(f"节省空间: {(1 - total_quantized / total_original) * 100:.1f}%")
    print(f"输出目录: {output_path}")
    
    config = {
        "model_path": output_path,
        "quantization": method,
        "original_size": format_size(total_original),
        "quantized_size": format_size(total_quantized),
        "reduction_percent": round((1 - total_quantized / total_original) * 100, 1)
    }
    
    config_file = os.path.join(output_path, "quantization_config.json")
    with open(config_file, 'w', encoding='utf-8') as f:
        json.dump(config, f, indent=2, ensure_ascii=False)
    print(f"\n配置文件: {config_file}")
    
    return True

def main():
    parser = argparse.ArgumentParser(description="Stable Audio Open - 模型量化工具")
    parser.add_argument("--model_path", type=str, default=None, help="源模型路径")
    parser.add_argument("--output", type=str, default=None, help="量化模型输出路径")
    parser.add_argument("--method", type=str, default="fp16", choices=["fp16", "int8"], help="量化方法: fp16 (推荐) 或 int8")
    parser.add_argument("--compare", action="store_true", help="比较不同量化方法的大小")
    args = parser.parse_args()
    
    model_path = get_model_path(args.model_path)
    
    if not os.path.exists(model_path):
        print(f"错误: 模型路径不存在: {model_path}")
        return 1
    
    if args.compare:
        print("\n比较不同量化方法...")
        
        print("\n" + "=" * 60)
        print("方法1: FP16 量化 (推荐)")
        print("=" * 60)
        fp16_output = model_path + "_fp16"
        if not os.path.exists(fp16_output):
            quantize_model_components(model_path, fp16_output, "fp16")
        else:
            print(f"FP16 量化已完成: {fp16_output}")
        
        return 0
    
    if args.output:
        output_path = args.output
    else:
        suffix = "_fp16" if args.method == "fp16" else "_int8"
        output_path = model_path + suffix
    
    quantize_model_components(model_path, output_path, args.method)
    
    print("\n" + "=" * 60)
    print("下一步")
    print("=" * 60)
    print(f"使用量化模型运行推理:")
    print(f"  python stable_audio_simple_final.py --prompt \"rain sound\" --output test.wav --model_path \"{output_path}\"")
    
    return 0

if __name__ == "__main__":
    sys.exit(main())
