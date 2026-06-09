# `openstock dart show`

Purpose: query disclosures for a symbol and fetch the selected disclosure document.

Usage:

```bash
openstock dart show 005930 --from 20260601 --to 20260609 --index 1
```

Inputs: `symbol`, optional `--from`, `--to`, `--index`, `--page-count`, `--force`, `--max-chars`.

Reads: `OPENDART_API_KEY`; corp-code and document cache.

External IO: OpenDART list and document APIs.

Output fields: `query`, `resolved`, `disclosures`, `selected`, `document`.

Agent rule: preferred command when the task asks for a specific stock's latest disclosure and contents.
