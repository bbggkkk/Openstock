# `openstock score delete`

Purpose: delete a saved stock evaluation score.

Usage:

```bash
openstock score delete 005930
```

Inputs: `symbol`.

Writes: `~/.config/openstock/scores.json` when a record exists.

Output fields: `path`, `symbol`, `deleted`, `removed`.

Agent rule: deletion changes local evaluation state; run only when requested.
