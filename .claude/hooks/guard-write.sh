#!/bin/bash
# PreToolUse (Edit|Write): enforce the AGENTS.md rules before a file is written.
# Exit 2 = deny (stderr goes to the agent). Protected config files ask the human instead.
# Silent and instant when nothing is wrong, so it costs no tokens.
input=$(cat)
file=$(jq -r '.tool_input.file_path // empty' <<<"$input")
[ -z "$file" ] && exit 0
content=$(jq -r '.tool_input.content // .tool_input.new_string // empty' <<<"$input")
root=${CLAUDE_PROJECT_DIR:-$PWD}
rel=${file#"$root"/}
base=$(basename "$rel")

deny() { echo "Blocked by .claude/hooks/guard-write.sh: $1" >&2; exit 2; }

# Protected rails: the human decides, the agent never silently weakens them.
case "$rel" in
  lefthook.yml | deny.toml | rust-toolchain.toml | .claude/settings.json | .claude/hooks/*)
    jq -n --arg r "$rel is a quality rail (AGENTS.md: fix the code, never the check). Approve only if you asked for this change." \
      '{hookSpecificOutput:{hookEventName:"PreToolUse",permissionDecision:"ask",permissionDecisionReason:$r}}'
    exit 0 ;;
  Cargo.toml)
    if jq -e '.tool_name == "Write" or (((.tool_input.old_string // "") + (.tool_input.new_string // "")) | test("lints|clippy|unwrap|expect_used|allow"))' <<<"$input" >/dev/null; then
      jq -n --arg r "Workspace lints or the whole root Cargo.toml are being changed. Lints are a rail; approve only if you asked for this." \
        '{hookSpecificOutput:{hookEventName:"PreToolUse",permissionDecision:"ask",permissionDecisionReason:$r}}'
      exit 0
    fi ;;
esac

case "$rel" in crates/*) ;; *) exit 0 ;; esac

# No dumping-ground modules (same names as the guard crate).
stem=${base%.rs}
case "$stem" in
  utils | util | helpers | helper | common | misc)
    deny "'$base' is a dumping ground. Name the module for what it does (e.g. chunk_palette.rs)." ;;
esac
case "/$rel/" in
  */utils/* | */util/* | */helpers/* | */helper/* | */common/* | */misc/*)
    deny "$rel sits in a utils/helpers/common/misc folder. Folders are named by feature." ;;
esac

# Dependencies: no network crates anywhere, no Bevy in the pure crates.
if [ "$base" = "Cargo.toml" ]; then
  grep -Eq '^\s*(reqwest|hyper|ureq|surf|isahc|tungstenite|tokio-tungstenite|bevy_(http|networking|renet)|renet|laminar)\b' <<<"$content" &&
    deny "network crate in $rel. The game is fully offline (no network layer)."
  grep -Eq '^\s*tokio\b.*\b(net|full)\b' <<<"$content" &&
    deny "tokio with net/full in $rel. The game is fully offline."
  case "$rel" in
    crates/voxel-*/Cargo.toml)
      grep -Eq '^\s*bevy' <<<"$content" &&
        deny "Bevy dependency in a pure crate ($rel). Keep Bevy in game/platform (ADR 0002)." ;;
  esac
  exit 0
fi

# Rust sources: no silent lint bypass, no unwrap outside tests.
[[ "$file" == *.rs ]] || exit 0
case "$rel" in crates/guard/* | */tests/* | */benches/*) exit 0 ;; esac

grep -Eq '#!?\[allow\(' <<<"$content" &&
  deny "#[allow(..)] is denied. Use #[expect(lint, reason = \"...\")] with a reason."

# Whole-file writes: only look above the inline test module. Edits: skip files that have one.
if [ "$(jq -r .tool_name <<<"$input")" = "Write" ]; then
  code=${content%%'#[cfg(test)]'*}
else
  code=$content
  grep -q '#\[cfg(test)\]' "$file" 2>/dev/null && code=""
fi
grep -Eq '\.unwrap\(\)' <<<"$code" &&
  deny ".unwrap() in non-test code. Return a typed error (thiserror), or .expect(\"invariant\") for a real invariant."
exit 0
