# Design

## Approach

This task is smoke-test driven. Use a temporary PDF and the user-provided OpenAI-compatible provider to exercise the real BabelDOC path first. Then make narrow fixes based on observed failures.

## Secret Handling

- Do not write the API key into task files, source files, README, shell scripts, committed fixtures, or final response.
- Prefer transient environment variables or local app storage/keyring for tests.
- If a command must pass the key to BabelDOC, treat that as a local one-off smoke command and do not preserve the command text in docs.

## Expected Data Flow

1. Temporary PDF -> BabelDOC sidecar.
2. BabelDOC sidecar -> OpenAI-compatible endpoint/model.
3. Output directory receives mono/dual PDF files.
4. Logs are checked for blocked requests, auth failures, model errors, output path mismatches, or missing offline assets.

## Fix Policy

- Fix code only when the smoke test exposes a strict-MVP blocker.
- Keep changes scoped to translate/provider/settings/runtime paths.
- If a failure is environmental, document the blocker in `implement.md` instead of coding around it blindly.
