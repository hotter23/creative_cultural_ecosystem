# -*- mode: python ; coding: utf-8 -*-
import os
import sys
from PyInstaller.utils.hooks import collect_all, collect_submodules

block_cipher = None

script_dir = os.path.dirname(os.path.abspath(SPEC))
venv_dir = os.path.join(script_dir, ".venv")
venv_site_packages = os.path.join(venv_dir, "Lib", "site-packages")

if not os.path.exists(venv_site_packages):
    print(f"警告: 虚拟环境 site-packages 不存在: {venv_site_packages}")

# 使用 collect_all 收集所有包（包含数据文件）
datas = []
binaries = []
hiddenimports = []

packages_to_collect = [
    "torch",
    "torchsde",
    "transformers",
    "diffusers",
    "numpy",
    "scipy",
    "safetensors",
    "PIL",
    "soundfile",
    "tokenizers",
    "huggingface_hub",
    "accelerate",
    "hf_xet",
]

for pkg in packages_to_collect:
    try:
        print(f"收集 {pkg}...")
        pkg_datas, pkg_binaries, pkg_hiddenimports = collect_all(pkg)
        datas.extend(pkg_datas)
        binaries.extend(pkg_binaries)
        hiddenimports.extend(pkg_hiddenimports)
        print(f"  ✓ {pkg}: {len(pkg_hiddenimports)} 个导入")
    except Exception as e:
        print(f"  ✗ {pkg} 失败: {e}")
        try:
            submodules = collect_submodules(pkg)
            hiddenimports.extend(submodules)
        except:
            pass

hiddenimports = list(set(hiddenimports))

a = Analysis(
    ["stable_audio_simple_final.py"],
    pathex=[
        venv_site_packages,
    ],
    binaries=binaries,
    datas=datas,
    hiddenimports=hiddenimports,
    hookspath=[],
    hooksconfig={},
    runtime_hooks=[],
    excludes=[
        "tkinter",
        "matplotlib",
        "cv2",
        "torchvision",
        "torchaudio",
        "tensorboard",
        "torch.utils.tensorboard",
    ],
    win_no_prefer_redirects=False,
    win_private_assemblies=False,
    cipher=block_cipher,
    noarchive=False,
)

pyz = PYZ(a.pure, a.zipped_data, cipher=block_cipher)

exe = EXE(
    pyz,
    a.scripts,
    a.binaries,
    a.zipfiles,
    a.datas,
    [],
    name="stable_audio_inference",
    debug=False,
    bootloader_ignore_signals=False,
    strip=False,
    upx=False,
    upx_exclude=[],
    runtime_tmpdir=None,
    console=True,
    disable_windowed_traceback=False,
    argv_emulation=False,
    target_arch=None,
    codesign_identity=None,
    entitlements_file=None,
)
