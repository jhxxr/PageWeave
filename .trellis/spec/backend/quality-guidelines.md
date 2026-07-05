# Quality Guidelines

> Code quality standards for backend development.

---

## Overview

<!--
Document your project's quality standards here.

Questions to answer:
- What patterns are forbidden?
- What linting rules do you enforce?
- What are your testing requirements?
- What code review standards apply?
-->

(To be filled by the team)

---

## Forbidden Patterns

<!-- Patterns that should never be used and why -->

(To be filled by the team)

---

## Required Patterns

<!-- Patterns that must always be used -->

(To be filled by the team)

---

## Testing Requirements

<!-- What level of testing is expected -->

(To be filled by the team)

---

## Code Review Checklist

<!-- What reviewers should check -->

(To be filled by the team)

---

## Scenario: Bundled BabelDOC Sidecar User-Agent Patch

### 1. Scope / Trigger

- Trigger: The bundled Python sidecar uses BabelDOC's OpenAI-compatible translator through the `openai` Python package.
- Problem: Some OpenAI-compatible gateways block the package default `User-Agent` value, which starts with `OpenAI/Python`.
- Scope: PyInstaller sidecar startup only. Rust command signatures, Tauri events, SQLite schemas, and frontend payloads do not change.

### 2. Signatures

- Runtime hook file: `sidecar/sitecustomize.py`
- PyInstaller spec:
  - `pathex` must include `str(ROOT / "sidecar")`
  - `hiddenimports` must include `"sitecustomize"`
  - `runtime_hooks` must include `str(ROOT / "sidecar" / "sitecustomize.py")`
- Patched client classes: `openai.OpenAI`, `openai.AsyncOpenAI`, `openai.AzureOpenAI`, `openai.AsyncAzureOpenAI`

### 3. Contracts

- The hook must set `default_headers["User-Agent"]` to `PageWeave/0.1` when the caller did not explicitly provide a `User-Agent`.
- The hook must preserve caller-provided `default_headers` and must not overwrite an explicit caller `User-Agent`.
- The hook must no-op if `openai` is unavailable or its client shape changes, so sidecar startup does not fail because of the patch.

### 4. Validation & Error Matrix

- `openai` import fails -> ignore and continue sidecar startup.
- Client class is missing -> skip that class and continue.
- Gateway blocks default UA -> rebuilt sidecar must send `PageWeave/0.1` and receive the gateway's normal response.
- Runtime hook is omitted from the spec -> BabelDOC may emit `Your request was blocked` from compatible gateways even when curl succeeds.

### 5. Good/Base/Bad Cases

- Good: `api.xinr.de` + `mimo-v2.5` translates a real PDF with `blocked` count equal to `0`.
- Base: Official OpenAI-compatible providers continue to receive normal OpenAI-compatible requests, only with the neutral PageWeave UA.
- Bad: Removing the runtime hook or `sidecar/` from `pathex` lets the frozen sidecar fall back to `OpenAI/Python <ver>`.

### 6. Tests Required

- `sidecar/dist/babeldoc-sidecar/babeldoc-sidecar.exe --version` exits successfully.
- An OpenAI Python client request through the frozen sidecar path uses `User-Agent: PageWeave/0.1`.
- End-to-end PDF translation against a gateway known to block `OpenAI/Python <ver>` produces mono and dual PDFs and reports zero blocked requests.
- Project checks still pass: Rust `cargo check`/unit tests and frontend `tsc --noEmit`/Vite build.

### 7. Wrong vs Correct

#### Wrong

```python
# Only works for curl/manual tests; BabelDOC still uses OpenAI/Python in process.
headers = {"User-Agent": "PageWeave/0.1"}
```

#### Correct

```python
# Loaded by PyInstaller before BabelDOC creates OpenAI clients.
runtime_hooks=[str(ROOT / "sidecar" / "sitecustomize.py")]
```

---

## Scenario: Required Offline BabelDOC Assets

### 1. Scope / Trigger

