# Quality Guidelines

> Code quality standards for frontend development.

---

## Overview

Frontend follows feature-sliced modules under `src/features/*`, Zustand stores, and thin `services/api.ts` invoke wrappers. Shared openers and settings are reused across features.

---

## Forbidden Patterns

- Do not listen to `translate://progress` for convert UI state (or the reverse). Each engine has its own channel.
- Do not put convert logic into `TranslatePage` / `translateStore`; convert is a peel-off feature under `features/convert` + `convertStore`.
- Do not call provider/API-key APIs from the convert page.
- Do not invent a second open/reveal path; reuse `translateApi.openFilePath` / `revealFilePath` (generic file openers).

---

## Required Patterns

- New desktop engines: independent route + store + `*Api` namespace + event channel prefix.
- File-type whitelists that gate UX must mirror Rust constants (e.g. `CONVERT_ALLOWED_EXTENSIONS` ↔ `convert::args::ALLOWED_EXTENSIONS`).
- Event unions must use lowercase `type` discriminators matching Rust `rename_all = "lowercase"`.
- While a convert task is `running`, ignore drag-drop replacements of the input file.
- On start, allow the first `convert://progress` status event even if local `taskId` is still null (race with invoke return).

---

## Scenario: Convert Page ↔ markitdown Commands

### 1. Scope / Trigger

- Trigger: Frontend must drive the peel-off markitdown convert module without coupling to PDF translation.
- Scope: `src/features/convert/ConvertPage.tsx`, `src/stores/convertStore.ts`, `convertApi`, `ConvertEvent` / `ConvertRequest` types, App menu/route/listener, i18n.

### 2. Signatures

- `convertApi.start(req: ConvertRequest): Promise<string>` → invoke `start_convert` with `{ req }`
- `convertApi.cancel(task_id: string): Promise<boolean>` → invoke `cancel_convert` with `{ taskId: task_id }`
- `convertApi.markitdownInfo(): Promise<MarkitdownInfo>` → invoke `get_markitdown_info`
- Event channel: `convert://progress`
- Types: `ConvertRequest`, `ConvertEvent`, `MarkitdownInfo`, `CONVERT_ALLOWED_EXTENSIONS`

### 3. Contracts

- Route: `/convert`; sider entry always visible (no feature flag).
- Single-file picker only (`multiple: false`); filters limited to whitelist extensions.
- Success UI: show `output_file` path + open file / open folder; **no** in-app Markdown preview.
- No provider/model/API key fields on the convert page.
- i18n: zh + en keys for menu label, page copy, errors, and “LLM-oriented Markdown, not print fidelity” disclaimer.
- Default output directory seeds from `settings.default_output_dir`.

### 4. Validation & Error Matrix

- Unsupported extension selected/dropped -> surface error / disable start; do not invoke backend.
- Backend busy error -> show message; keep previous successful output if any.
- Missing sidecar (`installed: false`) -> show `hint` from `get_markitdown_info`.
- Cancel click while running -> `convertApi.cancel(taskId)`.

### 5. Good/Base/Bad Cases

- Good: Drop a `.pdf`, start, see logs/status, open produced `.md`.
- Base: Drop while running is ignored; input path unchanged.
- Bad: Wiring convert progress into `useTranslateStore` or sharing `taskId` with translation.

### 6. Tests Required

- `pnpm exec tsc --noEmit`
- Manual: menu → convert page; invalid extension; start/cancel; openers; language switch zh/en

### 7. Wrong vs Correct

#### Wrong

```ts
listen<ConvertEvent>("translate://progress", handler);
```

#### Correct

```ts
listen<ConvertEvent>("convert://progress", handler);
// cancel payload uses Tauri camelCase rename:
convertApi.cancel(taskId); // { taskId }
```

---

## Testing Requirements

- Typecheck after any invoke name or event shape change.
- Keep zh/en keys in lockstep for new user-visible strings.

---

## Code Review Checklist

- [ ] Feature lives under `src/features/<name>/` with matching store/api types
- [ ] Event channel prefix matches the engine module
- [ ] No secret/provider dependency on offline-only features
- [ ] Peel-off features do not import sibling feature internals
