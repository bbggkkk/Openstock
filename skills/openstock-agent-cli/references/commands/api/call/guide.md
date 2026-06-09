# `openstock api call`

Purpose: directly call a KIS GET endpoint with access-token headers.

Usage:

```bash
openstock api call /uapi/... --param tr_id=... --param KEY=VALUE
```

Inputs: `endpoint`, repeated `--param KEY=VALUE`.

Reads: KIS credentials and access token from config/env.

External IO: KIS endpoint GET.

Output fields: `broker`, `endpoint`, `params`, `request_semantics`, `response`, `response_semantics`.

Agent rule: prefer typed commands; use this only for endpoints not yet implemented as stable commands.
