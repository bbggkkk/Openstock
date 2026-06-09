# OpenStock CLI Command Map

This directory mirrors the `openstock` CLI command depth for agent execution.

| CLI Depth | Folder |
| --- | --- |
| `openstock version` | `version/` |
| `openstock update` | `update/` |
| `openstock search` | `search/` |
| `openstock api ...` | `api/` |
| `openstock account ...` | `account/` |
| `openstock market ...` | `market/` |
| `openstock universe ...` | `universe/` |
| `openstock dart ...` | `dart/` |
| `openstock order ...` | `order/` |
| `openstock cache ...` | `cache/` |

Subcommands also have leaf folders matching CLI depth, for example:

| CLI Depth | Folder |
| --- | --- |
| `openstock api login` | `api/login/` |
| `openstock account status` | `account/status/` |
| `openstock market history` | `market/history/` |
| `openstock universe chunks` | `universe/chunks/` |
| `openstock dart show` | `dart/show/` |
| `openstock order buy` | `order/buy/` |
| `openstock cache prune` | `cache/prune/` |

Agents should read `../../SKILL.md` first, then open the folder matching the command depth they intend to run.
