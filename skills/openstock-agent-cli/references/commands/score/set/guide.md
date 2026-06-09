# `openstock score set`

Purpose: save or update a stock evaluation score.

Usage:

```bash
openstock score set 005930 87
```

Inputs: `symbol`, `score` from 0 to 100.

Writes: `~/.config/openstock/scores.json`.

Output fields: `path`, `symbol`, `score`, `updated_at_unix`.

Agent rule: use only when the user or scoring workflow has produced a concrete score.
