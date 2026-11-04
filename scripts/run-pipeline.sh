#!/usr/bin/env bash
# Builds rust-forge through loki-mode in stages. Every implementation run is followed by a validation run in a
# fresh loki session, so the code is never checked by the context that wrote it. The pipeline stops at the first
# run that does not end VERIFIED with a receipt that `loki verify` accepts.
#
# Works with the bash 3.2 that ships with macOS.
set -euo pipefail

STAGES='
tools         impl   docs/prd/00-core-tools.md
v-tools       valid  docs/prd/00-core-tools.md
core          impl   docs/prd/00-core.md
v-core        valid  docs/prd/00-core.md
cli           impl   docs/prd/01-cli.md
v-cli         valid  docs/prd/01-cli.md
game          impl   docs/prd/02-game.md
v-game        valid  docs/prd/02-game.md
ffi-mobile    impl   docs/prd/03-ffi-mobile.md
v-ffi-mobile  valid  docs/prd/03-ffi-mobile.md
ml            impl   docs/prd/04-ml.md
v-ml          valid  docs/prd/04-ml.md
web           impl   docs/prd/05-web.md
v-web         valid  docs/prd/05-web.md
desktop       impl   docs/prd/06-desktop.md
v-desktop     valid  docs/prd/06-desktop.md
wasm          impl   docs/prd/07-wasm.md
v-wasm        valid  docs/prd/07-wasm.md
embedded      impl   docs/prd/08-embedded.md
v-embedded    valid  docs/prd/08-embedded.md
automation    impl   docs/prd/09-automation.md
v-automation  valid  docs/prd/09-automation.md
journal       impl   docs/prd/10-journal.md
v-journal     valid  docs/prd/10-journal.md
final         final  docs/prd/99-final.md
'

# Spend caps per run in USD. Rough starting points, not measurements: core is the largest piece of work,
# a domain stage is mostly one skill plus a fixture, a validation run reads more than it writes.
COST_CORE=${FORGE_COST_CORE:-40}
COST_DOMAIN=${FORGE_COST_DOMAIN:-15}
COST_VALID=${FORGE_COST_VALID:-10}
COST_FINAL=${FORGE_COST_FINAL:-15}
PUSH=${FORGE_PUSH:-1}

# loki engine10 defaults are tuned for small JS/Python tasks and failed the first core run:
# - it split the PRD into parallel units with no dependencies between them (README started before the code it
#   documents), so LOKI_SPEED=0 makes each PRD one sequential run;
# - it stops a run at a fixed wall-clock cap (implement and fix phases are capped too), too short for Rust with
#   compilation, so --deep raises the caps.
export LOKI_SPEED=${LOKI_SPEED:-0}
LOKI_FLAGS=${FORGE_LOKI_FLAGS:---deep}

usage() {
  cat <<'EOF'
Usage: scripts/run-pipeline.sh [--from <stage>] [--only <stage>] [--dry-run]

Runs the stages below in order with `loki start <prd> --no-pr --max-cost <usd> --deep`, checks each receipt with
`loki verify`, fast-forwards main to the run's commits and pushes main to origin.

Stages:
  tools v-tools core v-core cli v-cli game v-game ffi-mobile v-ffi-mobile ml v-ml
  web v-web desktop v-desktop wasm v-wasm embedded v-embedded
  automation v-automation journal v-journal final

Options:
  --from <stage>   start at this stage (resume after fixing a stopped run)
  --only <stage>   run just this stage
  --dry-run        print the plan and exit

Environment:
  FORGE_COST_CORE (40)  FORGE_COST_DOMAIN (15)  FORGE_COST_VALID (10)  FORGE_COST_FINAL (15)
  FORGE_PUSH (1)        set to 0 to keep everything local
  FORGE_LOKI_FLAGS (--deep)   extra flags for every loki start
  LOKI_SPEED (0)        0 = one sequential run per PRD; loki's own default splits PRDs into parallel units
  LOKI_AUTO_CONFIRM     set to true when running without a terminal (for example from a Claude Code session)
EOF
}

log() { printf '\n[pipeline] %s\n' "$*"; }
die() { printf '\n[pipeline] STOP: %s\n' "$*" >&2; exit 1; }

FROM='' ONLY='' DRY=0
while [ $# -gt 0 ]; do
  case $1 in
    --from) FROM=${2:?--from needs a stage}; shift 2 ;;
    --only) ONLY=${2:?--only needs a stage}; shift 2 ;;
    --dry-run) DRY=1; shift ;;
    -h|--help) usage; exit 0 ;;
    *) usage; die "unknown argument: $1" ;;
  esac
done

stage_names() { printf '%s\n' "$STAGES" | awk 'NF==3 {print $1}'; }
for s in $FROM $ONLY; do
  stage_names | grep -qx "$s" || die "unknown stage: $s (see --help)"
done

cost_for() { # name kind
  case $2 in
    impl) case $1 in tools|core|automation|journal) echo "$COST_CORE" ;; *) echo "$COST_DOMAIN" ;; esac ;;
    valid) echo "$COST_VALID" ;;
    final) echo "$COST_FINAL" ;;
  esac
}

