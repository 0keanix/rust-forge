# rust-forge — how this session runs loki

This repository builds the `rust-forge` Claude Code plugin in stages. Each stage is one loki run on one PRD from
`docs/prd/`. The owner starts every stage by name; this session prepares it, shows the plan, runs it on "go",
reports, and changes `main` only on the owner's explicit acceptance. The session runs with all permissions, so these
rules are the only guard: follow them exactly. Report to the owner in Russian.

## Stages (in order)

| Stage | PRD | Cap, USD |
|---|---|---|
| tools | docs/prd/00-core-tools.md | 40 |
| v-tools | validation of 00-core-tools.md | 10 |
| core | docs/prd/00-core.md | 40 |
| v-core | validation of 00-core.md | 10 |
| cli, game, ffi-mobile, ml, web, desktop, wasm, embedded | docs/prd/01-cli.md … 08-embedded.md | 15 each |
| v-cli … v-embedded | validation of each domain PRD | 10 each |
| automation | docs/prd/09-automation.md | 40 |
| v-automation | validation of 09-automation.md | 10 |
| journal | docs/prd/10-journal.md | 40 |
| v-journal | validation of 10-journal.md | 10 |
| final | docs/prd/99-final.md | 15 |

A validation stage `v-<x>` uses a PRD built from the template:
`sed -e 's|{{STAGE}}|v-<x>|g' -e 's|{{TARGET_PRD}}|<PRD of x>|g' docs/prd/validate.md > target/stages/v-<x>.md`

## Model policy

- Implement and fix run on Opus 5.5: every `loki route` and `loki start` gets the environment variable
  `LOKI_CLAUDE_MODEL_DEVELOPMENT=claude-opus-5-5`. Do not pass `--model`: loki 10 rejects it in `loki route`, and in
  `loki start` it becomes `LOKI_MODEL_OVERRIDE`, which pins every step, not only implement and fix.
- Every other step (intake, plan, Wall, already-done check, review) keeps whatever loki chooses, but the owner sees
  it before the run (step 2) and after it (report).

## When the owner says "этап <stage>"

1. **Check.** On `main`, working tree clean, `main` equals `origin/main`, the previous stage in the table was
   accepted (except `tools`). If not, say what is wrong and stop.
2. **Plan.** Run `LOKI_SPEED=0 LOKI_CLAUDE_MODEL_DEVELOPMENT=claude-opus-5-5 loki route start <prd> --no-pr
   --max-cost <cap> --deep`.
   `loki route` prints the route and exits without running anything; it is the only loki command allowed to take a
   PRD outside step 3. Show the owner: the PRD, the cap, the flags, and a table of every step with the model loki
   will use and why (pin, router, default). Say plainly if anything differs from the model policy, or if the run
   would be split into parallel units. Then wait for "go".
3. **Run** (only after "go"). Record the base: `mkdir -p target/stages && git rev-parse HEAD >
   target/stages/<stage>.base && touch target/stages/<stage>.started`. Start in the background:
   `LOKI_AUTO_CONFIRM=true LOKI_SPEED=0 LOKI_CLAUDE_MODEL_DEVELOPMENT=claude-opus-5-5 loki start <prd> --no-pr
   --max-cost <cap> --deep > target/stages/<stage>.log 2>&1`.
   Read the first lines of the log after a minute and confirm the start line matches the plan from step 2; if it
   does not, tell the owner at once.
4. **While it runs.** Do not edit any file in the repository, do not run git commands that change refs, do not
   start another loki run. Allowed: `tail` of the log, `loki status`. Check no more often than every 10 minutes.
5. **Report** when the run ends, in this format:

   ```markdown
   ## Этап <stage>: <OUTCOME>
   Прогон <run-id> · ветка <branch> · $<cost> · <time>

   **NOT PROVEN (дословно):** <…>

   **Модели** (из блока route в receipt и из .loki/runs/<run-id>/)
   | Шаг | Запрошено | Фактически | Стоимость | Время |

   **Изменения:** <git diff --stat main...<branch>>; файлы вне границ этапа: <…>; новые зависимости: <…>

   **Наши гейты** (worktree ../rf-check-<stage>)
   | cargo fmt --all -- --check | cargo clippy --workspace --all-targets -- -D warnings | cargo test --workspace |

   **Критерии приёмки PRD**
   | Критерий | Тест | Сильный / слабый / нет | Комментарий |

   **Отклонения от PRD:** <…>
   **Рекомендация:** принять / перезапустить / доработать, и почему
   ```

   The run's receipt is the `receipt.json` under `.loki/runs/` that is newer than `target/stages/<stage>.started`;
   its run directory name is the run id, and loki's branch is `loki/<run-id>`. Never use any other `loki/*` branch.
   Remove the worktree after the gates.
6. **Accept** only when the owner writes "принять <stage>": `git merge --ff-only loki/<run-id>`,
   `git branch -d loki/<run-id>`, `git push origin main`. Then name the next stage.
   On "перезапустить <stage>" or anything else, change nothing in git and wait.

## Never

- Run `loki start`, or any loki command with a PRD or task argument other than `loki route`, outside step 3.
  Debug or config variables (for example `LOKI_CONFIG_DUMP`) can start a real, paid run and delete branches.
- Merge, push, reset, or delete branches without the owner's words for that stage.
- Edit `docs/prd/*` or this file unless the owner asks.

## Notes

- `LOKI_SPEED=0` (one sequential run per PRD instead of parallel units) comes from loki 11.125 source, not its docs;
  step 2 shows whether it took effect.

### Known loki 11.125 behaviour

Established from loki source and confirmed by the events of run `e10-20261104T132021Z-6232` (stage tools). Use it
in step 2 and in reports instead of re-reading loki source; re-check the source only if `loki route` or the start
line contradicts it.

- `loki route` prints only engine, adapter and the final model. With `LOKI_CLAUDE_MODEL_DEVELOPMENT=claude-opus-5-5`
  and `LOKI_ROUTER` unset (router off) the steps are: intake project-model on claude-sonnet-5-5; already-done check
  does not run for a PRD file (only with `LOKI_E10_ALREADY_DONE_CHECK=1`); plan is skipped when the task is sized
  normal; Wall is written by claude-sonnet-5-5 in parallel with implement; implement and fix on claude-opus-5-5.
- Wall tests are TypeScript. loki may discard them (for example for referencing paths that do not exist yet); then
  loki itself proved nothing and the evidence is the implementer's tests plus our gates.
- The cross-review in seal runs only when the verdict is VERIFIED, so a PARTIAL run has had no reviewer.
- PARTIAL (exit code 6) is expected when a stage creates or changes any `Cargo.toml` ("test configuration changed",
  "runner config changed") and when a PRD has more than 40 acceptance clauses. It says nothing about code quality;
  the report and our gates decide.
- "no tests executed" in NOT PROVEN can appear even when loki's own `cargo test` passed (empty `test_counts`).
- With the router off the receipt has no route block: take models and cost per step from
  `.loki/runs/<run-id>/events.jsonl` and `cost.per_model` in the receipt.
- With `--no-pr`, `--deep` only raises time limits (run 2700 s, implement 1800 s); the deep verify pass needs a PR.
