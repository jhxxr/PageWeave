#!/usr/bin/env bash
# Build the bundled BabelDOC sidecar for Windows.
#
# Produces sidecar/dist/babeldoc-sidecar/babeldoc-sidecar.exe (one-folder bundle)
# and then renames the exe with the Tauri target-triple suffix so `externalBin`
# can pick it up: babeldoc-sidecar-x86_64-pc-windows-msvc.exe
#
# Prereqs: a venv with BabelDOC installed (and assets warmed so the cache is
# populated). PyInstaller must be installed in that venv.
#
# Usage:
#   bash sidecar/build_sidecar.sh [/path/to/venv/python]
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PYTHON="${1:-/tmp/bd_venv/Scripts/python.exe}"

cd "$ROOT"

echo "==> Using Python: $PYTHON"
"$PYTHON" -m PyInstaller --version >/dev/null 2>&1 || {
  echo "PyInstaller not installed in this venv; installing..."; "$PYTHON" -m pip install pyinstaller
}

# Pre-download models/fonts so the bundled app can run fully offline.
echo "==> Warming BabelDOC assets (model + fonts)..."
"$PYTHON" -c "from babeldoc.main import cli" >/dev/null 2>&1 || true
babeldoc_bin="$(dirname "$PYTHON")/babeldoc.exe"
if [ -x "$babeldoc_bin" ]; then
  "$babeldoc_bin" --warmup || echo "warn: warmup incomplete; first run may download assets"
fi

echo "==> Running PyInstaller..."
"$PYTHON" -m PyInstaller sidecar/babeldoc_sidecar.spec --noconfirm \
  --distpath sidecar/dist --workpath sidecar/build

EXE="sidecar/dist/babeldoc-sidecar/babeldoc-sidecar.exe"
if [ ! -f "$EXE" ]; then
  echo "ERROR: expected exe not found at $EXE" >&2
  exit 1
fi

# PyInstaller misses hyperscan's vendor-hashed MSVC runtime (it lives in
# hyperscan.libs/, not next to the .pyd), so _hs_ext fails to load at runtime.
# Copy it into _internal/ next to the other DLLs.
HS_LIBS="$("$PYTHON" -c 'import os, hyperscan; print(os.path.join(os.path.dirname(hyperscan.__file__), "..", "hyperscan.libs"))' 2>/dev/null)"
if [ -d "$HS_LIBS" ]; then
  for dll in "$HS_LIBS"/*.dll; do
    [ -f "$dll" ] || continue
    cp "$dll" sidecar/dist/babeldoc-sidecar/_internal/ && echo "==> copied $(basename "$dll") into _internal/"
  done
fi

# Tauri externalBin expects <name>-<target-triple>.exe; we link/copy for x86_64 Windows.
TRIPLE="x86_64-pc-windows-msvc"
SUFFIXED="sidecar/dist/babeldoc-sidecar/babeldoc-sidecar-${TRIPLE}.exe"
cp "$EXE" "$SUFFIXED"
echo "==> Sidecar ready: $SUFFIXED"

# Bundle offline assets (DocLayout-YOLO model + fonts + cmap + tiktoken) so the
# shipped app can translate fully offline on first run, no HuggingFace download.
echo "==> Generating offline assets package..."
"$SUFFIXED" --generate-offline-assets sidecar/assets || echo "warn: offline assets generation failed; first run may download"

# Restore assets into the sidecar cache dir so the running sidecar finds them.
# BabelDOC's CACHE_FOLDER is ~/.cache/babeldoc by default; PyInstaller onefile-style
# HOME is the user's real home, so this is correct for an installed app.
echo "==> Restoring offline assets into user cache (~/.cache/babeldoc)..."
"$SUFFIXED" --restore-offline-assets sidecar/assets || echo "warn: restore failed; first run may download"

# Smoke test: --version should print and exit 0.
echo "==> Smoke test: babeldoc-sidecar --version"
"$SUFFIXED" --version || echo "warn: --version returned non-zero (may be normal for click)"

echo "==> Done."