- Trigger: BabelDOC needs large model/font assets for first-run PDF translation, but `offline_assets_*.zip` is larger than GitHub's normal 100MB Git blob limit.
- Scope: Release packaging, Rust Tauri commands, frontend settings controls, startup readiness state, and BabelDOC cache restoration.
- Rule: Offline assets are required before translation starts. They must never be committed to Git; publish them as GitHub Release attachments.

### 2. Signatures

- Release asset name: `offline_assets_*.zip`
- Repository source for online install: `https://api.github.com/repos/jhxxr/PageWeave/releases/latest`
- Rust commands:
  - `get_offline_assets_info() -> OfflineAssetsInfo`
  - `install_offline_assets_from_release(app) -> OfflineAssetsInstallResult`
  - `install_offline_assets_from_file(app, path: String) -> OfflineAssetsInstallResult`
  - `start_translate(app, req) -> AppResult<String>`
- Frontend API wrappers:
  - `translateApi.offlineAssetsInfo()`
  - `translateApi.installOfflineAssetsFromRelease()`
  - `translateApi.installOfflineAssetsFromFile(path)`

### 3. Contracts

- `OfflineAssetsInfo` fields:
  - `installed: boolean`
  - `cache_dir: string`
  - `size_bytes: number`
  - `message: string`
- `OfflineAssetsInstallResult` fields:
  - `ok: boolean`
  - `cache_dir: string`
  - `asset_name?: string`
  - `message: string`
- Online install must discover the newest Release asset whose name starts with `offline_assets_` and ends with `.zip`.
- Local install must accept only a selected `offline_assets_*.zip` file, then pass its parent directory to BabelDOC `--restore-offline-assets`.
- App startup and Settings refresh must use `get_offline_assets_info()` as the UI readiness source. Do not use Python package or PATH probes for the home-page readiness banner.
- `start_translate` must reject requests when `OfflineAssetsInfo.installed == false`.
- Restoration must use the bundled sidecar only. Do not fall back to `babeldoc` on PATH or `python -m babeldoc`.
- Install commands must short-circuit successfully when the BabelDOC cache is already ready; a user should not see a restore failure while `get_offline_assets_info()` reports `installed=true`.
- A PyInstaller one-folder sidecar is usable only when the sidecar executable and its sibling `_internal/` runtime directory are both present. `resolve_sidecar()` must not return a bare copied/renamed exe that lacks `_internal/python*.dll`.
- Packaged builds must include both `bundle.externalBin` for the sidecar exe and `bundle.resources` for the full `sidecar/dist/babeldoc-sidecar/` directory.
- Runtime cache path detection must match BabelDOC's default cache shape: `$BABELDOC_CACHE_DIR`, `$XDG_CACHE_HOME/babeldoc`, or `<home>/.cache/babeldoc`.

### 4. Validation & Error Matrix

- Latest GitHub Release has no matching asset -> return `not_found`.
- Selected local file does not exist -> return `not_found`.
- Selected local file is not named `offline_assets_*.zip` -> return `invalid_input`.
- Translation starts while cache is not ready -> return `invalid_input` with an install-offline-assets message.
- Cache is already ready before install starts -> return `ok=true` without downloading or restoring.
- Sidecar exe exists without `_internal/python*.dll` -> ignore it and keep searching bundled/resource sidecar locations only.
- No usable bundled sidecar can restore assets -> return `not_found` with a reinstall-PageWeave message.
- Cache size remains below the ready threshold after restore -> return `ok=false` with the cache path for user diagnosis.

### 5. Good/Base/Bad Cases

- Good: User clicks Settings -> Install online; PageWeave downloads the Release asset, restores it, and Settings shows installed cache size.
- Base: User downloads the Release zip manually and chooses it from Settings; PageWeave restores from the local file without needing GitHub access.
- Bad: Home page reports "Python/BabelDOC not installed" while Settings reports offline assets are installed.
- Bad: `sidecar/assets/offline_assets_*.zip` is committed to Git. GitHub rejects push with GH001 or bloats repository history.

