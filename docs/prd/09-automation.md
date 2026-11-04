# PRD 09: rust-forge — self-update: drift check, eval gate, CI and automerge

Builds on `00-core-tools.md`, `00-core.md` and the domain stages `01`–`08` (all implemented). Read `00-core.md`
§4, §7, §8 and §10 first.

## 1. Goal

Keep the plugin current without the owner doing the routine work, while making it impossible for an automatic
change to weaken the checks that judge it:

- a deterministic drift check finds domain skills whose verified crate versions fell behind a breaking release;
- a weekly GitHub Actions job turns one drift finding into a change and a pull request;
- every `forge-auto` pull request goes through a deterministic policy check and the eval gate, and is merged
  automatically when both pass and automerge is switched on;
- because the plugin has no `version` field (`00-core.md` §4), each merge to `main` reaches installs through
  marketplace auto-update.

The next stage (`10-journal.md`) adds a second source of `forge-auto` pull requests (retro). Build the gate so it
serves both kinds.

## 2. Deliver

```
tools/forge-tools/src/...            # drift, automerge-check
docs/prd/templates/drift-update.md   # PRD template filled by `drift --write-prd`
scripts/forge-gate.sh                # the eval gate, used by CI and by retro
.github/workflows/ci.yml
.github/workflows/forge-auto.yml
.github/workflows/drift.yml
README.md                            # "Automation" section
```

## 3. `forge-tools drift [--json] [--write-prd <dir>]`

- Reads `metadata.forge-verified` from every `skills/rust-domain-*/SKILL.md` (format from `00-core.md` AC-10).
- For each crate, gets the newest stable version from crates.io through the same HTTP trait as `crate-check`
  (recorded responses in tests, no network).
- Classifies each pair: `CURRENT`; `COMPATIBLE` (newer, semver-compatible: same major for ≥1.0, same minor for
  0.x); `BREAKING` (newer and not compatible); `UNKNOWN` (crate not found or network failure).
- Prints one line per pair: `<skill> <crate> <verified> -> <newest> <class>`; `--json` prints the same as a JSON
  array.
- `--write-prd <dir>` writes one PRD per `BREAKING` pair, `<dir>/drift-<skill>-<crate>-<newest>.md`, from
  `docs/prd/templates/drift-update.md` with the skill, crate, old and new version filled in.
- Exit codes: 0 nothing breaking, 4 at least one `BREAKING`, 3 any `UNKNOWN` caused by a network failure.

`docs/prd/templates/drift-update.md`: a PRD for one skill and one crate. It asks to update the skill text to the new
version using the crate's official changelog or migration guide (one version step at a time when several were
skipped), set `metadata.forge-verified` to the new version, update the version pinned by that domain's eval case
scaffold, change nothing outside that skill directory and that eval case, and leave every `graders/` file unchanged.
Its acceptance criteria are the domain criteria D-1–D-4 plus: the new `metadata.forge-verified` version equals the
target version.

## 4. `forge-tools automerge-check --base <rev> --head <rev> --kind <retro|drift>`

Deterministic policy over `git diff --name-status base...head`. Prints every violated rule; exits 0 when eligible,
1 when not, 2 on git errors.

| Rule | retro | drift |
|---|---|---|
| Changed paths are only under `skills/`, `agents/` and `evals/` | ✓ | ✓ |
| Changes under `skills/` touch exactly one skill directory | — | ✓ |
| No file under an existing case's `graders/` is modified or deleted, and no case directory is deleted | ✓ | ✓ |
| At least one new case directory is added under `evals/` | ✓ | — |
| The `model:` line of every agent is unchanged | ✓ | ✓ |
| Total added plus deleted lines ≤ `FORGE_AUTOMERGE_MAX_LINES` (default 200) | ✓ | ✓ |
| For drift: the touched skill's `metadata.forge-verified` changed, and only versions changed | — | ✓ |

The first three rows exist so an automatic change can never edit the gate that judges it: CI, hooks, tools, scripts,
PRDs and existing graders are out of reach. The line limit keeps automatic changes small enough to read in the
history. Explain each default in a comment where it is defined.

## 5. `scripts/forge-gate.sh <base-rev> <head-rev> <retro|drift>`

Runs in a checkout of the head revision. Steps, stopping at the first failure:

1. `forge-tools automerge-check --base <base> --head <head> --kind <kind>`.
2. `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`.
3. Retro only: for each new case directory, create a worktree of base under `target/forge/gate/`, copy the new case
   into its `evals/`, and run `scripts/plugin-eval.sh <worktree> --case <name> --ablation none --json <file>`. Each
   new case must score below the threshold on base. A case that already passes on base does not show a gap, so the
   change is not proven.
