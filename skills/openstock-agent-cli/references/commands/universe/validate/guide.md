# `openstock universe validate`

Purpose: validate universe size and representative stock expectations.

Usage:

```bash
openstock universe validate
openstock universe validate --expect 005930=삼성전자
```

Inputs: minimum count options and repeated `--expect SYMBOL=NAME`.

Reads: universe cache.

Output fields: validation checks and status.

Agent rule: run after sync when verifying that the cached universe is usable.
