#!/bin/bash
# Self-test for the guard hooks: feeds sample tool calls and checks exit codes.
# Run: .claude/hooks/test-hooks.sh   (no cargo needed)
cd "$(dirname "$0")/../.." || exit 1
export CLAUDE_PROJECT_DIR=$PWD
fail=0
expect() { # name expected_exit script json
  out=$(echo "$4" | ".claude/hooks/$3" 2>&1); code=$?
  if [ "$code" -ne "$2" ]; then echo "FAIL $1: exit $code, wanted $2: $out"; fail=1; else echo "ok   $1"; fi
}
w() { jq -nc --arg p "$PWD/$1" --arg c "$2" '{tool_name:"Write",tool_input:{file_path:$p,content:$c}}'; }
b() { jq -nc --arg c "$1" '{tool_name:"Bash",tool_input:{command:$c}}'; }

expect "blocks utils.rs"            2 guard-write.sh "$(w crates/voxel-core/src/utils.rs 'fn a() {}')"
expect "blocks helpers/ folder"     2 guard-write.sh "$(w crates/game/src/helpers/x.rs 'fn a() {}')"
expect "allows chunk_palette.rs"    0 guard-write.sh "$(w crates/voxel-core/src/chunk_palette.rs 'fn a() {}')"
expect "blocks bevy in voxel-core"  2 guard-write.sh "$(w crates/voxel-core/Cargo.toml $'[dependencies]\nbevy = "0.16"')"
expect "allows bevy in game"        0 guard-write.sh "$(w crates/game/Cargo.toml $'[dependencies]\nbevy = "0.16"')"
expect "blocks reqwest"             2 guard-write.sh "$(w crates/game/Cargo.toml $'reqwest = "0.12"')"
expect "blocks #[allow"             2 guard-write.sh "$(w crates/game/src/a.rs $'#[allow(dead_code)]\nfn a() {}')"
expect "allows #[expect"            0 guard-write.sh "$(w crates/game/src/a.rs $'#[expect(dead_code, reason = "x")]\nfn a() {}')"
expect "blocks unwrap in src"       2 guard-write.sh "$(w crates/game/src/a.rs 'fn a() { x.unwrap(); }')"
expect "allows unwrap_or"           0 guard-write.sh "$(w crates/game/src/a.rs 'fn a() { x.unwrap_or(1); }')"
expect "allows unwrap in tests mod" 0 guard-write.sh "$(w crates/game/src/a.rs $'fn a() {}\n#[cfg(test)]\nmod t { fn b() { x.unwrap(); } }')"
expect "allows unwrap in tests/"    0 guard-write.sh "$(w crates/game/tests/a.rs 'fn a() { x.unwrap(); }')"
expect "asks on lefthook.yml"       0 guard-write.sh "$(w lefthook.yml 'x')"
[ -n "$(echo "$(w lefthook.yml x)" | .claude/hooks/guard-write.sh)" ] && echo "ok   lefthook.yml asks" || { echo "FAIL lefthook.yml should ask"; fail=1; }

expect "blocks --no-verify"         2 guard-bash.sh "$(b 'git commit --no-verify -m x')"
expect "blocks commit -n"           2 guard-bash.sh "$(b 'git commit -n -m x')"
expect "blocks force push"          2 guard-bash.sh "$(b 'git push --force origin main')"
expect "blocks push -f"             2 guard-bash.sh "$(b 'git push -f')"
expect "allows normal push"         0 guard-bash.sh "$(b 'git push origin main')"
expect "blocks reset --hard"        2 guard-bash.sh "$(b 'git reset --hard HEAD~1')"
expect "blocks deny -c"             2 guard-bash.sh "$(b 'cargo deny -c /tmp/x.toml check')"
expect "allows cargo deny check"    0 guard-bash.sh "$(b 'cargo deny check')"
expect "blocks clippy -A"           2 guard-bash.sh "$(b 'cargo clippy -- -A warnings')"
expect "allows clippy -D"           0 guard-bash.sh "$(b 'cargo clippy --workspace -- -D warnings')"
expect "blocks rm -rf /"            2 guard-bash.sh "$(b 'rm -rf /')"
expect "blocks rm -rf ~/x"          2 guard-bash.sh "$(b 'rm -rf ~/x')"
expect "blocks rm -rf ../x"         2 guard-bash.sh "$(b 'rm -rf ../x')"
expect "allows rm -rf in repo"      0 guard-bash.sh "$(b "rm -rf $PWD/target/tmp")"
expect "allows rm -rf relative"     0 guard-bash.sh "$(b 'rm -rf target/tmp')"
expect "allows rm single file"      0 guard-bash.sh "$(b 'rm foo.txt')"
expect "blocks sed -i lefthook"     2 guard-bash.sh "$(b "sed -i 's/a/b/' lefthook.yml")"
expect "allows cat lefthook"        0 guard-bash.sh "$(b 'cat lefthook.yml')"
expect "allows 2>&1 with deny.toml" 0 guard-bash.sh "$(b 'cargo build 2>&1 | head')"
exit $fail
