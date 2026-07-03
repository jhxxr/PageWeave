# Implementation Plan

## Phase 0: Setup

- [x] Load `trellis-before-dev` before source edits.
- [x] Create temporary smoke-test PDF/output directory outside committed source.
- [x] Verify sidecar `--version`.

## Phase 1: Real Smoke

- [x] Test provider/model connectivity without committing secrets.
- [x] Run BabelDOC sidecar translation against the temporary PDF.
- [x] Inspect output files and logs.

## Phase 2: Fixes

- [x] Fix any strict-MVP blocker discovered by the smoke test.
- [x] Keep fixes narrow and rerun the smoke where relevant.

## Phase 3: Verification

- [x] `pnpm build`
- [x] `cargo check`
- [x] `cargo test`
- [x] Record whether real smoke succeeded or what blocked it.

## Smoke Result

- Provider connectivity succeeded with the user-provided endpoint/model.
- BabelDOC sidecar translated the temporary PDF successfully and produced `pageweave-smoke-valid.zh.mono.pdf`.
- Discovered blocker: Rust output scanner expected legacy `stem-mono.pdf` naming and would miss current BabelDOC `stem.zh.mono.pdf` outputs.
- Fix: `scan_outputs()` now recognizes current and legacy mono/dual naming, with regression tests.
- Discovered sidecar UX issue: PyInstaller multiprocessing child args leaked into BabelDOC argparse, logging `--multiprocessing-fork` errors even though translation eventually succeeded.
- Fix: `sidecar/babeldoc_entry.py` now calls `multiprocessing.freeze_support()` before invoking BabelDOC CLI.
- Rebuilt local sidecar dist for verification only; generated dist remains ignored and uncommitted.
- Follow-up smoke after rebuild succeeded and no longer logged `multiprocessing-fork`, `unrecognized arguments`, or `PDF save with clean=False failed`.
