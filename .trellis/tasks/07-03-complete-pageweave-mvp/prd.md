# Complete PageWeave MVP

## Goal

Turn PageWeave from a mostly MVP-shaped prototype into a complete, shippable Windows local PDF translation MVP: a user can configure an OpenAI-compatible provider, select or drop PDFs, translate through the bundled BabelDOC sidecar, observe progress/logs, open outputs, and recover from common setup/runtime errors without reading source code.

## User Value

The app should feel usable as a personal desktop tool, not just a demo. The priority is a reliable end-to-end PDF translation workflow with local-only API key storage and clear operational guidance when BabelDOC, offline assets, provider settings, or output paths are wrong.

## Confirmed Facts

- Original scope in `guide.md` defines the MVP as Windows desktop shell, PDF selection/drag-drop, AI API config, BabelDOC CLI translation, live logs/progress, output PDF, open output folder, local config storage, and basic errors (`guide.md:188`, `guide.md:192`).
- The previous archived MVP task intentionally excluded batch queues, task history persistence, advanced parameters, OCR deep optimization, and full multilingual UI (`.trellis/tasks/archive/2026-07/07-01-local-format-translator/prd.md:66`).
- A real PDF translation has already succeeded through the bundled BabelDOC sidecar (`.trellis/tasks/archive/2026-07/07-01-local-format-translator/acceptance-2026-07-03.md:5`).
- The current app already builds with `pnpm build`.
- Packaging direction has advanced beyond the first MVP: README and Tauri config now describe bundling `babeldoc-sidecar` through `externalBin`, with offline assets support (`README.md:32`, `README.md:40`, `README.md:48`, `src-tauri/tauri.conf.json:31`).
- Provider CRUD, provider export without API keys, provider connection testing, and model fetching are implemented as Tauri commands and UI flows (`src-tauri/src/provider/commands.rs:198`, `src/features/provider/ProviderPage.tsx:189`).
- Settings persist theme, language, default output directory, default languages, default provider, log retention, and cache directory (`src-tauri/src/settings/commands.rs:10`).
- Translation starts/cancels through Tauri commands, streams progress/log events, and scans output files (`src-tauri/src/lib.rs:57`, `src-tauri/src/translate/runner.rs:384`).
- The current translation model accepts multiple PDF paths but intentionally translates only the first PDF (`src-tauri/src/translate/model.rs:16`).
- A `task_record` table exists but there are no task-history commands or persistent history UI wired to it (`src-tauri/src/db/schema.rs:6`, `src/features/tasks/TasksPage.tsx:14`).
- Params and Tasks pages still present MVP placeholder copy, with advanced params and history deferred (`src/i18n/locales/en.ts:112`, `src/i18n/locales/en.ts:117`).
- API keys are still passed to BabelDOC via `--openai-api-key`, which is documented as known MVP debt because process command lines can be visible to other local processes (`src-tauri/src/translate/args.rs:4`).

## Requirements

### Scope Decision

This task targets the strict shippable MVP: one active PDF translation workflow that is clear, stable, and package-ready. True batch translation and persistent task history are deferred to a post-MVP task.

### R1 End-to-End Translation Hardening

- The Translate page must make the one-PDF MVP scope obvious and prevent unsupported batch ambiguity.
- Starting a translation must validate selected provider, API key presence, model, output directory, PDF list, and BabelDOC/sidecar availability before spawning work.
- Status, progress, logs, success outputs, cancellation, and retry after failure must remain coherent across the whole run.
- Output discovery must reliably show every produced PDF for the selected output mode.

### R2 Provider Configuration Readiness

- Provider creation/editing/deletion/default selection must be usable without exposing API keys in normal list/export flows.
- Test connection and fetch models must give actionable success/failure messages.
- Provider export without API keys must remain functional. Provider import is deferred unless discovered to be necessary for the strict MVP.

### R3 Local Runtime And Packaging Readiness

- Sidecar detection and missing-runtime messaging must align with the current bundled sidecar direction, not older "install Python + pip install BabelDOC" copy when a packaged app should be dependency-free.
- Offline assets install/restore state must be visible enough that first-run failures are understandable.
- README and in-app About/Settings text must match the actual distribution story.

### R4 Task Visibility

- The Tasks page must no longer feel like a dead placeholder.
- Minimum acceptable behavior is a clear current-task summary with status, file, progress, outputs, and logs/errors.
- Persistent history is deferred. The existing `task_record` table should remain unused unless a narrow implementation detail requires it.

### R5 Settings And Defaults

- Settings must reliably feed Translate defaults on startup: output directory, languages, default provider, and language/theme.
- Changing defaults must persist and not desynchronize from provider default selection.

### R6 Documentation And Verification

- The project must have a documented "happy path" for development, sidecar setup, build, and packaged use.
- Verification must include at least frontend build/type check, Rust check/test where available, and one smoke path for translation or a documented reason it cannot run locally.

## Acceptance Criteria

- [ ] AC1 `pnpm build` passes.
- [ ] AC2 Rust backend check/test passes or any failing external dependency is documented with exact command/output.
- [ ] AC3 A user can add/edit/delete a provider, set default provider, test connection, and fetch models from the UI.
- [ ] AC4 API keys are stored through keyring and never included in provider export.
- [ ] AC5 Translate page clearly restricts the MVP run to one selected PDF and prevents unsupported batch ambiguity.
- [ ] AC6 A configured provider plus valid PDF/output directory can start a BabelDOC translation and stream live logs/progress.
- [ ] AC7 Success state shows produced output file(s) and supports opening/revealing them.
- [ ] AC8 Cancel stops the active BabelDOC subprocess and produces a clear cancelled state.
- [ ] AC9 Missing sidecar/BabelDOC/offline asset/provider/key/output-dir cases show actionable user-facing errors.
- [ ] AC10 Settings defaults are applied on app startup and persist after restart.
- [ ] AC11 Params and Tasks pages are implemented to the strict MVP scope or de-emphasized so the app does not advertise dead features.
- [ ] AC12 README and in-app dependency/license notes match the actual bundled-sidecar distribution model.

## Out Of Scope Candidates

These remain out of scope unless the user chooses a broader "complete product" target instead of "complete MVP":

- DOCX/PPTX translation.
- User accounts, cloud sync, payment, plugin system, auto-update.
- Full advanced parameter surface for glossary/context/cache/OCR tuning.
- Deep OCR quality optimization beyond current BabelDOC defaults.
- True batch translation and persistent task history.
- Multi-provider load balancing.
- Cross-platform guarantees beyond Windows.