### 6. Tests Required

- `cargo check` validates Rust command signatures and error propagation.
- `pnpm exec tsc --noEmit` validates TypeScript mirrors and settings-page API calls.
- `pnpm build` validates the settings UI bundle.
- `pnpm tauri build --no-bundle` validates Tauri config, including `bundle.resources`.
- With a cache larger than the ready threshold, app startup and Settings both mark translation readiness as installed.
- With a cache smaller than the ready threshold, `start_translate` rejects before spawning the sidecar.
- With a bare renamed sidecar exe but no `_internal/`, sidecar resolution skips it instead of surfacing `Failed to load Python DLL`.
- Manual release smoke test: after CI publishes a Release, confirm it contains one `offline_assets_*.zip` attachment.
- Manual app smoke test: install online from Settings, then refresh status and verify the cache path/size changes.

### 7. Wrong vs Correct

#### Wrong

```text
git add sidecar/assets/offline_assets_*.zip
```

```rust
if path.exists() {
    return Some(path);
}
```

```rust
Command::new("python").arg("-m").arg("babeldoc")
```

#### Correct

```yaml
- name: Upload required offline assets
  run: gh release upload "pageweave-v$env:PAGEWEAVE_VERSION" $asset.FullName --clobber
```

```rust
if path.is_file() && path.parent().unwrap().join("_internal").is_dir() {
    return Some(path);
}
```

```json
{
  "externalBin": ["../sidecar/dist/babeldoc-sidecar/babeldoc-sidecar"],
  "resources": {
    "../sidecar/dist/babeldoc-sidecar/": "babeldoc-sidecar/"
  }
}
```

---

## Scenario: Translate Progress Event Contract

### 1. Scope / Trigger

- Trigger: Rust emits `translate://progress` events and React switches on `payload.type`.
- Scope: `TranslateEvent` serialization in `src-tauri/src/translate/model.rs`, event emission in `runner.rs` / `commands.rs`, and frontend handling in `src/App.tsx`.
- Risk: Serde's default enum variant names are `Status`, `Progress`, and `Log`; the frontend contract expects lowercase `status`, `progress`, and `log`.

### 2. Signatures

- Rust event enum: `TranslateEvent`
- Serde shape: `#[serde(tag = "type", rename_all = "lowercase")]`
- Tauri event channel: `translate://progress`
- Frontend TypeScript union: `TranslateEvent` with `type: "log" | "progress" | "status"`

### 3. Contracts

- Every emitted translate event must include a lowercase `type` discriminator.
- `status` events update lifecycle state and optional `message` / `output_files`.
- `progress` events update `overall` and `stage`.
- `log` events append masked stdout/stderr lines.
- Frontend event handlers must consume the shared TypeScript union, not compare against Rust enum variant names.

### 4. Validation & Error Matrix

- Serialized event has `type: "Status"` -> frontend ignores it; UI appears stuck with no error.
- Serialized event has `type: "Progress"` -> progress bar never advances.
- Serialized event has `type: "Log"` -> live log stays empty.
- Event has lowercase type but wrong fields -> TypeScript mirror or runtime handling must be updated with the enum change.

### 5. Good/Base/Bad Cases

- Good: Start translation emits `status`, then stderr `log`, then `progress`, and the UI updates.
- Base: BabelDOC exits with an error; the final `status` event is lowercase and the UI shows the error message.
- Bad: Adding a new Rust event variant without updating the frontend union and serialization test.

### 6. Tests Required

- Rust unit test serializes `TranslateEvent::Status`, `Progress`, and `Log` and asserts lowercase `type` values.
- `pnpm build` validates the frontend union and switch handling.
- Manual smoke: start a translation and verify running state, log output, progress, and final status are visible.

### 7. Wrong vs Correct

#### Wrong

```rust
#[serde(tag = "type")]
pub enum TranslateEvent {
    Status { /* ... */ },
}
```

#### Correct

