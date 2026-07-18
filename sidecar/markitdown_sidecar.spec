# -*- mode: python ; coding: utf-8 -*-
"""PyInstaller spec for the bundled markitdown sidecar (Windows, x86_64).

Produces a one-folder bundle named `markitdown-sidecar` (same shape as the
babeldoc sidecar: start fast, keep a stable `_internal/` next to the exe).

Build with:
    pyinstaller sidecar/markitdown_sidecar.spec --noconfirm \
        --distpath sidecar/dist --workpath sidecar/build/markitdown
"""

from pathlib import Path

from PyInstaller.utils.hooks import collect_all

ROOT = Path(SPECPATH).resolve().parent  # sidecar/; project root is its parent.

datas = []
bins = []
hidden = []

# Core package + optional extras for PDF/Office formats.
for pkg in [
    "markitdown",
    "pdfminer",
    "pdfminer.six",
    "pypdf",
    "pymupdf",
    "fitz",
    "docx",
    "pptx",
    "openpyxl",
    "xlrd",
    "olefile",
    "magika",
    "beautifulsoup4",
    "bs4",
    "lxml",
    "html5lib",
    "charset_normalizer",
    "chardet",
    "defusedxml",
    "mammoth",
    "striprtf",
    "onnxruntime",  # magika may pull this; collect if present
]:
    try:
        d, b, h = collect_all(pkg)
        datas += d
        bins += b
        hidden += h
    except Exception:
        pass

hiddenimports_extra = [
    "markitdown",
    "markitdown.__main__",
    "markitdown.converters",
    "pdfminer",
    "pdfminer.high_level",
    "pypdf",
    "docx",
    "pptx",
    "openpyxl",
    "xlrd",
    "olefile",
    "magika",
    "bs4",
    "lxml",
    "defusedxml",
]

a = Analysis(
    [str(ROOT / "sidecar" / "markitdown_entry.py")],
    pathex=[str(ROOT / "sidecar"), str(ROOT)],
    binaries=bins,
    datas=datas,
    hiddenimports=hidden + hiddenimports_extra,
    hookspath=[],
    hooksconfig={},
    runtime_hooks=[],
    excludes=[
        "matplotlib",
        "IPython",
        "pytest",
        "tornado",
        "notebook",
        "jupyter",
        "torch",
        "tensorflow",
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
    name="markitdown-sidecar",
    debug=False,
    bootloader_ignore_signals=False,
    strip=False,
    upx=False,
    console=True,  # need stdout/stderr pipes for the Rust runner
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
    name="markitdown-sidecar",
)
