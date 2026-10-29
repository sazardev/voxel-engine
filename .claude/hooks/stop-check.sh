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

output=$( (
  cargo fmt --all --check &&
    cargo clippy --workspace --all-targets --all-features -- -D warnings &&
    typos &&
    dprint check
) 2>&1) || {
  echo "Quality gates failed. Fix before finishing:" >&2
  echo "$output" | tail -40 >&2
  exit 2
}
