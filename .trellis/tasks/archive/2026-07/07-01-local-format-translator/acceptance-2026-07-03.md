# End-to-End Acceptance Note - 2026-07-03

## Result

Real PDF translation succeeded through the bundled BabelDOC sidecar using `api.xinr.de` and `mimo-v2.5`.

## Evidence

- Input: `fncom-18-1431815.pdf`
- Pages: 10
- Input size: 375 KB
- Output: `*.zh.mono.pdf` and `*.zh.dual.pdf`
- Output pages: 10 each
- Output sizes: about 9.6 MB and 9.5 MB
- Blocked request count: 0
- Verified Chinese text includes: `基于脑电图的自适应闭环脑机接口在神经康复中的应用：综述`

## Root Cause

The gateway blocks the default `openai` Python package HTTP `User-Agent` (`OpenAI/Python <ver>`). Curl worked because it did not send that UA, while BabelDOC requests made through the `openai` package were blocked.

## Fix Captured

- Added `sidecar/sitecustomize.py` to patch OpenAI Python client default headers at sidecar startup.
- Updated `sidecar/babeldoc_sidecar.spec` so PyInstaller includes `sidecar/` in `pathex`, imports `sitecustomize`, and runs it as a runtime hook.
- Rebuilt the sidecar and verified the patched client sends `User-Agent: PageWeave/0.1`.

## Verification Summary

- Sidecar starts and `--version` works.
- UA patch is active.
- Real PDF translation produces Chinese mono and dual PDFs.
- Rust `cargo check` and unit tests passed.
- Frontend `tsc --noEmit` and Vite build passed.
- Offline model/font assets are present.
