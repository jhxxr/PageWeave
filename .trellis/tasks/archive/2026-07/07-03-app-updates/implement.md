# Add Update Checks and Silent Updates - Implementation Plan

## Checklist

1. Add updater dependencies.
   - Add npm packages for Tauri updater/process plugins if required by the frontend API.
   - Add Rust dependencies `tauri-plugin-updater` and `tauri-plugin-process`.

2. Configure Tauri updater.
   - Add updater plugin registration in `src-tauri/src/lib.rs`.
   - Add updater config to `src-tauri/tauri.conf.json` using the GitHub Releases `latest.json` endpoint.
   - Add a public key placeholder or real public key once generated.
   - Add updater/process permissions to `src-tauri/capabilities/default.json`.

3. Add update frontend service/state.
   - Read app version through `@tauri-apps/api/app`.
   - Wrap `check`, download, install, and relaunch behavior in a focused service or hook.
   - Support background check/download without automatic install/restart.
   - Preserve a pending downloaded update during the current app session.

4. Update Settings/About UI.
   - Replace hard-coded `0.1.0` with runtime version.
   - Add manual "check updates" button.
   - Show update status and install/restart action when an update is ready.
   - Add English and Chinese i18n keys for new UI text.

5. Update release workflow.
   - Configure `tauri-apps/tauri-action` or Tauri CLI so the release uploads updater metadata and signed artifacts.
   - Wire `TAURI_SIGNING_PRIVATE_KEY` and optional `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` from GitHub Secrets.
   - Keep optional offline assets upload unchanged.

6. Document signing setup.
   - Add concise updater signing instructions to README or release docs.
   - Include the local key generation command and GitHub Secrets names.

7. Validate.
   - Run TypeScript build.
   - Run Rust compile/check if dependencies resolve.
   - Inspect generated config/workflow for no private key leakage.

## Validation Commands

```powershell
pnpm build
pnpm tauri build
```

If full `pnpm tauri build` is too slow or blocked by signing/sidecar availability during local development:

```powershell
Set-Location src-tauri
cargo check
```

## Risky Files

- `src-tauri/tauri.conf.json`: invalid updater config or placeholder public key can break builds.
- `.github/workflows/release.yml`: updater metadata/signing changes affect release publishing.
- `src/features/settings/SettingsPage.tsx`: existing page is dense and has hard-coded version text.
- locale files: existing Chinese strings appear mojibake; keep changes scoped.

## Key Setup Instructions To Provide User

Preferred owner-side setup:

```powershell
pnpm tauri signer generate -w updater.key
```

Then:

- Copy the printed public key into updater config.
- Add the generated private key content to GitHub repository secret `TAURI_SIGNING_PRIVATE_KEY`.
- If a password is used, add it to `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`.
- Delete or securely store the local private key file; never commit it.

Confirm the exact command against the installed Tauri CLI during implementation before finalizing docs.

## Review Gate Before Start

Before running `task.py start`, confirm with the user:

- Background check/download with explicit install/restart is the intended UX.
- User will provide or generate the public key needed for committed config.
- CI may require repository secrets before release artifacts are updater-compatible.
