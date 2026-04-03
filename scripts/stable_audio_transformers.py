#!/usr/bin/env python3
"""
Stable Audio Open - Transformers API 版本
直接使用 model.generate() 方法，绕过 diffusers pipeline 开销
"""
import sys
import os

os.environ['HF_ENDPOINT'] = 'https://hf-mirror.com'

import argparse
import json
import warnings
warnings.filterwarnings("ignore")

import numpy as np
import scipy.io.wavfile as wavfile
import torch
import time

def setup_cuda():
    if torch.cuda.is_available():
        torch.backends.cudnn.benchmark = True
        torch.backends.cuda.matmul.allow_tf32 = True
        torch.backends.cudnn.allow_tf32 = True
        torch.cuda.empty_cache()

def main():
    parser = argparse.ArgumentParser(description="Stable Audio - Transformers API")
    parser.add_argument("--prompt", type=str, required=True, help="生成提示词")
    parser.add_argument("--duration", type=float, default=10.0, help="音频时长（秒）")
    parser.add_argument("--output", type=str, required=True, help="输出文件路径")
    parser.add_argument("--guidance_scale", type=float, default=3.5, help="引导强度")
    parser.add_argument("--num_steps", type=int, default=50, help="推理步数")
    parser.add_argument("--cpu_only", action="store_true", help="强制使用 CPU")
    parser.add_argument("--model_path", type=str, default=None, help="本地模型路径")
    args = parser.parse_args()

    output_dir = os.path.dirname(args.output)
    if output_dir and not os.path.exists(output_dir):
        os.makedirs(output_dir, exist_ok=True)

    try:
        start_time = time.time()
        use_cuda = not args.cpu_only and torch.cuda.is_available()
        device = "cuda" if use_cuda else "cpu"

        setup_cuda()

        model_path = args.model_path
        if model_path is None:
            if getattr(sys, 'frozen', False):
                app_dir = os.path.dirname(sys.executable)
            else:
                app_dir = os.path.dirname(os.path.abspath(__file__))
            model_path = os.path.join(app_dir, "model", "f21265c1e2710b3bd2386596943f0007f55f802e_fp16")
            if not os.path.exists(model_path):
                model_path = os.path.join(app_dir, "model", "f21265c1e2710b3bd2386596943f0007f55f802e_fp16")

        print(f"[Step 1] Loading model from {model_path}...", file=sys.stderr)
        
        load_start = time.time()
        from transformers import AutoModelForTextToAudio, AutoProcessor
        
        processor = AutoProcessor.from_pretrained(model_path)
        print(f"[Step 1a] Processor loaded: {time.time() - load_start:.2f}s", file=sys.stderr)
        
        model_load_start = time.time()
        model = AutoModelForTextToAudio.from_pretrained(
            model_path,
            torch_dtype=torch.float16 if use_cuda else torch.float32,
            low_cpu_mem_usage=True
        )
        print(f"[Step 1b] Model loaded: {time.time() - model_load_start:.2f}s", file=sys.stderr)
        
        if use_cuda:
            model = model.to(device)
            print("[Optimize] Model moved to GPU", file=sys.stderr)

        sample_rate = processor.sampling_rate
        max_new_tokens = int(args.duration * sample_rate / processor.tokens_per_second)
        print(f"[Step 2] max_new_tokens = {max_new_tokens} (duration={args.duration}s, sr={sample_rate})", file=sys.stderr)

        inference_start = time.time()
        
        inputs = processor(
            text=args.prompt,
            padding="max_length",
            max_length=512,
            return_tensors="pt"
        ).to(device, torch.float16 if use_cuda else torch.float32)
        
        print(f"[Step 2b] Text encoded: {time.time() - inference_start:.2f}s", file=sys.stderr)

        print(f"[Step 3] Starting generation...", file=sys.stderr)
        gen_start = time.time()
        
        with torch.no_grad():
            audio_values = model.generate(
                **inputs,
                max_new_tokens=max_new_tokens,
                num_inference_steps=args.num_steps,
                guidance_scale=args.guidance_scale
            )
            
        print(f"[Step 3] Generation done: {time.time() - gen_start:.2f}s", file=sys.stderr)
        
        if use_cuda:
            torch.cuda.synchronize()

        inference_time = time.time()
        print(f"[Total Inference Time] {inference_time - inference_start:.2f}s", file=sys.stderr)

        audio_np = audio_values.squeeze().cpu().numpy()
        
        if audio_np.ndim == 2:
            audio_np = audio_np.mean(axis=0)
        
        audio_np = audio_np.astype(np.float32)
        max_val = np.abs(audio_np).max()
        if max_val > 0:
            audio_np = audio_np / max_val
        audio_int16 = (audio_np * 32767).astype(np.int16)
        
        target_samples = int(args.duration * 24000)
        if len(audio_int16) > target_samples:
            audio_int16 = audio_int16[:target_samples]
        
        wavfile.write(args.output, 24000, audio_int16)
        print(f"[Step 4] WAV saved: {time.time() - inference_time:.2f}s", file=sys.stderr)

        total_time = time.time() - start_time
        print(f"[Total Time] {total_time:.2f}s", file=sys.stderr)

        print(json.dumps({
            "success": True,
            "output_path": args.output,
            "duration": len(audio_int16) / 24000,
            "sample_rate": 24000,
            "device": device,
            "total_time": total_time
        }))

    except Exception as e:
        import traceback
        traceback.print_exc()
        print(json.dumps({"success": False, "error": str(e)}), file=sys.stderr)
        sys.exit(1)

if __name__ == "__main__":
    main()