```rust
#[serde(tag = "type", rename_all = "lowercase")]
pub enum TranslateEvent {
    Status { /* ... */ },
}
```

---

## Scenario: Tauri Updater Release Signing

### 1. Scope / Trigger

- Trigger: PageWeave enables in-app application updates through Tauri's updater plugin.
- Problem: Tauri updater requires signed artifacts, exact plugin permissions, and release metadata. A wrong permission or missing signing env causes build or runtime update failures.
- Scope: Tauri config, capabilities, Rust plugin registration, frontend updater calls, and GitHub Actions release publishing.

### 2. Signatures

- Tauri plugins:
  - `tauri_plugin_updater::Builder::new().build()`
  - `tauri_plugin_process::init()`
- Frontend APIs:
  - `@tauri-apps/plugin-updater.check() -> Promise<Update | null>`
  - `Update.download(onEvent?) -> Promise<void>`
  - `Update.install() -> Promise<void>`
  - `@tauri-apps/plugin-process.relaunch() -> Promise<void>`
- Config keys:
  - `bundle.createUpdaterArtifacts: true`
  - `plugins.updater.pubkey`
  - `plugins.updater.endpoints`
  - `plugins.updater.windows.installMode`

### 3. Contracts

- `plugins.updater.pubkey` must contain the public key generated by `pnpm tauri signer generate`.
- The matching private key must never be committed. Store it in GitHub Actions secret `TAURI_SIGNING_PRIVATE_KEY`.
- If the private key has a password, store it in `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`.
- The updater endpoint for GitHub Releases is `https://github.com/jhxxr/PageWeave/releases/latest/download/latest.json`.
- Capabilities must include:
  - `updater:default`
  - `process:allow-restart`
- Use Windows `installMode: "passive"` for user-confirmed install/restart flows. Fully quiet installs are not reliable when elevation is needed.

### 4. Validation & Error Matrix

- Missing `bundle.createUpdaterArtifacts` -> release may lack updater metadata/signatures.
- Missing signing secret in CI -> release build or updater artifact signing fails.
- Wrong public/private key pair -> update download succeeds but signature verification rejects install.
- Missing `updater:default` -> frontend update check/download/install commands are denied.
- Using `process:allow-relaunch` -> Tauri capability build fails; the correct permission is `process:allow-restart`.
- Missing `latest.json` on the latest release -> background check should stay quiet; manual check should show an error.

### 5. Good/Base/Bad Cases

- Good: App checks GitHub Releases in the background, downloads an available signed update, then waits for the user to click install/restart.
- Base: No update is available; Settings reports the current version is up to date.
- Bad: App installs or restarts automatically while a translation task may still be running.

### 6. Tests Required

- `pnpm build` validates frontend updater imports, state, and Settings UI.
- `cargo check` validates Rust plugin dependencies and registration.
- `pnpm tauri build --no-bundle` validates Tauri config, capabilities, and release compile path.
- Secret scan should confirm no private key content is present in tracked diffs.
- Manual release smoke: after GitHub Actions publishes a release, verify it contains `latest.json` and signed updater artifacts.

### 7. Wrong vs Correct

#### Wrong

```json
{
  "permissions": ["updater:default", "process:allow-relaunch"]
}
```

```yaml
env:
  GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

#### Correct

```json
{
  "permissions": ["updater:default", "process:allow-restart"]
}
```

```yaml
env:
  GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
  TAURI_SIGNING_PRIVATE_KEY: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY }}
  TAURI_SIGNING_PRIVATE_KEY_PASSWORD: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY_PASSWORD }}
