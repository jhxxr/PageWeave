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
