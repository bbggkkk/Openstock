# `openstock dart disclosures`

Purpose: query disclosure lists from OpenDART.

Usage:

```bash
openstock dart disclosures 005930 --from 20260601 --to 20260609
openstock dart disclosures --corp-code 00126380 --page-count 50
```

Inputs: optional `symbol`, `--corp-code`, `--from`, `--to`, `--corp-cls`, `--page-no`, `--page-count`.

Reads: `OPENDART_API_KEY`; corp-code cache when resolving a symbol.

External IO: OpenDART list API.

Output fields: `query`, `resolved`, `disclosures`.

Agent rule: use for disclosure evidence lists without fetching document text.
