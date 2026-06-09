# `openstock score`

## Purpose

Store and query local stock evaluation scores by stock ID.

## Storage

Scores are stored beside `.env` under the OpenStock config directory:

```text
~/.config/openstock/scores.json
```

Each score is an integer from 0 to 100. `0` is the lowest evaluation and `100` is the highest.

## Subcommands

| Command | Purpose |
| --- | --- |
| `openstock score set <symbol> <score>` | Save or update a stock score. |
| `openstock score get <symbol>` | Read one stock score. |
| `openstock score list` | List all saved scores. |
| `openstock score delete <symbol>` | Delete one stock score. |

## Output Fields

| Field | Meaning |
| --- | --- |
| `path` | Score file path. |
| `symbol` | Normalized stock ID. |
| `score` | 0 to 100 score. |
| `updated_at_unix` | Update Unix timestamp in seconds. |
| `count` | Number of saved scores. |
| `scores` | Score records sorted by score descending. |
| `deleted` | Whether a record was deleted. |
| `removed` | Deleted record, if any. |

## Agent Notes

Use this for user- or strategy-defined evaluation state. Do not treat the score as market evidence unless the workflow defines how it was produced.
