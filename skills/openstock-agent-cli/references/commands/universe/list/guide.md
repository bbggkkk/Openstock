# `openstock universe list`

Purpose: page through normalized stock universe records.

Usage:

```bash
openstock universe list --market KOSPI --kind common_stock --offset 0 --limit 100
```

Inputs: optional `--market`, `--kind`, `--offset`, `--limit`.

Reads: universe cache; may refresh on cache miss.

Output fields: `source`, `cache_date`, `total_count`, `filtered_count`, `offset`, `limit`, `stocks`.

Agent rule: use pagination instead of loading the full universe when possible.
