---
description: Create a numbered Architecture Decision Record from the template
argument-hint: <short title>
---

Create an ADR for: $ARGUMENTS

1. Read `docs/adr/README.md` and pick the next number.
2. Copy `docs/adr/0000-template.md` to `docs/adr/NNNN-kebab-title.md`; fill Status, Date, Requirements, Context, Decision, Consequences and Alternatives considered.
3. Add a row to the table in `docs/adr/README.md`. If it replaces an ADR, mark the old one `superseded by NNNN` in its status line and in the index.
4. Run `/check` (the `guard` tests verify the index and sections).
