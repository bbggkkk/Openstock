# `openstock market`

## Purpose

Read KIS market data for a stock symbol.

## Subcommands

| Command | Purpose |
| --- | --- |
| `openstock market <symbol>` | Current price and company/basic information. |
| `openstock market history <symbol> --from YYYYMMDD --to YYYYMMDD [--period D|W|M|Y] [--raw-price]` | OHLCV price history. |

## `market <symbol>`

### Output Fields

| Field | Meaning |
| --- | --- |
| `broker` | Broker API used for data. |
| `symbol` | Queried KRX symbol. |
| `price` | Current price and market values. |
| `company` | Stock/company basic information. |

## `market history`

### Input

| Input | Meaning |
| --- | --- |
| `symbol` | KRX symbol. |
| `--from` | Start date in `YYYYMMDD`. |
| `--to` | End date in `YYYYMMDD`. |
| `--period` | `D`, `W`, `M`, or `Y`; default `D`. |
| `--raw-price` | Use original prices instead of adjusted prices. |

### Output Fields

| Field | Meaning |
| --- | --- |
| `broker` | Broker API used for data. |
| `symbol` | Queried KRX symbol. |
| `period` | Candle period. |
| `date_range` | Requested date range. |
| `adjusted` | Whether adjusted price was requested. |
| `count` | Number of candles. |
| `candles` | Date-ascending OHLCV values. Numeric values may be strings. |
| `summary` | KIS summary data. |

## IO

| Direction | Data |
| --- | --- |
| Env read | KIS app credentials and access token. |
| External IO | KIS domestic stock quote/history endpoints. |
| Side effect | none |

## Agent Notes

Use `market` for current context and `market history` for price movement evidence. Always resolve ambiguous names with `search` first.
