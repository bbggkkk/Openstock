# `openstock universe status`

Purpose: inspect local KIND universe cache status.

Usage:

```bash
openstock universe status
```

Reads: universe cache.

Output fields: `source`, `cache_date`, `refreshed_at`, `refreshed`, `stock_count`, `counts_by_market`.

Agent rule: if cache is missing, run `openstock universe sync`.
