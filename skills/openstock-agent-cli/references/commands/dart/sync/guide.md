# `openstock dart sync`

Purpose: refresh OpenDART listed-company corp code mapping cache.

Usage:

```bash
openstock dart sync
openstock dart sync --force
```

Reads: `OPENDART_API_KEY`.

Writes: `~/.config/openstock/cache/opendart/corp_codes.json`.

Output fields: source/cache metadata and corp count.

Agent rule: run when corp-code cache is missing, stale, or explicitly requested.
