# Error handling

Rules, not suggestions: every change follows them and a review checks them. When one has to be broken, say so in the change and why.

- **Never swallow**: every `Result` is handled, returned or logged-and-escalated exactly once; ignored results (`let _ =`) and empty branches need a comment saying why.
- **Wrap with context**: add what was being done (`load chunk (3,0,-2): …`) while keeping the cause inspectable (`#[source]`, `thiserror`).
- **Domain errors are types**: enums such as `EditError` are declared in the crate that owns the rule; infrastructure errors stay in adapters and are translated.
- **Map at the boundary, once**: one place turns errors into UI messages, a log entry or a non-zero exit; no UI text inside core logic.
- **Fail fast at startup**: missing or invalid configuration or capabilities stop with a clear message instead of failing on first use.
- **No panics for control flow**: expected outcomes (not found, out of bounds, corrupt file) are values; `panic!` and `expect` are for programmer errors and invariants only.
- **User-safe messages**: players see what happened and what to do; internal detail goes only to logs (`tracing`).
- **Log where handled**, not at every layer: one log entry per failure, with its context.
- **Retries** only for transient errors, with a limit; a failed save is never retried blindly.
- **Recoverable by default**: a bad chunk or save slot must not take the whole game down; degrade (skip, regenerate, offer recovery) and report.
- Review check: no new code path drops an error; every new error type is mapped at the boundary.
