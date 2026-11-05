#!/usr/bin/env bash
# Run the rust-forge eval suite with `claude plugin eval`.
# Usage: scripts/plugin-eval.sh [plugin-path] [extra claude plugin eval flags...]
#
# `claude plugin eval` refuses a plugin directory that contains hard links, and cargo fills `target/` with them,
# so the suite runs on a clean copy of the plugin without `target/`, `.git` and `.loki`.
#
# Cases that compile Rust need a toolchain the eval sandbox can read. On macOS it cannot: the sandbox blocks loading
# rustc's libraries from anywhere under /Users, and cargo cannot write its registry cache (README "Evals").
# Those cases are meant to run in Linux CI with the toolchain and forge-tools installed outside the home directory
# and on PATH.
set -uo pipefail

plugin="."
if [[ $# -gt 0 && $1 != -* ]]; then
  plugin=$1
  shift
fi
plugin=$(cd "$plugin" && pwd) || exit 2
work=$(mktemp -d "${TMPDIR:-/tmp}/rust-forge-eval.XXXXXX") || exit 2
trap 'rm -rf "$work"' EXIT
rsync -a --exclude target --exclude .git --exclude .loki --exclude evals/results "$plugin/" "$work/rust-forge/" || exit 2

args=(
  plugin eval "$work/rust-forge"
  --trust-plugin
  --scaffold
  --threshold "${FORGE_EVAL_THRESHOLD:-0.8}"
  --no-publish
  --max-cost-usd "${FORGE_EVAL_MAX_COST:-15}"
)
if [[ -n ${FORGE_EVAL_MODEL:-} ]]; then
  args+=(--model "$FORGE_EVAL_MODEL")
fi
if [[ -n ${FORGE_EVAL_JUDGE_MODEL:-} ]]; then
  args+=(--judge-model "$FORGE_EVAL_JUDGE_MODEL")
fi
# Results would otherwise land in the temporary copy and be deleted with it.
if [[ " $* " != *" --output-dir "* ]]; then
  args+=(--output-dir "$plugin/target/plugin-eval/$(date +%Y%m%dT%H%M%S)")
fi
args+=(
  --allow-tools
  Bash
  Edit
  Write
  "WebFetch(domain:index.crates.io)"
  "WebFetch(domain:static.crates.io)"
  "WebFetch(domain:crates.io)"
)

# The target comes before --allow-tools; extra flags such as --case, --tag, --ablation and --json
# follow it as separate options.
claude "${args[@]}" "$@"
exit $?
