# `openstock dart document`

Purpose: fetch and extract text from a DART disclosure document by receipt number.

Usage:

```bash
openstock dart document 20260609000000 --max-chars 20000
```

Inputs: `rcept_no`, optional `--force`, `--max-chars`.

Reads: `OPENDART_API_KEY`.

Writes: document ZIP cache.

Output fields: document metadata and extracted text.

Agent rule: use `--max-chars` to cap large documents.
