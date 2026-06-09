# `openstock universe`

## Purpose

Build and query a local universe of listed Korean stocks from KIND.

## Subcommands

| Command | Purpose |
| --- | --- |
| `openstock universe sync [--force]` | Refresh KIND universe cache. |
| `openstock universe status` | Show local universe cache status. |
| `openstock universe list [--market MARKET] [--kind KIND] [--offset N] [--limit N]` | Page through stocks. |
| `openstock universe chunks [--market MARKET] [--kind KIND] [--size N]` | Build scan chunks. |
| `openstock universe validate` | Validate expected size and representative symbols. |

## IO

| Direction | Data |
| --- | --- |
| External IO | KIND listed company download on sync/cache miss. |
| File read/write | `~/.config/openstock/cache/universe/kind`. |
| Side effect | `sync` writes cache and prunes old snapshots. Read commands can sync on cache miss. |

## Output Fields

| Field | Meaning |
| --- | --- |
| `source` | Data source. |
| `cache_date` | Cache date. |
| `refreshed_at` | Refresh timestamp. |
| `refreshed` | Whether command refreshed data. |
| `stock_count` | Number of stocks in snapshot. |
| `counts_by_market` | Count by market. |
| `total_count` | Total stocks before filtering. |
| `filtered_count` | Stocks after filter. |
| `offset` | Pagination offset. |
| `limit` | Pagination limit. |
| `stocks` | Normalized stock records. |
| `chunk_size` | Requested chunk size. |
| `chunk_count` | Number of chunks. |
| `chunks` | Scan chunk list. |

## Agent Notes

Use `chunks` when scanning many stocks. Do not request the whole universe if a paged or chunked workflow is enough.
