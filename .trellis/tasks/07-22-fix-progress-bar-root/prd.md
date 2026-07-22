# PRD: Fix translate progress bar root cause

## Problem

Translation UI progress bar stays stuck, while "latest activity" / logs flicker because babeldoc rich progress redraws are treated as high-frequency log events.

## Root causes

1. `--report-interval 0.1` emits progress ~10×/s.
2. Stage-local counters (e.g. `12/40`) were mapped to overall %, fighting real overall (`translate 42/100`) under monotonic commits.
3. Progress-bar CR redraws were emitted as `Log` events → store + UI thrash.

## Goals

- Overall % comes only from overall-task signals (`translate` / total=100 / explicit `%`).
- Stage labels still update without rewriting overall %.
- Bar redraws do not flood the log stream or activity strip.
- Report interval is calmer (0.5s) without losing usable progress.

## Acceptance criteria

- [x] Progress bar advances during a real translation when babeldoc reports overall progress. (parser: overall only from translate / total=100 / %)
- [x] Stage field updates on stage changes; overall never jumps backward mid-run.
- [x] Activity/log UI does not flicker from bar redraw noise. (bar noise not emitted as Log; report-interval 0.5)
- [x] `cargo test -p pageweave --lib translate::` — 38 passed.
