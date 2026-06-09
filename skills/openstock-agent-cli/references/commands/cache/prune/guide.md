# `openstock cache prune`

Purpose: apply cache retention policy.

Usage:

```bash
openstock cache prune --dry-run
openstock cache prune
```

Reads: `~/.config/openstock/cache`.

Deletes: old cache files unless `--dry-run` is set.

Output fields: `dry_run`, `deleted_files`, `deleted_bytes`, `reports`.

Agent rule: run `--dry-run` first unless the user explicitly requests cleanup.
