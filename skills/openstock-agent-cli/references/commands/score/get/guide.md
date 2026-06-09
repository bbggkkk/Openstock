# `openstock score get`

Purpose: read a saved stock evaluation score.

Usage:

```bash
openstock score get 005930
```

Inputs: `symbol`.

Reads: `~/.config/openstock/scores.json`.

Output fields: `path`, `symbol`, `score`, `updated_at_unix`.

Agent rule: if the command fails with no saved score, do not infer a score of 0.
