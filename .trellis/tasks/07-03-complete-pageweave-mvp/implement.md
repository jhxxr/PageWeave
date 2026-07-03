# Complete PageWeave MVP Implementation Plan

## Phase 0: Pre-Dev Context

- [x] Load `trellis-before-dev` before editing source files.
- [x] Read relevant frontend/backend spec docs listed by the skill.
- [x] Re-check `git status --short` and avoid touching unrelated user changes.

## Phase 1: Translation Workflow Hardening

- [x] Audit `TranslatePage.tsx`, `translateStore.ts`, `App.tsx`, `src-tauri/src/translate/*` for state/event mismatches.
- [x] Enforce strict one-PDF MVP in UI and backend validation.
- [x] Improve start validation for provider/key/model/output directory/BabelDOC state.
- [x] Fix status transitions for start failure, cancel, success, and retry.
- [x] Confirm output discovery matches mono/dual/both output modes.

Validation:

- [x] `pnpm build`
- [x] `cargo check` from `src-tauri`

## Phase 2: Current Task Visibility

- [x] Replace placeholder-feeling Tasks page with a useful current-task summary.
- [x] Show active/recent file, status, progress, output files, and error/status message.
- [x] Keep persistent history out of scope.

Validation:

- [x] `pnpm build`
- [ ] Manual UI smoke via dev server or Tauri if available. Blocked for real translation by missing local sample PDF and API key.

## Phase 3: Runtime, Settings, And Copy Alignment

- [x] Align BabelDOC missing-runtime copy with bundled sidecar plus fallback development install.
- [x] Align Settings/About dependency text and README with current sidecar/offline-assets distribution.
- [x] Ensure settings defaults still apply to Translate on startup.
- [x] Remove or soften dead-feature wording on Params/Tasks.

Validation:

- [x] `pnpm build`
- [x] Review English/Chinese i18n keys for missing or stale strings.

## Phase 4: Provider Readiness Check

- [x] Verify provider CRUD/default/test/fetch/export flows against acceptance criteria.
- [x] Fix only strict-MVP blockers.
- [x] Keep provider import deferred unless a blocker appears.

Validation:

- [x] `cargo check`
- [ ] Manual provider UI smoke if Tauri runtime is available. Not run; requires interactive Tauri runtime and a provider key.

## Phase 5: Final Quality Gate

- [x] Run `pnpm build`.
- [x] Run `cargo check` in `src-tauri`.
- [x] Run available Rust tests, especially translate progress tests.
- [x] If real translation smoke cannot run, document exact blocker.
- [ ] Update PRD acceptance criteria status if appropriate.
- [x] Run `trellis-check`.

## Risky Files And Rollback Points

- `src/features/translate/TranslatePage.tsx`: primary UX behavior. Roll back in small hunks if validation becomes too restrictive.
- `src-tauri/src/translate/commands.rs` and `runner.rs`: process lifecycle. Keep changes minimal and test with `cargo check`.
- `src/i18n/locales/*.ts`: copy-only but high visibility. Avoid unrelated translation rewrites.
- `README.md`: documentation only; keep factual and aligned with existing sidecar scripts.
