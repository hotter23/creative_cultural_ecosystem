#!/usr/bin/env python3
"""
Stable Audio Open - 高性能版本
直接调用 pipeline 核心组件，避免不必要的封装开销
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
from diffusers import StableAudioPipeline

def setup_cuda():
    if torch.cuda.is_available():
        torch.backends.cudnn.benchmark = True
        torch.backends.cuda.matmul.allow_tf32 = True
        torch.backends.cudnn.allow_tf32 = True
        torch.cuda.empty_cache()

def main():
    parser = argparse.ArgumentParser(description="Stable Audio - 高性能版本")
    parser.add_argument("--prompt", type=str, required=True, help="生成提示词")
    parser.add_argument("--duration", type=float, default=10.0, help="音频时长（秒）")
    parser.add_argument("--output", type=str, required=True, help="输出文件路径")
    parser.add_argument("--num_steps", type=int, default=50, help="推理步数")
    parser.add_argument("--guidance_scale", type=float, default=3.5, help="引导强度")
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

        print(f"[Step 1] Loading pipeline...", file=sys.stderr)
        
        load_start = time.time()
        pipe = StableAudioPipeline.from_pretrained(
            model_path,
        )
        print(f"[Step 1] Pipeline loaded: {time.time() - load_start:.2f}s", file=sys.stderr)

        if use_cuda:
            pipe = pipe.to(device)
            
        inference_start = time.time()
        
        print(f"[Step 2] Starting inference ({args.num_steps} steps)...", file=sys.stderr)
        
        params = pipe.__call__.__code__.co_varnames
        kwargs = {
            "num_inference_steps": args.num_steps,
            "guidance_scale": args.guidance_scale,
        }
        
        if "audio_length_in_s" in params:
            kwargs["audio_length_in_s"] = args.duration
        elif "duration" in params:
            kwargs["duration"] = args.duration
        elif "num_frames" in params:
            kwargs["num_frames"] = int(args.duration * 25)
        elif "audio_end_in_s" in params:
            kwargs["audio_end_in_s"] = args.duration
        
        print(f"[Step 2] Using params: {list(kwargs.keys())}", file=sys.stderr)
        
        with torch.inference_mode():
            result = pipe(args.prompt, **kwargs)
            
            if use_cuda:
                torch.cuda.synchronize()
        
        inference_time = time.time()
        print(f"[Step 3] Inference done: {inference_time - inference_start:.2f}s", file=sys.stderr)

        audio = result.audios[0]
        audio_np = audio.detach().cpu().numpy()
        
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
            "steps": args.num_steps,
            "total_time": total_time
        }))

    except Exception as e:
        import traceback
        traceback.print_exc()
        print(json.dumps({"success": False, "error": str(e)}), file=sys.stderr)
        sys.exit(1)

if __name__ == "__main__":
    main()
