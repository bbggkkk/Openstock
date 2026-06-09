---
name: openstock-agent-cli
description: Use when operating OpenStock through its CLI for Korean stock search, KIS market/account/order commands, KIND universe scans, OpenDART disclosures, cache management, or agent-safe trading evidence workflows. This skill intentionally avoids MCP/plugin wrappers and relies on direct CLI execution.
---

# OpenStock Agent CLI Skill

Use this skill when an agent needs Korean stock search, market data, KIS account/order access, KIND universe data, or OpenDART disclosure evidence from this repository.

## Strategy

Do not start an MCP server or create a plugin wrapper. Use the installed `openstock` CLI directly and treat its stdout/stderr JSON as the tool contract.

The CLI is the single integration surface for humans and agents:

```bash
openstock search 삼성전자
openstock market 005930
openstock dart show 005930 --from 20260601 --to 20260609
openstock account status
```

## Runtime Model

- `openstock` is a normal process per command invocation.
- It is not a daemon.
- It does not open a port.
- It reads config and cache from `~/.config/openstock`.
- It prints one JSON object to stdout on success.
- It prints one JSON object to stderr on failure.

## Output Contract

Every command returns explained JSON:

```json
{
  "command": "logical command name",
  "description": "human and agent readable result description",
  "fields": [
    {
      "name": "field_name",
      "description": "field meaning",
      "value": "actual value"
    }
  ],
  "raw": null
}
```

Agent behavior:

- Read `fields[].description` before interpreting `fields[].value`.
- Prefer typed commands over `api call` when a typed command exists.
- Use `raw` only when the command explicitly returns raw API data.
- Preserve numeric strings from broker APIs; do not coerce them unless calculation is required.
- Treat stderr JSON as a command failure even when it is parseable.

## Safety Rules

- `order buy` and `order sell` are live KIS orders.
- Do not place orders unless the user explicitly requests a real order in the current task.
- Before any order, check `account status`, target symbol evidence, quantity, order type, and price.
- For evidence gathering, use read-only commands: `search`, `market`, `market history`, `dart`, `universe`, `account status`, `order status`.
- Never infer that a missing cache means missing data; run the relevant `sync` command when appropriate.

## Command Documentation

Commands are documented by CLI depth under `references/commands`.

| Command | Documentation |
| --- | --- |
| `openstock version` | `references/commands/version/guide.md` |
| `openstock update` | `references/commands/update/guide.md` |
| `openstock search` | `references/commands/search/guide.md` |
| `openstock api *` | `references/commands/api/guide.md` |
| `openstock account *` | `references/commands/account/guide.md` |
| `openstock market *` | `references/commands/market/guide.md` |
| `openstock universe *` | `references/commands/universe/guide.md` |
| `openstock dart *` | `references/commands/dart/guide.md` |
| `openstock order *` | `references/commands/order/guide.md` |
| `openstock cache *` | `references/commands/cache/guide.md` |

## Recommended Evidence Flow

```bash
openstock api login
openstock universe sync
openstock dart sync
openstock search 삼성전자
openstock market 005930
openstock market history 005930 --from 20260101 --to 20260609
openstock dart show 005930 --from 20260601 --to 20260609 --index 1
openstock account status
```

This flow gathers evidence. It is not sufficient by itself for unattended trading.
