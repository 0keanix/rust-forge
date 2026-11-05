#!/usr/bin/env bash
# Run the rust-forge eval suite with `claude plugin eval`.
# Usage: scripts/plugin-eval.sh [plugin-path] [extra claude plugin eval flags...]
set -uo pipefail

plugin="."
if [[ $# -gt 0 && $1 != -* ]]; then
  plugin=$1
  shift
fi

args=(
  plugin eval "$plugin"
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
