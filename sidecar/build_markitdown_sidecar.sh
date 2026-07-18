#!/usr/bin/env bash
# Build the bundled markitdown sidecar for Windows.
#
# Produces sidecar/dist/markitdown-sidecar/markitdown-sidecar.exe (one-folder)
# and a Tauri externalBin triple-suffixed copy:
#   markitdown-sidecar-x86_64-pc-windows-msvc.exe
#
# Prereqs: a venv with markitdown extras installed:
#   pip install 'markitdown[pdf,docx,pptx,xlsx,xls]==0.1.6' pyinstaller
#
# Usage:
#   bash sidecar/build_markitdown_sidecar.sh [/path/to/venv/python]
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PYTHON="${1:-/tmp/bd_venv/Scripts/python.exe}"

cd "$ROOT"

echo "==> Using Python: $PYTHON"
"$PYTHON" -m PyInstaller --version >/dev/null 2>&1 || {
  echo "PyInstaller not installed in this venv; installing..."
  "$PYTHON" -m pip install pyinstaller
}

echo "==> Ensuring markitdown extras are installed..."
"$PYTHON" -c "import markitdown" >/dev/null 2>&1 || {
  echo "Installing markitdown[pdf,docx,pptx,xlsx,xls]==0.1.6..."
  "$PYTHON" -m pip install 'markitdown[pdf,docx,pptx,xlsx,xls]==0.1.6'
}

echo "==> Running PyInstaller..."
"$PYTHON" -m PyInstaller sidecar/markitdown_sidecar.spec --noconfirm \
  --distpath sidecar/dist --workpath sidecar/build/markitdown

EXE="sidecar/dist/markitdown-sidecar/markitdown-sidecar.exe"
if [ ! -f "$EXE" ]; then
  echo "ERROR: expected exe not found at $EXE" >&2
  exit 1
fi

# Tauri externalBin expects <name>-<target-triple>.exe
TRIPLE="x86_64-pc-windows-msvc"
SUFFIXED="sidecar/dist/markitdown-sidecar/markitdown-sidecar-${TRIPLE}.exe"
cp "$EXE" "$SUFFIXED"
echo "==> Sidecar ready: $SUFFIXED"

echo "==> Smoke test: markitdown-sidecar --version"
"$SUFFIXED" --version || echo "warn: --version returned non-zero"

echo "==> Done."
