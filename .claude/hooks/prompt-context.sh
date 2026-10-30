#!/bin/bash
# UserPromptSubmit: add only what is new or relevant to this prompt. Stdout becomes context.
# Token rule: repo state is sent only when it changed since the last prompt of the session;
# requirement text and ADR names only when the prompt mentions an FR-*/NFR-* or a crate.
input=$(cat)
prompt=$(jq -r '.prompt // empty' <<<"$input")
session=$(jq -r '.session_id // "none"' <<<"$input")
root=${CLAUDE_PROJECT_DIR:-$PWD}
cd "$root" || exit 0

gates=$(cat target/.claude-gates 2>/dev/null)
state="$(git branch --show-current) | $(git status --porcelain | wc -l) changed | ${gates%% *}"
cache="${TMPDIR:-/tmp}/voxel-prompt-$session"
if [ "$(cat "$cache" 2>/dev/null)" != "$state" ]; then
  echo "$state" >"$cache"
  echo "[repo] $state"
  case "$gates" in FAIL*) echo "[last gates] $gates" ;; esac
fi

# Requirements mentioned in the prompt: their lines from SPEC.md (max 3, 8 lines each).
for id in $(grep -oE '\bN?FR-[0-9]+\b' <<<"$prompt" | sort -u | head -3); do
  awk -v id="$id" '
    $0 ~ "^\\*\\*" id ":" { on=1; n=0 }
    on && ($0 ~ /^\*\*(N?FR)-[0-9]+:/ && $0 !~ "^\\*\\*" id ":" || /^## /) { on=0 }
    on && NF && n++ < 8 { print }
  ' SPEC.md
  adrs=$(grep -lE "\b$id\b" docs/adr/[0-9]*.md 2>/dev/null | xargs -n1 basename 2>/dev/null | tr '\n' ' ')
  [ -n "$adrs" ] && echo "[adr for $id] $adrs"
done

# Crates mentioned: remind which ADRs to read (names only).
for crate in $(grep -oE 'voxel-(core|gen|mesh)|(platform|game) crate' <<<"$prompt" | sed 's/ crate//' | sort -u); do
  adrs=$(grep -lE "\b$crate\b" docs/adr/[0-9]*.md 2>/dev/null | xargs -n1 basename 2>/dev/null | head -4 | tr '\n' ' ')
  [ -n "$adrs" ] && echo "[adr for $crate] $adrs(read before changing it)"
done
exit 0
