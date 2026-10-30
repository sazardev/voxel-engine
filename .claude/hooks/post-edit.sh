#!/bin/bash
# PostToolUse (Edit|Write): format the edited .rs file and type-check only its crate.
# Silent on success. On failure exit 2 with a short, capped error list (cheap in tokens).
input=$(cat)
file=$(jq -r '.tool_input.file_path // empty' <<<"$input")
[[ "$file" == *.rs ]] || exit 0
root=${CLAUDE_PROJECT_DIR:-$PWD}
rel=${file#"$root"/}
case "$rel" in crates/*/*) ;; *) exit 0 ;; esac
[ -f "$file" ] || exit 0
[ -f "$HOME/.cargo/env" ] && . "$HOME/.cargo/env"

rustfmt --edition 2024 "$file" >/dev/null 2>&1

dir=${rel#crates/}; dir=${dir%%/*}
pkg=$(sed -nE 's/^name = "([^"]+)".*/\1/p' "$root/crates/$dir/Cargo.toml" | head -1)
[ -z "$pkg" ] && exit 0

if ! out=$(cd "$root" && cargo check -p "$pkg" --all-features --message-format short --quiet 2>&1); then
  {
    echo "cargo check -p $pkg failed after editing $rel (first lines):"
    grep -E 'error|warning: unused' <<<"$out" | head -15
  } >&2
  exit 2
fi

lines=$(wc -l <"$file")
if [ "$lines" -gt 450 ]; then
  jq -n --arg m "$rel has $lines lines; guard fails files over 500. Split it by responsibility before it grows." \
    '{hookSpecificOutput:{hookEventName:"PostToolUse",additionalContext:$m}}'
fi
exit 0
