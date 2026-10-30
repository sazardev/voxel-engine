#!/bin/bash
# SessionStart: orient the agent after startup, /clear or compaction. Stdout becomes context.
# Compact on purpose: a handful of lines, never file contents.
root=${CLAUDE_PROJECT_DIR:-$PWD}
cd "$root" || exit 0

echo "Repo state:"
echo "- branch: $(git branch --show-current), $(git status --porcelain | wc -l) changed file(s)"
git status --porcelain | head -8 | sed 's/^/  /'
echo "- last commits:"
git log --oneline -3 | sed 's/^/  /'

gates="$root/target/.claude-gates"
if [ -f "$gates" ]; then
  echo "- last fast gates (fmt, clippy, typos, dprint): $(cat "$gates")"
else
  echo "- fast gates: not run yet in this checkout"
fi

echo "- recent ADRs: $(ls docs/adr | grep -E '^[0-9]{4}-' | grep -v '^0000' | tail -3 | tr '\n' ' ')"
echo "Rails: hooks block utils/helpers modules, unwrap and #[allow] in crates/, --no-verify and force push. Run /check before finishing."