```

---

## Scenario: Cancellable Tauri Child Processes

### 1. Scope / Trigger

- Trigger: A Tauri command starts a long-running child process and exposes a separate cancel command.
- Problem: `tokio::process::Child` cannot be safely "shared" by storing it in a registry and then moving it into the runner for `wait()`. Once the runner takes ownership, the registry no longer has a live child to kill.
- Scope: Translation runner and any future backend task runner that spawns an external process.

### 2. Signatures

- Registry task shape:
  - `RunningTask { cancel_tx: mpsc::UnboundedSender<()>, status: String }`
- Start command:
  - `start_translate(app, req) -> AppResult<String>`
- Cancel command:
  - `cancel_translate(app, task_id) -> AppResult<bool>`

### 3. Contracts

- The runner owns the `Child` for its full lifetime.
- The registry stores a cancellation sender, not the child itself.
- Cancel sets task status to `cancelled`, sends one cancellation signal, and returns `true` when the task exists.
- The runner listens for either `child.wait()` or cancellation. On cancellation it calls `child.start_kill()`, then waits again to reap the process.
- Final status emission must not overwrite a cancelled task with a generic error just because the killed process exits non-zero.

### 4. Validation & Error Matrix

- `task_id` not found -> `cancel_translate` returns `false`.
- Cancellation signal receiver is closed -> ignore send failure; task is already finishing.
- Child exits normally before cancel -> emit `success` or `error` based on exit status.
- Cancel arrives before child exits -> call `start_kill()`, wait for exit, emit `cancelled`.
- Killed child exits non-zero -> still emit `cancelled`, not `error`.

### 5. Good/Base/Bad Cases

- Good: User clicks Cancel during BabelDOC translation; the runner kills BabelDOC, reaps it, removes the registry entry, and the UI stays cancelled.
- Base: BabelDOC finishes successfully without cancellation; the runner emits success and output files.
- Bad: Registry stores `Arc<Mutex<Option<Child>>>`, runner takes the child out for `wait()`, and later cancel sees `None` or blocks until the process has already ended.

### 6. Tests Required

- `cargo check` for command and registry signatures.
- Unit or integration test for future runners should cover: start process, cancel task, assert cancelled status is emitted, assert no later error status overwrites it.
- Manual smoke test for BabelDOC: start a long translation, cancel while running, verify the child process disappears and UI status remains cancelled.

### 7. Wrong vs Correct

#### Wrong

```rust
pub struct RunningTask {
    pub child: Arc<Mutex<Option<Child>>>,
}

let mut owned_child = child_slot.lock().await.take();
let status = owned_child.as_mut().unwrap().wait().await;
```

#### Correct

```rust
pub struct RunningTask {
    pub cancel_tx: mpsc::UnboundedSender<()>,
    pub status: String,
}

