# `openstock universe chunks`

Purpose: divide the stock universe into scan chunks.

Usage:

```bash
openstock universe chunks --market KOSDAQ --kind common_stock --size 100
```

Inputs: optional `--market`, `--kind`, `--size`.

Reads: universe cache.

Output fields: `source`, `cache_date`, `filtered_count`, `chunk_size`, `chunk_count`, `chunks`.

Agent rule: preferred for full-market scan planning.
