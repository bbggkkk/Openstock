# `openstock universe sync`

Purpose: refresh KIND listed stock universe cache.

Usage:

```bash
openstock universe sync
openstock universe sync --force
```

External IO: KIND listed company download.

Writes: `~/.config/openstock/cache/universe/kind`.

Output fields: `source`, `cache_date`, `refreshed_at`, `refreshed`, `stock_count`, `counts_by_market`.

Agent rule: run before broad scans if the cache date matters.
