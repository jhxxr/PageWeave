# Complete PageWeave MVP Design

## Architecture And Boundaries

The task hardens the existing Tauri desktop architecture without changing the main stack:

- React/TypeScript frontend owns UX state, validation messages, navigation, and Tauri command calls.
- Rust/Tauri backend owns local persistence, keyring access, sidecar probing, BabelDOC process management, and filesystem/runtime integration.
- BabelDOC remains an external sidecar/CLI process. This task does not embed Python or switch to BabelDOC internal APIs.
- SQLite/keyring contracts remain unchanged unless a small compatibility fix is required.

The implementation should stay close to the current modules:

- `src/features/translate/`, `src/stores/translateStore.ts`, and `src/types/index.ts` for the one-PDF workflow.
- `src/features/tasks/TasksPage.tsx` for current-task visibility only.
- `src/features/settings/SettingsPage.tsx`, `src/i18n/locales/*`, and `README.md` for runtime/distribution messaging.
- `src-tauri/src/translate/*` for process lifecycle, output discovery, error messages, and sidecar probing.
- `src-tauri/src/provider/*` only for issues discovered in provider readiness verification.

## Data Flow

1. App startup loads settings/providers, probes BabelDOC/sidecar, and subscribes to `translate://progress`.
2. User selects or drops PDF files. The strict MVP exposes only one runnable PDF. If multiple files are dropped, the UI must either keep only the active file or clearly require the user to select one.
3. Start validates frontend-visible inputs first: provider, API key, model, output dir, PDF, and BabelDOC status.
4. `start_translate` validates backend invariants, resolves the key from keyring, probes sidecar/BabelDOC, builds CLI args, spawns BabelDOC, and emits status/log/progress events.
5. Frontend store updates status, progress, stage, logs, status message, and output files from events.
6. Success scans output paths and exposes open/reveal actions. Cancel kills the child process and produces a stable cancelled state.

## Contracts

### TranslateRequest

Keep the existing shape for compatibility, but the UI must send exactly one PDF for this MVP:

```ts
{
  pdf_paths: string[];
  output_dir: string;
  lang_in: string;
  lang_out: string;
  output_mode: "mono" | "dual" | "both";
  provider: { base_url: string; api_key_id: string; model: string };
  qps: number;
}
```

Backend should reject empty input and may reject `pdf_paths.length > 1` with a clear one-PDF MVP message, rather than silently using the first file.

### TranslateEvent

Retain the current event protocol:

- `log`: append masked output line.
- `progress`: update best-known percentage and stage.
- `status`: update lifecycle state, message, and output files.

## UX Decisions

- The Translate page remains the primary screen. It should feel like a working tool, not a setup wizard.
- Cards can remain because the existing app already uses Ant Design cards heavily; changes should avoid large visual rewrites.
- Params page should state that strict MVP controls live on Translate and should not imply advanced controls are usable.
- Tasks page should show the current active/recent translation clearly, including error and output paths. Persistent history is deferred.
- Settings/About text must describe bundled sidecar/offline assets as the preferred distribution story. Python install copy should only be fallback/development language.

## Compatibility And Migration

- No database migration is required for the strict MVP.
- `task_record` remains reserved for later persistent history.
- Existing providers/settings should continue to load unchanged.
- Existing exported provider JSON remains keyless and compatible.

## Risks And Trade-Offs

- The app currently accepts multiple PDFs in state while backend uses only the first. This is the highest UX correctness risk; resolving ambiguity is required.
- API key via `--openai-api-key` remains a known MVP security debt. This task documents and preserves it unless a small safer switch is trivial and compatible.
- Progress parsing depends on BabelDOC stderr formatting. The MVP treats progress as best-effort and logs as authoritative fallback.
- Real translation smoke testing may depend on local sidecar/assets/provider keys. If unavailable, verification must document the exact missing external prerequisite.

## Rollback Shape

Most changes are frontend UX/documentation hardening. Backend changes should be small and reversible:

- If backend multi-PDF rejection causes compatibility problems, revert to frontend-only enforcement.
- If output discovery changes regress, restore previous `scan_outputs` and keep improved UI messaging.
- If docs/i18n changes are too broad, revert copy-only changes independently from runtime fixes.
