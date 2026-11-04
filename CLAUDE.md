# rust-forge

- Start loki runs only through `scripts/run-pipeline.sh`. Do not run `loki start` or any loki command with a PRD or
  task argument directly, including with config or debug variables such as `LOKI_CONFIG_DUMP`: in this loki version
  that starts a real, paid run and can delete branches of earlier runs.
- Specs live in `docs/prd/`; validation reports in `docs/validation/`.
