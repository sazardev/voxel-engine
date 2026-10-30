#!/bin/sh
# Stop hook: when files changed, run the fast gates before the agent may finish.
# Exit 2 sends the output back to the agent so it fixes the problem (same commands as CI).
input=$(cat)
case "$input" in
  *'"stop_hook_active":true'* | *'"stop_hook_active": true'*) exit 0 ;;
esac

cd "$CLAUDE_PROJECT_DIR" || exit 0
[ -z "$(git status --porcelain)" ] && exit 0
[ -f "$HOME/.cargo/env" ] && . "$HOME/.cargo/env"

state="$CLAUDE_PROJECT_DIR/target/.claude-gates"
mkdir -p "$CLAUDE_PROJECT_DIR/target"

output=$( (
  cargo fmt --all --check &&
    cargo clippy --workspace --all-targets --all-features -- -D warnings &&
    typos &&
    dprint check
) 2>&1) || {
  # Read back by session-context.sh and prompt-context.sh.
  printf 'FAIL %s %s\n' "$(date -Iseconds)" "$(echo "$output" | grep -m1 -E 'error|Diff|typos' | cut -c1-120)" >"$state"
  echo "Quality gates failed. Fix before finishing:" >&2
  echo "$output" | tail -40 >&2
  exit 2
}
printf 'ok %s\n' "$(date -Iseconds)" >"$state"
