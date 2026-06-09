# `openstock score list`

Purpose: list saved stock evaluation scores.

Usage:

```bash
openstock score list
```

Reads: `~/.config/openstock/scores.json`.

Output fields: `path`, `count`, `scores`.

Agent rule: scores are sorted by score descending, then symbol ascending.
