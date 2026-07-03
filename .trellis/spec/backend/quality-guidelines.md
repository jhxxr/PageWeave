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

## Scenario: Optional Offline BabelDOC Assets

### 1. Scope / Trigger

- Trigger: BabelDOC needs large model/font assets for first-run PDF translation, but `offline_assets_*.zip` is larger than GitHub's normal 100MB Git blob limit.
- Scope: Release packaging, Rust Tauri commands, frontend settings controls, and BabelDOC cache restoration.
- Rule: Offline assets are optional runtime packages. They must never be committed to Git; publish them as GitHub Release attachments.

### 2. Signatures

- Release asset name: `offline_assets_*.zip`
- Repository source for online install: `https://api.github.com/repos/jhxxr/PageWeave/releases/latest`
- Rust commands:
  - `get_offline_assets_info() -> OfflineAssetsInfo`
  - `install_offline_assets_from_release(app) -> OfflineAssetsInstallResult`
  - `install_offline_assets_from_file(path: String) -> OfflineAssetsInstallResult`
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
- Restoration must use the bundled sidecar first, then `babeldoc` on PATH, then `python -m babeldoc`.
- Install commands must short-circuit successfully when the BabelDOC cache is already ready; a user should not see a restore failure while `get_offline_assets_info()` reports `installed=true`.
- A PyInstaller one-folder sidecar is usable only when the sidecar executable and its sibling `_internal/` runtime directory are both present. `resolve_sidecar()` must not return a bare copied/renamed exe that lacks `_internal/python*.dll`.
- Runtime cache path detection must match BabelDOC's default cache shape: `$BABELDOC_CACHE_DIR`, `$XDG_CACHE_HOME/babeldoc`, or `<home>/.cache/babeldoc`.

### 4. Validation & Error Matrix

- Latest GitHub Release has no matching asset -> return `not_found`.
- Selected local file does not exist -> return `not_found`.
- Selected local file is not named `offline_assets_*.zip` -> return `invalid_input`.
- Cache is already ready before install starts -> return `ok=true` without downloading or restoring.
- Sidecar exe exists without `_internal/python*.dll` -> ignore it and try `babeldoc` on PATH / `python -m babeldoc`.
- No sidecar/PATH BabelDOC/Python module can restore assets -> return `translate` or `io` error with captured process output.
- Cache size remains below the ready threshold after restore -> return `ok=false` with the cache path for user diagnosis.

### 5. Good/Base/Bad Cases

- Good: User clicks Settings -> Install online; PageWeave downloads the Release asset, restores it, and Settings shows installed cache size.
- Base: User downloads the Release zip manually and chooses it from Settings; PageWeave restores from the local file without needing GitHub access.
- Bad: `sidecar/assets/offline_assets_*.zip` is committed to Git. GitHub rejects push with GH001 or bloats repository history.

### 6. Tests Required

- `cargo check` validates Rust command signatures and error propagation.
- `pnpm exec tsc --noEmit` validates TypeScript mirrors and settings-page API calls.
- `pnpm build` validates the settings UI bundle.
- With a cache larger than the ready threshold, clicking either install path returns success and does not invoke a sidecar.
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

#### Correct

```yaml
- name: Upload optional offline assets
  run: gh release upload "pageweave-v$env:PAGEWEAVE_VERSION" $asset.FullName --clobber
```

```rust
if path.is_file() && path.parent().unwrap().join("_internal").is_dir() {
    return Some(path);
}
```