preflight() {
  for cmd in git loki cargo; do
    command -v "$cmd" >/dev/null 2>&1 || die "$cmd not found on PATH"
  done
  git rev-parse --show-toplevel >/dev/null 2>&1 || die "run from inside the rust-forge git repository"
  cd "$(git rev-parse --show-toplevel)"
  [ "$(git rev-parse --abbrev-ref HEAD)" = main ] || die "check out main first"
  [ -z "$(git status --porcelain --untracked-files=no)" ] || die "working tree has uncommitted changes"
  if [ "$PUSH" = 1 ]; then
    git remote get-url origin >/dev/null 2>&1 || die "no origin remote (or set FORGE_PUSH=0)"
  fi
  [ -f docs/prd/validate.md ] || die "docs/prd/validate.md missing"
}

# Validation runs get a PRD composed from the shared template, written under target/ (git-ignored) so it is
# regenerated each time and never committed.
compose_validation_prd() { # stage target_prd -> path
  local out="target/pipeline/validate-$1.md"
  mkdir -p target/pipeline
  sed -e "s|{{STAGE}}|$1|g" -e "s|{{TARGET_PRD}}|$2|g" docs/prd/validate.md > "$out"
  echo "$out"
}

# loki may commit on main or on its own branch and leave either checked out. Find the branch with the new
# commits and fast-forward main to it; never merge or rewrite history.
integrate() { # base_sha stage
  local base=$1 stage=$2 branch newest
  branch=$(git rev-parse --abbrev-ref HEAD)
  if [ "$branch" = main ] && [ "$(git rev-parse HEAD)" = "$base" ]; then
    newest=$(git for-each-ref --sort=-committerdate --format='%(refname:short)' refs/heads | head -n 1)
    if [ -n "$newest" ] && [ "$newest" != main ] \
       && git merge-base --is-ancestor main "$newest" \
       && [ "$(git rev-parse "$newest")" != "$(git rev-parse main)" ]; then
      branch=$newest
    fi
  fi
  if [ "$branch" != main ]; then
    git checkout -q main
    git merge --ff-only -q "$branch" || die "$stage: cannot fast-forward main to $branch; inspect and merge by hand"
  fi
  [ "$(git rev-parse HEAD)" != "$base" ] || die "$stage: run ended VERIFIED but main has no new commits; find loki's branch (loki status) and merge it, then rerun with --from $(next_stage "$stage")"
  if [ "$PUSH" = 1 ]; then
    git push -q origin main || die "$stage: push to origin failed; push by hand, then rerun with --from $(next_stage "$stage")"
  fi
}

run_stage() { # name kind prd
  local name=$1 kind=$2 prd=$3 cost prd_file base rc
  cost=$(cost_for "$name" "$kind")
  [ -f "$prd" ] || die "$name: $prd missing"
  if [ "$kind" = valid ]; then prd_file=$(compose_validation_prd "$name" "$prd"); else prd_file=$prd; fi

  log "=== $name ($kind) · $prd_file · max \$$cost"
  if [ "$DRY" = 1 ]; then return 0; fi

  base=$(git rev-parse HEAD)
  set +e
  # shellcheck disable=SC2086
  loki start "$prd_file" --no-pr --max-cost "$cost" $LOKI_FLAGS
  rc=$?
  set -e
  case $rc in
    0) ;;
    3) die "$name: BUDGET_STOP. Raise the cap (see --help) and rerun with --from $name" ;;
    4) die "$name: BLOCKED, loki asked a question (see loki status). Add the answer to $prd, commit it, rerun with --from $name" ;;
    5) die "$name: STALLED. Read loki status, fix or narrow the PRD, rerun with --from $name" ;;
    *) die "$name: run did not verify (exit $rc). Read loki status, then rerun with --from $name" ;;
  esac

  set +e
  loki verify
  rc=$?
  set -e
  [ $rc -eq 0 ] || die "$name: loki verify exit $rc (0 is the only verified result)"

  integrate "$base" "$name"
  log "$name done: $(git rev-parse --short HEAD)"
}

next_stage() { stage_names | awk -v s="$1" 'found {print; exit} $0==s {found=1}'; }

main() {
  preflight
  local started=0 name kind prd
  [ -z "$FROM" ] && started=1
  # Iterate over positional words, not a `while read` loop: loki is interactive and must keep the terminal
  # as stdin instead of swallowing the stage list.
  set -f
  # shellcheck disable=SC2086
  set -- $STAGES
  set +f
  while [ $# -ge 3 ]; do
    name=$1 kind=$2 prd=$3
    shift 3
    if [ -n "$ONLY" ]; then
      [ "$name" = "$ONLY" ] || continue
    elif [ "$started" = 0 ]; then
      [ "$name" = "$FROM" ] || continue
      started=1
    fi
    run_stage "$name" "$kind" "$prd"
  done
  log "all requested stages done"
}

main
