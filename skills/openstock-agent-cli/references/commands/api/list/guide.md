# `openstock api list`

Purpose: return registered broker APIs, credential requirements, capabilities, IO contracts, and side-effect metadata.

Usage:

```bash
openstock api list
```

Inputs: none.

Output fields: `count`, `apis`.

Agent rule: use this to discover broker capability metadata; it performs no external IO.
