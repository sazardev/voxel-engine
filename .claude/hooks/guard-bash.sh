#!/bin/bash
# PreToolUse (Bash): block irreversible or rail-weakening commands.
# Exit 2 = deny (stderr goes to the agent). Silent when the command is fine.
input=$(cat)
cmd=$(jq -r '.tool_input.command // empty' <<<"$input")
[ -z "$cmd" ] && exit 0
root=${CLAUDE_PROJECT_DIR:-$PWD}

deny() { echo "Blocked by .claude/hooks/guard-bash.sh: $1" >&2; exit 2; }
has() { grep -Eq -- "$1" <<<"$cmd"; }

# Hooks and checks are never bypassed.
has 'git[^;&|]*--no-verify' && deny "--no-verify. Fix what the hook reports instead (AGENTS.md)."
has 'git[[:space:]]+commit[^;&|]*[[:space:]]-[a-zA-Z]*n([[:space:]]|$)' && deny "git commit -n skips the hooks. Fix what they report."
has '(LEFTHOOK|HUSKY)=0|LEFTHOOK_EXCLUDE|core\.hooksPath' && deny "disabling hooks."
has 'cargo[[:space:]]+deny[^;&|]*[[:space:]](-c|--config|--exclude|--exclude-dev|--disable-fetch|--allow[a-z-]*)([[:space:]=]|$)' &&
  deny "cargo deny with a flag that weakens the check. Run plain 'cargo deny check'."
has 'cargo[[:space:]]+clippy[^;&|]*([[:space:]]-A|--cap-lints|[[:space:]]--allow)' && deny "clippy with lints turned off. Fix the code."
has 'RUSTFLAGS=[^;&|]*(--cap-lints|-A[[:space:]]|-Awarnings)' && deny "RUSTFLAGS that silence lints."

# History: no force pushes, no amending a published commit, no destructive resets.
has 'git[[:space:]]+push[^;&|]*([[:space:]]-f([[:space:]]|$)|--force)' && deny "force push."
if has 'git[[:space:]]+commit[^;&|]*--amend'; then
  if [ -n "$(git -C "$root" branch -r --contains HEAD 2>/dev/null)" ]; then
    deny "--amend on a commit that is already on a remote. Make a new commit."
  fi
fi
has 'git[[:space:]]+reset[^;&|]*--hard' && deny "git reset --hard discards work. Ask the user first."
has 'git[[:space:]]+clean[^;&|]*[[:space:]]-[a-zA-Z]*f' && deny "git clean -f deletes untracked files. Ask the user first."

# rm -r outside the repo (or with a target we cannot prove is inside it).
if has '(^|[;&|(]|sudo[[:space:]]+)[[:space:]]*rm[[:space:]]+([^;&|]*[[:space:]])?(-[a-zA-Z]*[rR]|--recursive)'; then
  rest=$(sed -E 's/.*(^|[;&|(]|sudo[[:space:]]+)[[:space:]]*rm[[:space:]]+//' <<<"$cmd" | sed -E 's/[;&|].*//')
  read -ra args <<<"$rest"
  for a in "${args[@]}"; do
    [[ "$a" == -* ]] && continue
    a=${a//\"/}; a=${a//\'/}
    case "$a" in
      "$root"/*) ;;
      /* | '~'* | '$'* | *..* | '*' | '.') deny "rm -r on '$a' is outside the repo or too broad. Name an explicit path inside it." ;;
    esac
  done
fi

# Editing rails through the shell (sed -i, redirects, mv...) is the same as editing them.
prot='(lefthook\.yml|deny\.toml|rust-toolchain\.toml|\.claude/(settings\.json|hooks/))'
if has "(sed|perl)[[:space:]]+-[a-zA-Z]*i[^;&|]*$prot|(tee|mv|cp|rm|truncate)[[:space:]][^;&|]*$prot|[^2&0-9]>>?[[:space:]]*[^;&|[:space:]]*$prot"; then
  deny "changing a quality rail through the shell. Use the Edit tool (it asks the user) or ask the user."
fi
exit 0
