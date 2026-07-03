# Polish PageWeave MVP With Real API Smoke

## Goal

Continue completing PageWeave beyond the strict MVP hardening by validating the actual translation path with a user-provided OpenAI-compatible provider and fixing any blockers or sharp edges discovered during real use.

## Confirmed Facts

- Previous strict MVP hardening is complete and archived.
- User provided a working OpenAI-compatible endpoint, API key, and model ID for validation. The API key must not be committed, logged in docs, or repeated in assistant responses.
- Bundled BabelDOC sidecar exists locally under `sidecar/dist/babeldoc-sidecar/` and previously passed `--version`.
- Real translation smoke was not run in the prior task because no sample PDF/API key was available.

## Scope

- Generate or locate a small temporary test PDF outside committed source.
- Run provider connectivity and/or real BabelDOC translation smoke using the provided endpoint/model.
- Inspect logs and outputs for blockers.
- Fix strict-MVP blockers discovered by the smoke test.
- Keep secrets out of Git, Trellis task artifacts, README, and final response.
- Re-run `pnpm build`, `cargo check`, and `cargo test` after code changes.

## Acceptance Criteria

- [ ] A small PDF translation smoke test is attempted with the provided provider/model.
- [ ] API key is not committed or printed in final summaries.
- [ ] Any discovered blocker in the strict MVP path is fixed or documented with exact reproduction.
- [ ] `pnpm build` passes after changes.
- [ ] `cargo check` passes after changes.
- [ ] `cargo test` passes after changes.

## Out Of Scope

- Persistent task history.
- Batch translation.
- Replacing BabelDOC CLI with Python API.
- Storing the user-provided key as a permanent default in repository files.
