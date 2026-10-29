# Security

Rules, not suggestions: every change follows them and a review checks them. When one has to be broken, say so in the change and why.

The game is offline and single-player, so the risks are untrusted files, the supply chain and unsafe code, not servers or accounts.

- **Validate at every boundary**: save files, settings, `.vox` models, `.yarn` scripts and mod-like content are parsed into typed values with size and range limits; reject, never trust. A corrupted save must fail with an error, never panic or allocate without bound.
- **Paths**: never build file paths from file contents without normalizing and confining them to the data directory.
- **Offline by construction**: no network crates (`cargo deny` bans them) and no `std::net` (guard test).
- **`unsafe`** is denied; any exception lives in a documented module with `// SAFETY:` and an ADR.
- **Dependencies** are pinned in `Cargo.lock` and checked by `cargo deny` (advisories, licenses, sources); a known-vulnerable version blocks the build; ignored advisories carry a reason.
- **Secrets** never in code, logs or the repository; workflow secrets come from repository or environment secrets.
- **CI**: third-party actions pinned to a full commit SHA; `permissions` default to `contents: read`.
- **No PII** collected or logged; no telemetry.
- Review check: every new file format has a size-limited, fuzzable parser and a test with malformed input; no new dependency without a license and advisory check.
