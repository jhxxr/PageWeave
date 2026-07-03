# Add update checks and silent updates

## Goal

Add application update support for PageWeave using the `jhxxr/PageWeave` GitHub Releases channel so users can detect newer builds from inside the app and receive updates with minimal interruption.

## Confirmed Facts

- PageWeave is a Tauri 2 desktop app with React, TypeScript, Ant Design, Zustand, and i18next.
- Release assets are published by `.github/workflows/release.yml` to GitHub Releases under tags like `pageweave-v__VERSION__`.
- The release workflow currently updates `package.json`, `src-tauri/tauri.conf.json`, and `src-tauri/Cargo.toml` to `BASE_VERSION.github.run_number` during CI.
- `src-tauri/src/translate/assets.rs` already uses `https://api.github.com/repos/jhxxr/PageWeave/releases/latest` for optional offline assets.
- The app currently shows a hard-coded version `0.1.0` in `src/features/settings/SettingsPage.tsx`.
- `src-tauri/tauri.conf.json` has no updater configuration and `src-tauri/Cargo.toml` has no `tauri-plugin-updater` dependency.
- `src-tauri/capabilities/default.json` currently grants core, opener, dialog, and fs permissions, but not updater permissions.
- Official Tauri v2 updater docs say update signatures are required and cannot be disabled. The app must provide a public key in config and CI must sign update artifacts with the private key.
- The updater signing private key must stay out of git and should be stored by the repository owner in GitHub Actions Secrets.
- Official Tauri v2 updater docs support a static GitHub Releases JSON endpoint such as `https://github.com/user/repo/releases/latest/download/latest.json`.
- On Windows, Tauri updater supports `passive`, `basicUi`, and `quiet` install modes. `quiet` has limitations because it cannot request admin privileges by itself; `passive` is the generally recommended default.

## Requirements

- Use the GitHub Releases channel from `jhxxr/PageWeave` as the update source.
- Add a user-visible update check in the app, preferably in Settings/About where version information already lives.
- Show the installed app version from runtime metadata instead of a hard-coded frontend string.
- Support an update flow that can download and install updates with minimal user interaction.
- Background behavior: automatically check for and download available updates in the background; require an explicit user action for install/restart.
- Signing key handling: document how the repository owner generates the updater key pair, stores `TAURI_SIGNING_PRIVATE_KEY` and optional `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` in GitHub Actions Secrets, and provides only the public key for committed app configuration.
- Keep update signing explicit in configuration and release automation.
- Keep optional BabelDOC offline asset release download behavior separate from app binary updates.
- Surface useful user feedback for update states: checking, no update, update available, downloading/installing, installed/restart needed, and failure.
- Do not block PDF translation workflows with update checks unless installation requires restart.

## Acceptance Criteria

- [x] The Settings/About UI can check for a newer PageWeave release.
- [x] When no update exists, the UI reports that the current version is up to date.
- [x] When an update exists, the UI can trigger download/install and reports progress or status.
- [x] The installed version displayed in the UI comes from the app runtime/package metadata, not a duplicated literal.
- [x] Tauri updater plugin is configured for signed update artifacts and GitHub Releases `latest.json`.
- [x] Release workflow generates or uploads updater artifacts and metadata compatible with Tauri updater.
- [x] Windows install mode is explicitly chosen and documented.
- [x] Build/type-check validation passes.

## Validation

- `pnpm build` passed.
- `cargo check` passed.
- `pnpm tauri build --no-bundle` passed.
- Secret scan confirmed no updater private key content is present in tracked diffs.

## Out of Scope

- Building a custom update server.
- Auto-updating optional BabelDOC offline assets as part of app binary updates.
- Cross-platform release expansion beyond the current Windows release workflow.
- Forced updates that interrupt a running translation task without user-visible state.

## Open Questions

- None blocking.