let status = tokio::select! {
    wait = child.wait() => wait,
    _ = cancel_rx.recv() => {
        let _ = child.start_kill();
        child.wait().await
    }
};
```

---

## Scenario: BabelDOC Output File Discovery

### 1. Scope / Trigger

- Trigger: Rust runner reports produced PDFs to the frontend after BabelDOC exits successfully.
- Problem: BabelDOC output naming is version-dependent. Current BabelDOC 0.6.3 emits names like `<stem>.<lang>.mono.pdf`, while older assumptions used `<stem>-mono.pdf`.
- Scope: `translate::runner::scan_outputs` and any future UI logic that relies on generated output paths.

### 2. Signatures

- Function: `scan_outputs(req: &TranslateRequest, task_id: &str) -> Vec<String>`
- Request fields used:
  - `req.pdf_paths[0]`
  - `req.output_dir`
  - `req.output_mode: OutputMode`

### 3. Contracts

- For `OutputMode::Mono`, report mono PDFs only.
- For `OutputMode::Dual`, report dual PDFs only.
- For `OutputMode::Both`, report both kinds when present.
- Recognize legacy names:
  - `<stem>-mono.pdf`
  - `<stem>-dual.pdf`
- Recognize current BabelDOC names:
  - `<stem>.<lang>.mono.pdf`
  - `<stem>.<lang>.dual.pdf`
  - `<stem>.<lang>.mono.no_watermark.pdf`
  - `<stem>.<lang>.dual.no_watermark.pdf`
- Sort returned paths for stable frontend rendering and tests.

### 4. Validation & Error Matrix

- Output directory is missing or unreadable -> return an empty list; translation success status may still show no output paths.
- No matching files -> return an empty list.
- Matching mono exists but mode is `Dual` -> exclude mono.
- Matching dual exists but mode is `Mono` -> exclude dual.
- Extra unrelated PDFs in output directory -> ignore them unless they match the input stem and requested kind.

### 5. Good/Base/Bad Cases

- Good: Real smoke with BabelDOC 0.6.3 produces `pageweave-smoke-valid.zh.mono.pdf`; the UI receives that path.
- Base: Older output `pageweave-smoke-valid-mono.pdf` is still recognized.
- Bad: Scanner only checks `format!("{stem}-mono.pdf")`, so successful translations appear to have no output files.

### 6. Tests Required

- Unit test for current BabelDOC `<stem>.<lang>.mono.pdf` / `<stem>.<lang>.dual.pdf`.
- Unit test for legacy `<stem>-mono.pdf` / `<stem>-dual.pdf`.
- Real smoke test after BabelDOC upgrades should verify the generated filename still matches scanner rules.

### 7. Wrong vs Correct

#### Wrong

```rust
if name == format!("{stem}-mono.pdf") {
    out.push(path);
}
```

#### Correct

```rust
if want_mono && is_babeldoc_output(&name, stem, "mono") {
    out.push(path);
}
```

---

## Scenario: Bundled Sidecar Multiprocessing Freeze Support

### 1. Scope / Trigger

- Trigger: The bundled `babeldoc-sidecar.exe` is built with PyInstaller and BabelDOC/PyMuPDF may spawn multiprocessing child processes while saving PDFs.
- Problem: Without `multiprocessing.freeze_support()` in the sidecar entry point, child processes can re-enter `babeldoc.main:cli` with PyInstaller's `--multiprocessing-fork ...` arguments. BabelDOC argparse then logs `unrecognized arguments`, and PDF save may emit a false failure before fallback succeeds.
- Scope: `sidecar/babeldoc_entry.py` and any future PyInstaller sidecar entry point.

### 2. Signatures

- Entry script: `sidecar/babeldoc_entry.py`
- Required call:
  - `multiprocessing.freeze_support()` before importing/running `babeldoc.main.cli`

### 3. Contracts

- Sidecar startup must call `freeze_support()` at the beginning of `main()`.
- PyInstaller multiprocessing child invocations must be handled before BabelDOC CLI argument parsing.
- The normal CLI path must preserve BabelDOC command semantics for Rust runner arguments.
- The generated sidecar bundle must still include PyInstaller's `pyi_rth_multiprocessing` runtime hook.

### 4. Validation & Error Matrix

- `babeldoc-sidecar.exe --version` -> exits successfully.
- Real translation with PDF save multiprocessing -> no `--multiprocessing-fork` in logs.
- Logs contain `unrecognized arguments: --multiprocessing-fork` -> entry point is missing or not reaching `freeze_support()`.
- Logs contain `PDF save with clean=False failed` but final output succeeds -> inspect multiprocessing startup before treating PDF generation as broken.

### 5. Good/Base/Bad Cases

- Good: Rebuilt sidecar translates a test PDF and logs `PDF save with clean=False completed successfully`.
- Base: Non-multiprocessing translation path still works exactly like the BabelDOC CLI.
- Bad: Entry imports and calls BabelDOC CLI directly; child process arguments fall into BabelDOC argparse and pollute the UI log.

### 6. Tests Required

- `python -m py_compile sidecar/babeldoc_entry.py`
- Rebuild sidecar after changing the entry point.
- `babeldoc-sidecar.exe --version` smoke test.
- Real sidecar translation smoke test and grep logs for absence of `multiprocessing-fork` and `unrecognized arguments`.

### 7. Wrong vs Correct

#### Wrong

```python
def main() -> int:
    from babeldoc.main import cli
    cli()
    return 0
```

#### Correct

```python
import multiprocessing

def main() -> int:
    multiprocessing.freeze_support()
    from babeldoc.main import cli
    cli()
    return 0
```
