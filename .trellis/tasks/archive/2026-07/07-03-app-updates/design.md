# Add Update Checks and Silent Updates - Design

## Architecture

Use Tauri's official updater plugin as the update mechanism.

- Rust/Tauri layer:
  - Add `tauri-plugin-updater` and `tauri-plugin-process`.
  - Register both plugins in `src-tauri/src/lib.rs`.
  - Configure updater endpoints in `src-tauri/tauri.conf.json`.
  - Add updater permissions to `src-tauri/capabilities/default.json`.
- Frontend layer:
  - Add a small update service around `@tauri-apps/plugin-updater`.
  - Add runtime version display through `@tauri-apps/api/app`.
  - Add Settings/About controls for manual check, status, install, and restart.
  - Add app-start background check and download after settings load or app mount.
- Release layer:
  - Extend `.github/workflows/release.yml` to produce updater metadata and signed artifacts.
  - Document required GitHub Actions Secrets for signing.

## Data Flow

1. App starts.
2. Frontend schedules a background updater check.
3. The updater plugin requests the configured endpoint:
   `https://github.com/jhxxr/PageWeave/releases/latest/download/latest.json`.
4. If no update exists, the app records an up-to-date state and stays quiet.
5. If an update exists, the app downloads it in the background and keeps a pending update object in frontend runtime state.
6. Settings/About shows that an update is ready.
7. User clicks install/restart.
8. The updater plugin installs the downloaded update and the process plugin relaunches the app when appropriate.

## Contracts

### Update Status

Frontend state should distinguish:

- `idle`
- `checking`
- `upToDate`
- `available`
- `downloading`
- `readyToInstall`
- `installing`
- `error`

### User Interaction

- Automatic behavior may check and download.
- Automatic behavior must not install or restart without an explicit user action.
- Manual check from Settings should show useful feedback even if background check already ran.

### Release Metadata

The release must include updater-compatible metadata, normally `latest.json`, containing:

- version
- notes
- pub_date
- platform-specific URL and signature

The exact generated shape should be delegated to Tauri tooling rather than handwritten.

## Compatibility Notes

- Current release workflow is Windows-only. This task should keep updater support scoped to Windows release output.
- Tauri updater signatures are mandatory; the public key is committed in app config, while the private key remains in GitHub Actions Secrets.
- The existing optional `offline_assets_*.zip` release asset is not an app update artifact and remains handled by `src-tauri/src/translate/assets.rs`.
- The Settings page currently has mojibake in Chinese locale strings. This task may add new localized strings, but should avoid broad unrelated locale rewrites unless required to keep UI comprehensible.

## Signing Key Setup

Repository owner flow:

1. Generate a Tauri updater key pair locally.
2. Commit only the public key in `src-tauri/tauri.conf.json`.
3. Store the private key as `TAURI_SIGNING_PRIVATE_KEY` in GitHub repository secrets.
4. If the key was generated with a password, store it as `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`.
5. Do not commit the private key or password to the repository.

## Trade-offs

- Tauri updater is stricter than a custom GitHub release checker because it requires signing, but it gives a safer install path and avoids custom installer execution logic.
- Fully silent install/restart would reduce friction, but it is not chosen because Windows elevation and active translation jobs make it risky. Background check/download plus explicit install/restart is the safer MVP.

## Rollback

If updater integration causes release problems:

- Remove updater and process plugins from Tauri builder.
- Remove updater config and capability permissions.
- Revert release workflow updater metadata/signing changes.
- Keep manual GitHub Releases downloads as the fallback distribution path.
