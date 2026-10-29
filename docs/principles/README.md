# Engineering principles

Rules, not suggestions: every change follows them and a review checks them. When one has to be broken, say so in the change and why. Each file ends with a review check. Mechanical enforcement is listed in `AGENTS.md` and ADR 0007.

| Principle                    | File                             | Applies to                           |
| ---------------------------- | -------------------------------- | ------------------------------------ |
| Clean Code                   | `clean-code.md`                  | all code                             |
| SOLID                        | `solid.md`                       | all code                             |
| One file, one responsibility | `one-file-one-responsibility.md` | layout of modules and files          |
| Hexagonal architecture       | `hexagonal-architecture.md`      | crate boundaries, ports and adapters |
| Error handling               | `error-handling.md`              | every fallible operation             |
| Data contracts               | `data-contracts.md`              | ports, save and settings formats     |
| Performance                  | `performance.md`                 | frame, memory, loading budgets       |
| Security                     | `security.md`                    | files, dependencies, unsafe, CI      |
| Testing                      | `testing.md`                     | every changed behaviour              |

An API-design principle set (REST, idempotency, pagination) is intentionally absent: there is no network API. `data-contracts.md` replaces it.
