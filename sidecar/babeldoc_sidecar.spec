# -*- mode: python ; coding: utf-8 -*-
"""PyInstaller spec for the bundled BabelDOC sidecar (Windows, x86_64).

Produces a one-folder bundle named `babeldoc-sidecar` (preferred over onefile
because onefile re-extracts to a temp dir on every launch, which is slow for a
~200MB bundle and breaks BabelDOC's cache-dir assumptions).

Build with:
    pyinstaller sidecar/babeldoc_sidecar.spec --noconfirm \
        --distpath sidecar/dist --workpath sidecar/build

The Rust `externalBin` target name must match the produced exe: `babeldoc-sidecar`
(Tauri appends the target-triple suffix at bundle time, e.g. `babeldoc-sidecar-x86_64-pc-windows-msvc.exe`).
"""

from pathlib import Path

# PyInstaller runtime hook helpers: pull in everything babeldoc ships
# (165 submodules + 155 data files) plus native binaries for its heavy deps.
from PyInstaller.utils.hooks import collect_all

ROOT = Path(SPECPATH).resolve().parent  # spec lives in sidecar/; project root is its parent.

# Collect babeldoc's full package tree (binaries + datas + hidden imports) in one shot.
_bd_datas, _bd_bins, _bd_hidden = collect_all("babeldoc")

babeldoc_datas = list(_bd_datas)
babeldoc_hidden = list(_bd_hidden)
babeldoc_bins = list(_bd_bins)

# Also pull native binaries for the heavy deps so the frozen exe can find them.
for pkg in [
    "onnxruntime",
    "cv2",
    "pymupdf",
    "fitz",
    "scipy",
    "skimage",
    "sklearn",
    "rtree",
    "freetype",
    "uharfbuzz",
    "pyzstd",
    "Levenshtein",
    "rapidfuzz",
    "bitarray",
    "bitstring",
    "tiktoken",
    "tiktoken_ext",
    "regex",
    "charset_normalizer",
    "chardet",
    "httpx",
    "socksio",
    "openai",
    "msgpack",
    "tenacity",
    "peewee",
    "psutil",
    "xsdata",
    "orjson",
    "pydantic",
    "click",
    "rich",
    "tqdm",
    "configargparse",
    "numpy",
    "PIL",
    "huggingface_hub",
    "httpcore",
    "anyio",
    "certifi",
    "cffi",
    "pycparser",
]:
    try:
        d, b, h = collect_all(pkg)
        babeldoc_datas += d
        babeldoc_bins += b
        babeldoc_hidden += h
    except Exception:
        pass

# Explicit safety net: modules that are sometimes imported dynamically.
hiddenimports_extra = [
    "tiktoken_ext.openai_public",
    "tiktoken_ext.model_info",
    "regex",
    "chardet",
    "socksio",
    "msgpack",
    "click",
    "rich",
    "tqdm",
    "configargparse",
    # sitecustomize patches openai's UA so gateways that filter on the
    # default `OpenAI/Python` UA don't block BabelDOC's requests.
    "sitecustomize",
]

a = Analysis(
    [str(ROOT / "sidecar" / "babeldoc_entry.py")],
    pathex=[str(ROOT / "sidecar"), str(ROOT)],
    binaries=babeldoc_bins,
    datas=babeldoc_datas,
    hiddenimports=babeldoc_hidden + hiddenimports_extra,
    hookspath=[],
    hooksconfig={},
    runtime_hooks=[str(ROOT / "sidecar" / "sitecustomize.py")],
    excludes=[
        "matplotlib",
        "IPython",
        "pytest",
        "tornado",
        "notebook",
        "jupyter",
    ],
    win_no_prefer_redirects=False,
    win_private_assemblies=False,
    cipher=None,
    noarchive=False,
)

pyz = PYZ(a.pure, a.zipped_data)

exe = EXE(
    pyz,
    a.scripts,
    [],
    exclude_binaries=True,
    name="babeldoc-sidecar",
    debug=False,
    bootloader_ignore_signals=False,
    strip=False,
    upx=False,
    console=True,  # sidecar: we need stdout/stderr pipes
    disable_windowed_traceback=False,
    target_arch=None,
    codesign_identity=None,
    entitlements_file=None,
)

coll = COLLECT(
    exe,
    a.binaries,
    a.zipfiles,
    a.datas,
    strip=False,
    upx=False,
    upx_exclude=[],
    name="babeldoc-sidecar",
)
