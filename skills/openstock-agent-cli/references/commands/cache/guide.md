# `openstock cache`

## Purpose

Inspect and prune local cache under `~/.config/openstock/cache`.

## Subcommands

| Command | Purpose |
| --- | --- |
| `openstock cache status` | Show cache size and namespace summary. |
| `openstock cache prune [--dry-run]` | Apply cache retention policy. |

## IO

| Direction | Data |
| --- | --- |
| File read | Cache directories. |
| File delete | Old cache files unless `--dry-run` is used. |
| External IO | none |
| Side effect | `prune` deletes files unless dry-run. |

## Output Fields

| Field | Meaning |
| --- | --- |
| `root` | Cache root path. |
| `exists` | Whether cache root exists. |
| `total_files` | Number of cache files. |
| `total_bytes` | Total cache size. |
| `namespaces` | Cache namespace summaries. |
| `dry_run` | Whether prune only reported actions. |
| `deleted_files` | Number of deleted files. |
| `deleted_bytes` | Bytes removed. |
| `reports` | Prune reports by namespace. |

## Agent Notes

Run `cache prune --dry-run` before destructive pruning unless the user clearly asks to clean cache.
