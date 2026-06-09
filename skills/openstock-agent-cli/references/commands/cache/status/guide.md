# `openstock cache status`

Purpose: inspect local cache size and namespace summary.

Usage:

```bash
openstock cache status
```

Reads: `~/.config/openstock/cache`.

Output fields: `root`, `exists`, `total_files`, `total_bytes`, `namespaces`.

Agent rule: use before cache cleanup or when diagnosing stale local data.