4. Run the full suite on head and on base with `--ablation none --json`. Head must exit 0 (every case at or above
   the threshold), and head `aggregates.overallScore` must be at least base minus `FORGE_GATE_TOLERANCE`
   (default 0.05: three runs with 2-of-3 judge votes make single scores noisy, and a small allowance stops noise
   from blocking every change; a larger drop is a real regression).
5. Any `claude plugin eval` exit code 2 (partial: cost ceiling or credentials) fails the gate.

Prints a markdown summary (rule results, per-case scores on base and head, overall scores, cost) to stdout and to
`target/forge/gate/summary.md`. Exit 0 eligible, 1 not eligible, 2 infrastructure error.

## 6. Workflows

Read the current GitHub Actions docs and the Claude Code docs for CI install, authentication
(https://code.claude.com/docs/en/authentication) and headless mode before writing them. Pin action versions.
Linux runners need `bubblewrap` and `socat` for the eval sandbox (per the plugin-evals docs); install them.

- `ci.yml`: on pull requests and pushes to `main`: fmt, clippy with `-D warnings`, `cargo test --workspace`.
- `forge-auto.yml`: on `pull_request` (labeled, synchronize) when the PR has label `forge-auto:retro` or
  `forge-auto:drift` and its head branch is in this repository (never forks). Installs Rust, Claude Code,
  `forge-tools`; runs `scripts/forge-gate.sh` with the kind taken from the label; posts the summary as a PR comment.
  Merges with squash when all of these hold: gate exit 0; repository variable `FORGE_AUTOMERGE` is `on`; no other
  `forge-auto` PR was merged in the last 24 hours. The squash commit subject starts with `[forge-auto]` and the body
  carries the gate summary. Otherwise it adds label `needs-review` and leaves the PR open.
- `drift.yml`: weekly cron plus `workflow_dispatch`. Runs `forge-tools drift --write-prd target/forge/drift`. When
  exit code is 4, takes the first PRD only (one change per week keeps cost and review load bounded), creates branch
  `forge-auto/drift-<skill>-<crate>-<version>`, runs Claude Code headless on that PRD with the plugin loaded and a
  job timeout, commits, runs `scripts/forge-gate.sh main HEAD drift`, opens the PR with label `forge-auto:drift` and
  the summary, and applies the same merge conditions as `forge-auto.yml` within this job. It does the merge itself
  because pull requests opened with the default `GITHUB_TOKEN` do not start other workflows.
- Secrets and variables, documented in README: the Claude credential the docs name for CI, `FORGE_AUTOMERGE`
  (`on` to enable, anything else disables automerge), optional `FORGE_EVAL_MODEL`, `FORGE_EVAL_JUDGE_MODEL`,
  `FORGE_EVAL_MAX_COST`.

## 7. README "Automation"

What runs when; secrets and the variable; how to turn automerge off (`FORGE_AUTOMERGE` to `off`); how to revert an
automerged change (`git revert` the `[forge-auto]` commit; the revert is itself a release); that installs receive a
merge only when marketplace auto-update is enabled; the expected cost per gate run measured once (step 5 below).

## 8. Acceptance criteria

All verified by `cargo test --workspace`, offline, unless stated. Tests never call the real `claude`.

- A-1 fmt, clippy with `-D warnings`, `cargo test --workspace` pass; the core structure and eval suite tests pass
  unchanged.
- A-2 `drift`: recorded responses cover CURRENT, COMPATIBLE and BREAKING for a 0.x crate and a ≥1.0 crate,
  NOT_FOUND, and a network failure (exit 3); `--write-prd` writes a PRD containing the skill, crate, old and new
  version, and no `{{` placeholders remain.
- A-3 `automerge-check`: one test per rule in §4 for each kind it applies to, on temporary git repositories, plus a
  happy path for each kind.
- A-4 `forge-gate.sh` on a temporary git repository, with a stub `claude` placed first on `PATH` (returns canned JSON and exit codes) covers: happy
  path (exit 0); a new retro case that already passes on base (exit 1); head below threshold (exit 1); head overall
  score below base minus tolerance (exit 1); eval exit 2 (gate exit 1); a policy violation stops before any eval.
- A-5 Workflow files parse as YAML; `forge-auto.yml` checks that the head repository equals the base repository and
  reads `FORGE_AUTOMERGE`; `actionlint` passes on all workflows when installed, otherwise the test prints SKIPPED.
- A-6 Outside `cargo test`: run `scripts/forge-gate.sh` once locally on a throwaway branch that adds a trivial retro
  case and a one-line skill edit; record the outcome and cost in README "Automation".

## 9. Out of scope

The journal and retro (`10-journal.md`); checking Rust toolchain or Claude Code releases; publishing to any public
marketplace.
